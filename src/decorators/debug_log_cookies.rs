use std::sync::Arc;

use crate::{
    application_context_trait::ApplicationContextTrait,
    cookies::cookies_iter,
    request_context_trait::RequestContextTrait,
    request_handler::{
        ErrorResponse, Request, RequestHandlerFn, RequestHandlerReturnTrait, Response,
    },
};

pub async fn debug_log_cookies<
    ApplicationContextType: ApplicationContextTrait,
    RequestContextType: RequestContextTrait<ApplicationContextType>,
    NextReturnType: RequestHandlerReturnTrait,
>(
    next: impl RequestHandlerFn<ApplicationContextType, RequestContextType, NextReturnType>,
    req: Request,
    app_context: Arc<ApplicationContextType>,
    request_context: RequestContextType,
) -> Result<Response, ErrorResponse> {
    log::debug!("STARTING debug_log_cookies");

    for cookie in cookies_iter(req.headers()) {
        match cookie {
            Ok(cookie) => log::debug!("{} {}", cookie.name(), cookie.value()),
            Err(e) => log::debug!("Cookie parse error = {e:?}"),
        }
    }

    next(req, app_context, request_context).await
}
