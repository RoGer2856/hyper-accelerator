#![allow(unstable_name_collisions)]

use std::sync::Arc;

use clap::Parser;
use hyper::StatusCode;
use hyper_accelerator::{
    application_context_trait::ApplicationContextTrait,
    create_request_handler_call_chain,
    decorators::{debug_log_cookies, debug_log_headers, debug_log_request_line},
    error::Error,
    prelude::ResultInspector,
    request_context_trait::RequestContextTrait,
    request_handler::{ErrorResponse, Request, Response},
    response::{create_empty_response, create_json_response},
    routing::{router_fn, RouterBuilder},
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

async fn index(
    _req: Request,
    _app_context: Arc<ApplicationContext>,
    _request_context: RequestContext,
) -> Result<Response, ErrorResponse> {
    let response_body = "/ -> this help message\n\
        /hello -> sends hello back\n\
        /echo?<query_params> -> echoes every query param back\n\
        /resources/<resource-id> -> queries the resource with the given id";
    Ok(Response::new(response_body.into()))
}

async fn hello(
    _req: Request,
    _app_context: Arc<ApplicationContext>,
    _request_context: RequestContext,
) -> Result<Response, ErrorResponse> {
    Ok(Response::new("Hello World!".into()))
}

async fn echo(
    req: Request,
    _app_context: Arc<ApplicationContext>,
    _request_context: RequestContext,
) -> Result<Response, ErrorResponse> {
    let mut response_body = String::new();

    for param in querystring::querify(req.uri().query().unwrap_or("")) {
        let name = param.0;
        let value = param.1;

        response_body += name;
        response_body += ": ";
        response_body += value;
        response_body += "\n";
    }

    Ok(Response::new(response_body.into()))
}

#[derive(serde::Serialize)]
struct Resource {
    id: String,
}

async fn resource_by_id(
    _req: Request,
    _app_context: Arc<ApplicationContext>,
    _request_context: RequestContext,
    id: String,
) -> Result<Response, ErrorResponse> {
    Ok(create_json_response(StatusCode::OK, &Resource { id })
        .inspect_err(|e| log::error!("Could not create JSON response, error = {:?}", e))
        .map_err(|_| create_empty_response(StatusCode::INTERNAL_SERVER_ERROR))?)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    env_logger::builder()
        .filter_level(log::LevelFilter::max())
        .init();

    log::info!("Starting application!");

    let router = RouterBuilder::new()
        .path(&[hyper::Method::GET], r"/", index)?
        .path(&[hyper::Method::GET], r"/hello", hello)?
        .path(&[hyper::Method::GET], r"/echo", echo)?
        .path_with_params(
            &[hyper::Method::GET],
            r"/resources/(\w+)",
            |req, app_context, request_context, path_captures| {
                resource_by_id(
                    req,
                    app_context,
                    request_context,
                    path_captures[1].to_string(),
                )
            },
        )?
        .build(ApplicationContext);

    let server_task = run_http1_tcp_server(
        cli.listener_address,
        create_request_handler_call_chain!(
            debug_log_request_line,
            debug_log_headers,
            debug_log_cookies,
            router_fn
        ),
        router,
    )
    .await?;
    server_task.await??;

    Ok(())
}
