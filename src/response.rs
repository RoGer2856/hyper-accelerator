use hyper::{header::InvalidHeaderValue, http::HeaderValue};

use crate::{
    body::{AsyncStream, Body},
    body_utils::{
        create_bytes_body, create_json_body, create_static_str_body, create_stream_body,
        create_string_body, SerializeToJsonBodyError,
    },
};

pub fn create_empty_response(status_code: hyper::StatusCode) -> hyper::Response<Body> {
    let mut response = hyper::Response::default();
    *response.status_mut() = status_code;
    response
}

pub fn create_json_response<T: serde::Serialize>(
    status: hyper::StatusCode,
    data: &T,
) -> Result<hyper::Response<Body>, SerializeToJsonBodyError> {
    let mut resp = hyper::Response::default();
    *resp.status_mut() = status;

    resp.headers_mut()
        .insert("Content-Type", HeaderValue::from_static("application/json"));

    *resp.body_mut() = create_json_body(data)?;

    Ok(resp)
}

pub fn create_static_str_response(
    status: hyper::StatusCode,
    text: &'static str,
    content_type: impl Into<&'static str>,
) -> hyper::Response<Body> {
    let mut resp = hyper::Response::default();
    *resp.status_mut() = status;

    resp.headers_mut().insert(
        "Content-Type",
        HeaderValue::from_static(content_type.into()),
    );

    *resp.body_mut() = create_static_str_body(text);

    resp
}

pub fn create_string_response(
    status: hyper::StatusCode,
    text: impl ToString,
    content_type: impl Into<&'static str>,
) -> hyper::Response<Body> {
    let mut resp = hyper::Response::default();
    *resp.status_mut() = status;

    resp.headers_mut().insert(
        "Content-Type",
        HeaderValue::from_static(content_type.into()),
    );

    *resp.body_mut() = create_string_body(text);

    resp
}

pub fn create_bytes_response(
    status: hyper::StatusCode,
    bytes: &[u8],
    content_type: impl Into<&'static str>,
) -> hyper::Response<Body> {
    let mut resp = hyper::Response::default();
    *resp.status_mut() = status;

    resp.headers_mut().insert(
        "Content-Type",
        HeaderValue::from_static(content_type.into()),
    );

    *resp.body_mut() = create_bytes_body(bytes);

    resp
}

pub fn create_stream_response(
    status: hyper::StatusCode,
    stream: impl AsyncStream<Vec<u8>>,
    content_type: impl Into<&'static str>,
) -> hyper::Response<Body> {
    let mut resp = hyper::Response::default();
    *resp.status_mut() = status;

    resp.headers_mut().insert(
        "Content-Type",
        HeaderValue::from_static(content_type.into()),
    );

    *resp.body_mut() = create_stream_body(stream);

    resp
}

pub fn create_file_response(
    status: hyper::StatusCode,
    body: Body,
    content_type: impl Into<&'static str>,
    filename: &str,
) -> Result<hyper::Response<Body>, InvalidHeaderValue> {
    let mut resp = hyper::Response::default();
    *resp.status_mut() = status;

    resp.headers_mut().insert(
        "Content-Type",
        HeaderValue::from_static(content_type.into()),
    );
    resp.headers_mut().insert(
        "Content-Disposition",
        HeaderValue::from_str(&format!("attachment; filename={filename}"))?,
    );

    *resp.body_mut() = body;

    Ok(resp)
}
