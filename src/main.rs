mod cli;
mod client;
mod utils;

#[tokio::main]
async fn main() {
    cli::Cli::run().await;
}
