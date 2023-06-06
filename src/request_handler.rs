use std::{future::Future, sync::Arc};

use crate::{
    application_context_trait::ApplicationContextTrait, body::Body,
    request_context_trait::RequestContextTrait,
};

pub type Response = hyper::Response<Body>;
pub struct ErrorResponse(pub Response);

impl<T> From<T> for ErrorResponse
where
    T: Into<Response>,
{
    fn from(e: T) -> Self {
        Self(e.into())
    }
}

pub trait RequestHandlerReturnTrait:
    Future<Output = Result<Response, ErrorResponse>> + Send + Sync + 'static
{
}

impl<T: Future<Output = Result<Response, ErrorResponse>> + Send + Sync + 'static>
    RequestHandlerReturnTrait for T
{
}

pub trait RequestHandlerFn<
    ApplicationContextType: ApplicationContextTrait,
    RequestContextType: RequestContextTrait,
    ReturnType: Future<Output = Result<Response, ErrorResponse>> + Send + Sync + 'static,
>:
    Fn(
        hyper::Request<hyper::body::Incoming>,
        Arc<ApplicationContextType>,
        RequestContextType,
    ) -> ReturnType
    + Send
    + Sync
    + 'static
{
}

impl<
        ApplicationContextType: ApplicationContextTrait,
        RequestContextType: RequestContextTrait,
        ReturnType: Future<Output = Result<Response, ErrorResponse>> + Send + Sync + 'static,
        T: Fn(
                hyper::Request<hyper::body::Incoming>,
                Arc<ApplicationContextType>,
                RequestContextType,
            ) -> ReturnType
            + Send
            + Sync
            + 'static,
    > RequestHandlerFn<ApplicationContextType, RequestContextType, ReturnType> for T
{
}
