use std::sync::Arc;

use clap::Parser;
use hyper_accelerator::{
    application_context_trait::ApplicationContextTrait, error::Error,
    request_context_trait::RequestContextTrait, request_handler::{ErrorResponse, Response},
    server::run_http1_tcp_server,
};

#[derive(Parser)]
#[command()]
pub struct Cli {
    #[arg(
        short('l'),
        long("listener-address"),
        help("Address where the server accepts the connections (e.g., 127.0.0.1:80)")
    )]
    listener_address: String,
}

struct ApplicationContext;

impl ApplicationContextTrait for ApplicationContext {}

#[derive(Default)]
struct RequestContext;

impl RequestContextTrait for RequestContext {}

async fn hello(
    _req: hyper::Request<hyper::body::Incoming>,
    _app_context: Arc<ApplicationContext>,
    _request_context: RequestContext,
) -> Result<Response, ErrorResponse> {
    Ok(Response::new("Hello World!".into()))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    env_logger::builder()
        .filter_level(log::LevelFilter::max())
        .init();

    log::info!("Starting application!");

    let server_task = run_http1_tcp_server(cli.listener_address, hello, ApplicationContext).await?;
    server_task.await??;

    Ok(())
}
