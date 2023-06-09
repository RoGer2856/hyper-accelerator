use hyper::{header::InvalidHeaderValue, http::HeaderValue};

use crate::{
    body_utils::{
        create_bytes_body, create_json_body, create_static_str_body, create_stream_body,
        create_string_body, SerializeToJsonBodyError,
    },
    request_handler::Response,
    response_body::{AsyncStream, ResponseBody},
};

pub fn create_empty_response(status_code: hyper::StatusCode) -> Response {
    let mut response = Response::default();
    *response.status_mut() = status_code;
    response
}

pub fn create_json_response<T: serde::Serialize>(
    status: hyper::StatusCode,
    data: &T,
) -> Result<Response, SerializeToJsonBodyError> {
    let mut resp = Response::default();
    *resp.status_mut() = status;

    resp.headers_mut()
        .insert("Content-Type", HeaderValue::from_static("application/json"));

    *resp.body_mut() = create_json_body(data)?;

    Ok(resp)
}

pub fn create_static_str_response(
    status: hyper::StatusCode,
    text: &'static str,
    content_type: impl Into<HeaderValue>,
) -> Response {
    let mut resp = Response::default();
    *resp.status_mut() = status;

    resp.headers_mut()
        .insert("Content-Type", content_type.into());

    *resp.body_mut() = create_static_str_body(text);

    resp
}

pub fn create_string_response(
    status: hyper::StatusCode,
    text: impl ToString,
    content_type: impl Into<HeaderValue>,
) -> Response {
    let mut resp = Response::default();
    *resp.status_mut() = status;

    resp.headers_mut()
        .insert("Content-Type", content_type.into());

    *resp.body_mut() = create_string_body(text);

    resp
}

pub fn create_bytes_response(
    status: hyper::StatusCode,
    bytes: &[u8],
    content_type: impl Into<HeaderValue>,
) -> Response {
    let mut resp = Response::default();
    *resp.status_mut() = status;

    resp.headers_mut()
        .insert("Content-Type", content_type.into());

    *resp.body_mut() = create_bytes_body(bytes);

    resp
}

pub fn create_stream_response(
    status: hyper::StatusCode,
    stream: impl AsyncStream<Vec<u8>>,
    content_type: impl Into<HeaderValue>,
) -> Response {
    let mut resp = Response::default();
    *resp.status_mut() = status;

    resp.headers_mut()
        .insert("Content-Type", content_type.into());

    *resp.body_mut() = create_stream_body(stream);

    resp
}

pub fn create_file_response(
    status: hyper::StatusCode,
    body: ResponseBody,
    content_type: impl Into<HeaderValue>,
    filename: &str,
) -> Result<Response, InvalidHeaderValue> {
    let mut resp = Response::default();
    *resp.status_mut() = status;

    resp.headers_mut()
        .insert("Content-Type", content_type.into());
    resp.headers_mut().insert(
        "Content-Disposition",
        HeaderValue::from_str(&format!("attachment; filename={filename}"))?,
    );

    *resp.body_mut() = body;

    Ok(resp)
}
