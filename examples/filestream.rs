#![allow(unstable_name_collisions)]

use std::sync::Arc;

use clap::Parser;
use hyper::StatusCode;
use hyper_accelerator::{
    application_context_trait::ApplicationContextTrait,
    body_utils::create_stream_body,
    content_type::ContentType,
    error::Error,
    filestream::FileStream,
    prelude::ResultInspector,
    request_context_trait::RequestContextTrait,
    request_handler::{ErrorResponse, Request, Response},
    response::{create_empty_response, create_file_response},
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

async fn file(
    _req: Request,
    _app_context: Arc<ApplicationContext>,
    _request_context: RequestContext,
) -> Result<Response, ErrorResponse> {
    let filestream = FileStream::new("examples/gandalf-quote.txt")
        .await
        .inspect_err(|e| log::error!("Could not open file, error = {:?}", e))
        .map_err(|_| create_empty_response(StatusCode::INTERNAL_SERVER_ERROR))?;

    create_file_response(
        StatusCode::OK,
        create_stream_body(filestream),
        ContentType::TextPlain,
        "gandalf-quote.txt",
    )
    .inspect_err(|e| log::error!("Could not create JSON response, error = {:?}", e))
    .map_err(|_| create_empty_response(StatusCode::INTERNAL_SERVER_ERROR).into())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    env_logger::builder()
        .filter_level(log::LevelFilter::max())
        .init();

    log::info!("Starting application!");

    let server_task = run_http1_tcp_server(cli.listener_address, file, ApplicationContext).await?;
    server_task.await??;

    Ok(())
}
