use std::sync::Arc;

use crate::{
    application_context_trait::ApplicationContextTrait,
    request_context_trait::RequestContextTrait,
    request_handler::{
        ErrorResponse, Request, RequestHandlerFn, RequestHandlerReturnTrait, Response,
    },
};

pub async fn debug_log_request_line<
    ApplicationContextType: ApplicationContextTrait,
    RequestContextType: RequestContextTrait<ApplicationContextType>,
    NextReturnType: RequestHandlerReturnTrait,
>(
    next: impl RequestHandlerFn<ApplicationContextType, RequestContextType, NextReturnType>,
    req: Request,
    app_context: Arc<ApplicationContextType>,
    request_context: RequestContextType,
) -> Result<Response, ErrorResponse> {
    log::debug!("Request line = {} {}", req.method(), req.uri());

    next(req, app_context, request_context).await
}
