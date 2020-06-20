use std::env::var;
use std::net::SocketAddr;

use surf::url::Url;
use tide::{Request, Response, StatusCode};

use lazy_static::lazy_static;

use crate::web::middlewares::access_log;
use crate::web::models::GetTraceResponse;

lazy_static! {
    static ref JAEGER_HOST: String = var("JAEGER_HOST").expect("JAEGER_HOST must be setted");
    static ref JAEGER_PORT: u16 = var("JAEGER_PORT").unwrap_or("80".to_string()).parse().expect("JAEGER_PORT must be an integer");
}

async fn proxy_handler(req: Request<()>) -> tide::Result {

    let mut path = req.url().clone();
    path.set_host(Some(&*JAEGER_HOST))?;
    path.set_port(Some(*JAEGER_PORT)).unwrap();


    let mut res = surf::get(path).await.unwrap();
    let body = res.body_bytes().await.unwrap();
    let headers = res.headers();

    let mut resp = Response::new(StatusCode::Ok);
    headers.iter().for_each(
        |h| resp.append_header(h.0, h.1)
    );
    resp.remove_header("Content-Encoding");
    resp.set_body(body);

    Ok(resp)
}


async fn added_log_handler(req: Request<()>) -> tide::Result {
    let mut path = req.url().clone();
    path.set_host(Some(&*JAEGER_HOST))?;
    path.set_port(Some(*JAEGER_PORT)).unwrap();

    let trace_id: String = req.param("trace_id").expect("Expected trace_id parameter");


    dbg!(&trace_id);

    let mut res = surf::get(path).await.unwrap();

    let body: String = res.body_string().await.unwrap();

    println!("{:#?}", body);

    let trace_response: GetTraceResponse = serde_json::from_str(body.as_ref()).unwrap();
    println!("{:#?}", trace_response);


    let headers = res.headers();

    let mut resp = Response::new(StatusCode::Ok);
    headers.iter().for_each(
        |h| resp.append_header(h.0, h.1)
    );
    resp.remove_header("Content-Encoding");

    resp.set_body(serde_json::to_string(&trace_response).unwrap());

    Ok(resp)
}


pub async fn serve(host: String, port: u16) -> Result<(), std::io::Error> {
    let jaeger_url: Url = format!("{}:{}", *JAEGER_HOST, *JAEGER_PORT).parse().expect("Invalid JAEGER_HOST:JAEGER_PORT");
    println!("Proxy request to {}", jaeger_url);

    let addr: SocketAddr = format!("{}:{}", host, port).parse().expect("Unable to parse socket address");

    let mut app = tide::new();

    app.middleware(access_log);

    app.at("/").get(proxy_handler);
    app.at("/*").get(proxy_handler);
    app.at("/api/traces/:trace_id").get(added_log_handler);

    println!("Listen {}:{}", host, port);
    app.listen(addr).await?;

    Ok(())
}
