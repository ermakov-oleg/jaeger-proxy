use std::collections::HashMap;
use std::net::SocketAddr;

use log::{debug, info, warn};
use surf::url::Url;
use tide::{Request, Response, StatusCode};

use crate::web::elasticsearch::{ESClient, ESLog};
use crate::web::middlewares::access_log;
use crate::web::models::{GetTraceResponse, Log};

struct ApplicationState {
    es_client: ESClient,
    jaeger_host: Url,
}

async fn proxy_handler(req: Request<ApplicationState>) -> tide::Result {
    let mut path = req.url().clone();
    path.set_host(req.state().jaeger_host.host_str())?;
    path.set_port(req.state().jaeger_host.port()).unwrap();

    let mut res = surf::get(path).await.unwrap();
    let body = res.body_bytes().await.unwrap();
    let headers = res.headers();

    let mut resp = Response::new(StatusCode::Ok);
    headers.iter().for_each(|h| resp.append_header(h.0, h.1));
    resp.remove_header("Content-Encoding");
    resp.set_body(body);

    Ok(resp)
}

async fn added_log_handler(req: Request<ApplicationState>) -> tide::Result {
    let mut path = req.url().clone();
    path.set_host(req.state().jaeger_host.host_str())?;
    path.set_port(req.state().jaeger_host.port()).unwrap();

    let trace_id: String = req.param("trace_id").expect("Expected trace_id parameter");

    let mut res = surf::get(path).await.unwrap();

    let body: String = res.body_string().await.unwrap();

    let mut trace_response: GetTraceResponse = serde_json::from_str(body.as_ref()).unwrap();
    debug!("{:#?}", trace_response);

    let mut from: Option<u64> = None;
    let mut to: Option<u64> = None;

    for trace in (&trace_response)
        .data
        .as_ref()
        .unwrap_or(&vec![])
        .into_iter()
    {
        for span in (&trace).spans.as_ref().unwrap_or(&vec![]).iter() {
            from = match from {
                Some(v) => Some(if span.start_time > v {
                    v
                } else {
                    span.start_time
                }),
                None => Some(span.start_time),
            }
        }
    }

    if let Some(val) = from {
        // Jaeger send timestamp in µs
        let from_ms = val / 1000;

        from = Some(from_ms);
        to = Some(from_ms + (60 * 10 * 1000)); // + 10 min
    }

    if let Some(_) = from {
        let hits = req
            .state()
            .es_client
            .get_logs(&trace_id, from.unwrap(), to.unwrap())
            .await;
        if let Some(logs) = hits {
            let mut span_logs: HashMap<String, Vec<ESLog>> = HashMap::new();

            for log in logs {
                let x_trace_id = log.x_trace_id.clone();

                if let Some(x_trace_id) = x_trace_id {
                    let parts: Vec<&str> = x_trace_id.split(":").collect();
                    if parts.len() == 4 {
                        let span_id = parts[1].to_string();
                        (*span_logs.entry(span_id).or_insert(vec![])).push(log);
                    }
                }
            }

            for trace in trace_response
                .data
                .as_mut()
                .unwrap_or(&mut vec![])
                .into_iter()
            {
                for span in trace.spans.as_mut().unwrap_or(&mut vec![]).iter_mut() {
                    match span_logs.remove(&span.span_id) {
                        Some(es_logs) => {
                            if let Some(j_logs) = &mut span.logs.as_mut() {
                                let new_logs: Vec<Log> =
                                    es_logs.into_iter().map(Into::into).collect();
                                j_logs.extend(new_logs);
                            };
                            ()
                        }
                        None => (),
                    }
                }
            }

            warn!("Unused logs {:?}", span_logs);
        }
    }

    let headers = res.headers();

    let mut resp = Response::new(StatusCode::Ok);
    headers.iter().for_each(|h| resp.append_header(h.0, h.1));
    resp.remove_header("Content-Encoding");

    resp.set_body(serde_json::to_string(&trace_response).unwrap());

    Ok(resp)
}

pub async fn serve(
    host: String,
    port: u16,
    jaeger_host: Url,
    es_host: Url,
    indexes: Vec<String>,
) -> Result<(), std::io::Error> {
    info!("Proxy request to {}", jaeger_host);
    info!("Elasticsearch host: {}", es_host);
    info!("Elasticsearch indexes: {}", &indexes.join(","));

    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .expect("Unable to parse socket address");

    let state = ApplicationState {
        jaeger_host,
        es_client: ESClient::new(
            es_host,
            indexes,
        ),
    };

    let mut app = tide::with_state(state);

    app.middleware(access_log);

    app.at("/").get(proxy_handler);
    app.at("/*").get(proxy_handler);
    app.at("/api/traces/:trace_id").get(added_log_handler);

    info!("Listen {}:{}", host, port);
    app.listen(addr).await?;

    Ok(())
}
