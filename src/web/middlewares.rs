use std::future::Future;
use std::pin::Pin;

use log::info;

use tide::{Next, Request, Result};

pub fn access_log<'a, State>(
    req: Request<State>,
    next: Next<'a, State>,
) -> Pin<Box<dyn Future<Output = Result> + Send + 'a>>
where
    State: Send + Sync,
    State: 'static,
{
    Box::pin(async {
        let url = req.url().clone();
        let res = next.run(req).await;

        let response_code = match &res.iter().next() {
            Some(v) => v.status().to_string(),
            None => "noop".to_string(),
        };

        info!("{} -> {}", url, response_code);
        res
    })
}
