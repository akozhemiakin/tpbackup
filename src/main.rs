use tracing_subscriber::EnvFilter;

mod cli;
mod client;
mod utils;

#[tokio::main]
async fn main() {
    init_logger();

    cli::Cli::run().await;
}

fn init_logger() {
    let is_debug = cfg!(debug_assertions);

    tracing_subscriber::fmt()
        .json()
        .with_file(is_debug)
        .with_env_filter(EnvFilter::from_default_env())
        .with_line_number(is_debug)
        .with_writer(std::io::stderr)
        .init();
}
