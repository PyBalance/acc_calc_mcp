mod cli;
mod error;
mod handler;
mod server;
mod tools;

use clap::Parser;
use error::ServiceResult;

#[tokio::main]
async fn main() -> ServiceResult<()> {
    server::start_server(cli::CommandArguments::parse()).await
}
