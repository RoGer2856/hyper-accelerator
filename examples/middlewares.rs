use std::sync::Arc;

use clap::Parser;
use fn_decorator::use_decorator;
use hyper_accelerator::{
    application_context_trait::ApplicationContextTrait,
    create_request_handler_call_chain, decorators,
    error::Error,
    request_context_trait::RequestContextTrait,
    request_handler::{ErrorResponse, Request, Response},
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

#[use_decorator(decorators::debug_log_headers())]
async fn hello(
    _req: Request,
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

    let server_task = run_http1_tcp_server(
        cli.listener_address,
        create_request_handler_call_chain!(
            decorators::debug_log_headers,
            decorators::debug_log_cookies,
            hello
        ),
        ApplicationContext,
    )
    .await?;
    server_task.await??;

    Ok(())
}
