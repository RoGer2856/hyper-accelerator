use std::sync::Arc;

use crate::{
    application_context_trait::ApplicationContextTrait,
    body::Body,
    request_context_trait::RequestContextTrait,
    request_handler::{ErrorResponse, RequestHandlerFn, RequestHandlerReturnTrait},
};

pub async fn debug_log_headers<
    ApplicationContextType: ApplicationContextTrait,
    RequestContextType: RequestContextTrait,
    NextReturnType: RequestHandlerReturnTrait,
>(
    req: hyper::Request<hyper::body::Incoming>,
    app_context: Arc<ApplicationContextType>,
    request_context: RequestContextType,
    next: impl RequestHandlerFn<ApplicationContextType, RequestContextType, NextReturnType>,
) -> Result<hyper::Response<Body>, ErrorResponse> {
    log::debug!("STARTING debug_log_headers");

    for header in req.headers().iter() {
        let header_name: &hyper::header::HeaderName = header.0;
        let header_value: &hyper::header::HeaderValue = header.1;
        match header_value.to_str() {
            Ok(header_value) => {
                log::debug!("{}: {}", header_name.as_str(), header_value);
            }
            Err(_) => {
                log::warn!("Header value cannot convert to string")
            }
        }
    }
    next(req, app_context, request_context).await
}
