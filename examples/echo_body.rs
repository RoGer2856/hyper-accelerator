#![allow(unstable_name_collisions)]

use std::sync::Arc;

use clap::Parser;
use hyper_accelerator::{
    application_context_trait::ApplicationContextTrait,
    body_ext::BodyExt,
    content_type::ContentType,
    error::Error,
    prelude::ResultInspector,
    request_context_trait::RequestContextTrait,
    request_handler::{ErrorResponse, Request, Response},
    response::{create_bytes_response, create_empty_response},
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

struct RequestContext {
    _app_context: Arc<ApplicationContext>,
}

impl RequestContextTrait<ApplicationContext> for RequestContext {
    fn create(app_context: Arc<ApplicationContext>) -> Self {
        Self {
            _app_context: app_context,
        }
    }
}

#[derive(serde::Serialize)]
struct SerializableResponse {
    message: String,
}

async fn echo_body(
    req: Request,
    _app_context: Arc<ApplicationContext>,
    _request_context: RequestContext,
) -> Result<Response, ErrorResponse> {
    let (parts, body) = req.into_parts();

    let content_type = parts
        .headers
        .get("content-type")
        .cloned()
        .unwrap_or_else(|| ContentType::ApplicationOctetstream.into());

    let (payload, _trailers) = body
        .collect()
        .await
        .inspect_err(|e| log::warn!("Error while reading payload, error = {e:?}"))
        .map_err(|_| create_empty_response(hyper::StatusCode::INTERNAL_SERVER_ERROR))?
        .aggregate();
    let payload = payload.ok_or_else(|| create_empty_response(hyper::StatusCode::BAD_REQUEST))?;

    Ok(create_bytes_response(
        hyper::StatusCode::OK,
        payload.as_ref(),
        content_type,
    ))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    env_logger::builder()
        .filter_level(log::LevelFilter::max())
        .init();

    log::info!("Starting application!");

    let server_task =
        run_http1_tcp_server(cli.listener_address, echo_body, ApplicationContext).await?;
    server_task.await??;

    Ok(())
}
