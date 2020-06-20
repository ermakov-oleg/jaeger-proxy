use std::env::var;
use std::net::SocketAddr;

use tide::{Request, Response, StatusCode};

use lazy_static::lazy_static;
use surf::url::Url;

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



pub async fn serve(host: String, port: u16) -> Result<(), std::io::Error> {
    let jaeger_url: Url = format!("{}:{}", *JAEGER_HOST, *JAEGER_PORT).parse().expect("Invalid JAEGER_HOST:JAEGER_PORT");
    println!("Proxy request to {}", jaeger_url);

    let addr: SocketAddr = format!("{}:{}", host, port).parse().expect("Unable to parse socket address");

    let mut app = tide::new();
    app.at("/").get(proxy_handler);
    app.at("/*").get(proxy_handler);

    println!("Listen {}:{}", host, port);
    app.listen(addr).await?;

    Ok(())
}
