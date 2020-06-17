#![warn(rust_2018_idioms)]

use structopt::StructOpt;

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

fn main() {
    let opt = ApplicationArguments::from_args();

    match opt.command {
        Command::Serve(params) => println!("{:?}", params),
    }
}
