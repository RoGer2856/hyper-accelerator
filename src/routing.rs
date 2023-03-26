use std::{future::Future, pin::Pin, sync::Arc};

use regex::Regex;

use crate::{
    application_context_trait::ApplicationContextTrait,
    body::Body,
    request_context_trait::RequestContextTrait,
    request_handler::{ErrorResponse, RequestHandlerFn},
    response::create_empty_response,
};

type RouterFnReturnType =
    Pin<Box<dyn Future<Output = Result<hyper::Response<Body>, ErrorResponse>> + Send + Sync>>;

#[allow(type_alias_bounds)]
type RouterFnType<
    ApplicationContextType: ApplicationContextTrait,
    RequestContextType: RequestContextTrait,
> = Pin<
    Box<
        dyn Fn(
                hyper::Request<hyper::body::Incoming>,
                Arc<ApplicationContextType>,
                RequestContextType,
                regex::Captures,
            ) -> RouterFnReturnType
            + Send
            + Sync,
    >,
>;

struct RoutingRecord<
    ApplicationContextType: ApplicationContextTrait,
    RequestContextType: RequestContextTrait,
> {
    methods: Vec<hyper::Method>,
    path: Regex,
    request_handler: RouterFnType<ApplicationContextType, RequestContextType>,
}

pub struct RouterBuilder<
    ApplicationContextType: ApplicationContextTrait,
    RequestContextType: RequestContextTrait,
> {
    routing_table: Vec<RoutingRecord<ApplicationContextType, RequestContextType>>,
}

pub struct Router<
    ApplicationContextType: ApplicationContextTrait,
    RequestContextType: RequestContextTrait,
> {
    app_context: Arc<ApplicationContextType>,
    routing_table: Arc<Vec<RoutingRecord<ApplicationContextType, RequestContextType>>>,
}

impl<ApplicationContextType: ApplicationContextTrait, RequestContextType: RequestContextTrait>
    Default for RouterBuilder<ApplicationContextType, RequestContextType>
{
    fn default() -> Self {
        RouterBuilder::new()
    }
}

impl<ApplicationContextType: ApplicationContextTrait, RequestContextType: RequestContextTrait>
    RouterBuilder<ApplicationContextType, RequestContextType>
{
    pub fn new() -> Self {
        Self {
            routing_table: Vec::new(),
        }
    }

    pub fn path<
        ReturnType: Future<Output = Result<hyper::Response<Body>, ErrorResponse>> + Send + Sync + 'static,
    >(
        mut self,
        methods: &[hyper::Method],
        path: impl ToString,
        request_handler: impl RequestHandlerFn<ApplicationContextType, RequestContextType, ReturnType>,
    ) -> Result<Self, regex::Error> {
        let path = "^".to_string() + &path.to_string() + "$";
        self.routing_table.push(RoutingRecord {
            methods: methods.into(),
            path: Regex::new(&path)?,
            request_handler: Box::pin(move |req, app_context, request_context, _captures| {
                Box::pin(request_handler(req, app_context, request_context))
            }),
        });

        Ok(self)
    }

    pub fn path_with_params<
        ReturnType: Future<Output = Result<hyper::Response<Body>, ErrorResponse>> + Send + Sync + 'static,
    >(
        mut self,
        methods: &[hyper::Method],
        path: impl ToString,
        request_handler: impl Fn(
                hyper::Request<hyper::body::Incoming>,
                Arc<ApplicationContextType>,
                RequestContextType,
                regex::Captures,
            ) -> ReturnType
            + Send
            + Sync
            + 'static,
    ) -> Result<Self, regex::Error> {
        let path = "^".to_string() + &path.to_string() + "$";
        self.routing_table.push(RoutingRecord {
            methods: methods.into(),
            path: Regex::new(&path)?,
            request_handler: Box::pin(move |req, app_context, request_context, captures| {
                Box::pin(request_handler(req, app_context, request_context, captures))
            }),
        });

        Ok(self)
    }

    pub fn build(
        self,
        app_context: ApplicationContextType,
    ) -> Router<ApplicationContextType, RequestContextType> {
        Router {
            app_context: Arc::new(app_context),
            routing_table: Arc::new(self.routing_table),
        }
    }
}

impl<ApplicationContextType: ApplicationContextTrait, RequestContextType: RequestContextTrait>
    Router<ApplicationContextType, RequestContextType>
{
    pub async fn dispatch(
        &self,
        req: hyper::Request<hyper::body::Incoming>,
        app_context: Arc<ApplicationContextType>,
        request_context: RequestContextType,
    ) -> Result<hyper::Response<Body>, ErrorResponse> {
        for router_record in self.routing_table.iter() {
            if router_record.methods.contains(req.method()) {
                let path = req.uri().path().to_string();
                if let Some(captures) = router_record.path.captures(&path) {
                    return router_record.request_handler.as_ref()(
                        req,
                        app_context,
                        request_context,
                        captures,
                    )
                    .await;
                }
            }
        }

        Err(create_empty_response(hyper::StatusCode::NOT_FOUND).into())
    }
}

impl<ApplicationContextType: ApplicationContextTrait, RequestContextType: RequestContextTrait>
    ApplicationContextTrait for Router<ApplicationContextType, RequestContextType>
{
}

pub async fn router_fn<
    ApplicationContextType: ApplicationContextTrait,
    RequestContextType: RequestContextTrait,
>(
    req: hyper::Request<hyper::body::Incoming>,
    router: Arc<Router<ApplicationContextType, RequestContextType>>,
    request_context: RequestContextType,
) -> Result<hyper::Response<Body>, ErrorResponse> {
    log::debug!("Request = {} {}", req.method(), req.uri().path());

    router
        .as_ref()
        .dispatch(req, router.app_context.clone(), request_context)
        .await
}
