use std::sync::Arc;

use crate::{
    application_context_trait::ApplicationContextTrait,
    body::Body,
    cookies::{cookies_iter, CookieType},
    request_context_trait::RequestContextTrait,
    request_handler::{ErrorResponse, RequestHandlerFn, RequestHandlerReturnTrait},
};

pub async fn debug_log_cookies<
    ApplicationContextType: ApplicationContextTrait,
    RequestContextType: RequestContextTrait,
    NextReturnType: RequestHandlerReturnTrait,
>(
    req: hyper::Request<hyper::body::Incoming>,
    app_context: Arc<ApplicationContextType>,
    request_context: RequestContextType,
    next: impl RequestHandlerFn<ApplicationContextType, RequestContextType, NextReturnType>,
) -> Result<hyper::Response<Body>, ErrorResponse> {
    log::debug!("STARTING debug_log_cookies");

    for cookie in cookies_iter(CookieType::Cookie, req.headers()) {
        let cookie: cookie::Cookie = cookie;
        log::debug!("{} {}", cookie.name(), cookie.value());
    }

    next(req, app_context, request_context).await
}
