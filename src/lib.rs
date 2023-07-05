#![allow(unstable_name_collisions)]

pub mod app_loop_state;
pub mod application_context_trait;
pub mod body_ext;
pub mod body_utils;
pub mod content_type;
pub mod cookies;
pub mod decorators;
pub mod error;
pub mod filestream;
pub mod jwt_manager;
pub mod prelude;
pub mod request_context_trait;
pub mod request_handler;
pub mod response;
pub mod response_body;
pub mod routing;
pub mod server;

#[cfg(test)]
mod tests;

#[macro_export]
macro_rules! create_request_handler_call_chain {
    ($request_handler:path $(,)?) => {
        // println!("|req, app_context, req_context| async move {{");
        // println!("{}(req, app_context, req_context).await", stringify!($request_handler));
        // println!("}}");
        |req, app_context, req_context| async move {
            $request_handler(req, app_context, req_context).await
        }
    };
    ($decorator0:path, $decorator1:path $(,)?) => {
        // println!("|req, app_context, req_context| async move {{");
        // println!("{}(req, app_context, req_context, {}).await", stringify!($decorator0), stringify!($decorator1));
        // println!("}}");
        |req, app_context, req_context| async move {
            $decorator0($decorator1, req, app_context, req_context).await
        }
    };
    ($decorator0:path, $decorator1:path $(, $decorators:path)+ $(,)?) => {
        // println!("|req, app_context, req_context| async move {{");
        // println!("{}(req, app_context, req_context, ", stringify!($decorator0));
        // create_request_handler_call_chain!($decorator1 $(, $decorators)*);
        // println!(").await");
        // println!("}}");
        |req, app_context, req_context| async move {
            $decorator0(create_request_handler_call_chain!($decorator1 $(, $decorators)*), req, app_context, req_context).await
        }
    };
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use crate::{
        application_context_trait::ApplicationContextTrait,
        request_context_trait::RequestContextTrait,
        request_handler::{
            ErrorResponse, Request, RequestHandlerFn, RequestHandlerReturnTrait, Response,
        },
    };

    use super::*;

    struct ApplicationContext;

    impl ApplicationContextTrait for ApplicationContext {}

    #[derive(Default)]
    struct RequestContext;

    impl RequestContextTrait for RequestContext {}

    async fn request_handler(
        _req: Request,
        _app_context: Arc<ApplicationContext>,
        _req_context: RequestContext,
    ) -> Result<Response, ErrorResponse> {
        Ok(Response::new("".into()))
    }

    async fn dummy_decorator<
        ApplicationContextType: ApplicationContextTrait,
        RequestContextType: RequestContextTrait,
        NextReturnType: RequestHandlerReturnTrait,
    >(
        next: impl RequestHandlerFn<ApplicationContextType, RequestContextType, NextReturnType>,
        req: Request,
        app_context: Arc<ApplicationContextType>,
        req_context: RequestContextType,
    ) -> Result<Response, ErrorResponse> {
        return next(req, app_context, req_context).await;
    }

    #[test]
    #[allow(unused_must_use)]
    fn syntax_correctness() {
        create_request_handler_call_chain!(request_handler);
        create_request_handler_call_chain!(request_handler,);

        create_request_handler_call_chain!(dummy_decorator, request_handler,);

        create_request_handler_call_chain!(dummy_decorator, dummy_decorator, request_handler);

        create_request_handler_call_chain!(
            dummy_decorator,
            dummy_decorator,
            dummy_decorator,
            request_handler,
        );
    }
}
