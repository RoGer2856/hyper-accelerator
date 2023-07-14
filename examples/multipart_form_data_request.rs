#![allow(unstable_name_collisions)]

use std::{io::Read, sync::Arc};

use clap::Parser;
use hyper::body::Buf;
use hyper_accelerator::{
    application_context_trait::ApplicationContextTrait,
    body_ext::BodyExt,
    content_type::ContentType,
    error::Error,
    prelude::ResultInspector,
    request_context_trait::RequestContextTrait,
    request_handler::{ErrorResponse, Request, Response},
    response::{create_empty_response, create_static_str_response},
    server::run_http1_tcp_server,
};
use regex::Regex;

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

macro_rules! create_invalid_content_type_response {
    () => {
        create_static_str_response(
            hyper::StatusCode::BAD_REQUEST,
            "content-type is not multipart form data",
            ContentType::TextPlain,
        )
    };
}

async fn handle_request(
    req: Request,
    _app_context: Arc<ApplicationContext>,
    _request_context: RequestContext,
) -> Result<Response, ErrorResponse> {
    let (parts, body) = req.into_parts();

    let content_type = parts
        .headers
        .get("content-type")
        .ok_or_else(|| create_invalid_content_type_response!())?
        .to_str()
        .map_err(|_| create_invalid_content_type_response!())?;

    let multipart_form_data_regex = Regex::new("^multipart/form-data; *boundary=([\\w\\-]*)$")
        .inspect_err(|e| log::warn!("Error compiling regex, error = {e}"))
        .map_err(|_| create_empty_response(hyper::StatusCode::INTERNAL_SERVER_ERROR))?;
    let captures = multipart_form_data_regex
        .captures(content_type)
        .ok_or_else(|| create_invalid_content_type_response!())?;

    let boundary = match captures.get(1) {
        Some(capture) => capture.as_str(),
        None => unreachable!(),
    };

    let mut frame_iter = body.frame_iter();
    while let Some(frame) = frame_iter.next_frame().await {
        match frame {
            Ok(frame) => {
                if frame.is_data() {
                    let data = match frame.into_data() {
                        Ok(data) => data,
                        Err(_) => unreachable!(),
                    };

                    let mut multipart =
                        multipart::server::Multipart::with_body(data.reader(), boundary);
                    while let Some(mut entry) = multipart.read_entry().map_err(|_| {
                        create_empty_response(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                    })? {
                        let mut buf = Vec::new();
                        entry.data.read_to_end(&mut buf).map_err(|_| {
                            create_empty_response(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                        })?;
                        let buf_len = buf.len();
                        if let Ok(text) = String::from_utf8(buf) {
                            log::info!("entry, text = {}", text);
                        } else {
                            log::info!("binary entry, length = {}", buf_len);
                        }
                    }
                }
            }
            Err(_) => Err(create_empty_response(
                hyper::StatusCode::INTERNAL_SERVER_ERROR,
            ))?,
        }
    }

    Ok(create_empty_response(hyper::StatusCode::OK))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    env_logger::builder()
        .filter_module("multipart_form_data_request", log::LevelFilter::max())
        .init();

    log::info!("Starting application!");

    let server_task =
        run_http1_tcp_server(cli.listener_address, handle_request, ApplicationContext).await?;
    server_task.await??;

    Ok(())
}
