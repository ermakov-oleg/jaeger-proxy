#![warn(rust_2018_idioms)]

use env_logger;
use structopt::StructOpt;
use surf::url::Url;

use jaeger_proxy::web::serve;

fn setup_logger() {
    let logger = env_logger::builder().build();
    let level = logger.filter().clone();
    async_log::Logger::wrap(logger, || /* get the task id here */ 0)
        .start(level)
        .unwrap();
}

#[derive(Debug, StructOpt)]
struct Serve {
    /// Activate debug mode
    #[structopt(short, long)]
    debug: bool,

    /// Run on host
    #[structopt(short, long, default_value = "127.0.0.1")]
    host: String,

    /// Listen port
    #[structopt(short, long, default_value = "8000")]
    port: u16,

    /// Jaeger http://host:port
    #[structopt(short, long, env = "JAEGER_HOST")]
    jaeger_host: Url,

    /// Elasticsearch http://host:port
    #[structopt(short, long, env = "ES_HOST")]
    es_host: Url,

    /// Elasticsearch indexes for search traces
    #[structopt(short, long)]
    indexes: Vec<String>,
}

#[derive(Debug, StructOpt)]
enum Command {
    #[structopt(name = "serve")]
    Serve(Serve),
}

#[derive(Debug, StructOpt)]
#[structopt(name = "classify")]
struct ApplicationArguments {
    #[structopt(subcommand)]
    command: Command,
}

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    setup_logger();
    let opt = ApplicationArguments::from_args();

    match opt.command {
        Command::Serve(params) => {
            serve(
                params.host,
                params.port,
                params.jaeger_host,
                params.es_host,
                params.indexes,
            )
            .await?
        }
    };

    Ok(())
}
