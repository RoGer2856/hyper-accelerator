use std::{future::Future, ops::Deref, pin::Pin, sync::Arc};

use regex::Regex;

use crate::{
    application_context_trait::ApplicationContextTrait,
    request_context_trait::RequestContextTrait,
    request_handler::{ErrorResponse, Request, RequestHandlerFn, Response},
    response::create_empty_response,
};

type RouterFnReturnType =
    Pin<Box<dyn Future<Output = Result<Response, ErrorResponse>> + Send + Sync>>;

#[allow(type_alias_bounds)]
type RouterFnType<
    ApplicationContextType: ApplicationContextTrait,
    RequestContextType: RequestContextTrait<ApplicationContextType>,
> = Pin<
    Box<
        dyn Fn(
                Request,
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
    RequestContextType: RequestContextTrait<ApplicationContextType>,
> {
    methods: Vec<hyper::Method>,
    path: Regex,
    request_handler: RouterFnType<ApplicationContextType, RequestContextType>,
}

pub struct RouterBuilder<
    ApplicationContextType: ApplicationContextTrait,
    RequestContextType: RequestContextTrait<ApplicationContextType>,
> {
    routing_table: Vec<RoutingRecord<ApplicationContextType, RequestContextType>>,
}

pub struct Router<
    ApplicationContextType: ApplicationContextTrait,
    RequestContextType: RequestContextTrait<ApplicationContextType>,
> {
    app_context: Arc<ApplicationContextType>,
    routing_table: Arc<Vec<RoutingRecord<ApplicationContextType, RequestContextType>>>,
}

impl<
        ApplicationContextType: ApplicationContextTrait,
        RequestContextType: RequestContextTrait<ApplicationContextType>,
    > Deref for Router<ApplicationContextType, RequestContextType>
{
    type Target = ApplicationContextType;

    fn deref(&self) -> &Self::Target {
        &self.app_context
    }
}

impl<
        ApplicationContextType: ApplicationContextTrait,
        RequestContextType: RequestContextTrait<ApplicationContextType>,
    > Default for RouterBuilder<ApplicationContextType, RequestContextType>
{
    fn default() -> Self {
        RouterBuilder::new()
    }
}

impl<
        ApplicationContextType: ApplicationContextTrait,
        RequestContextType: RequestContextTrait<ApplicationContextType>,
    > RouterBuilder<ApplicationContextType, RequestContextType>
{
    pub fn new() -> Self {
        Self {
            routing_table: Vec::new(),
        }
    }

    pub fn path<
        ReturnType: Future<Output = Result<Response, ErrorResponse>> + Send + Sync + 'static,
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
        ReturnType: Future<Output = Result<Response, ErrorResponse>> + Send + Sync + 'static,
    >(
        mut self,
        methods: &[hyper::Method],
        path: impl ToString,
        request_handler: impl Fn(
                Request,
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

impl<
        ApplicationContextType: ApplicationContextTrait,
        RequestContextType: RequestContextTrait<ApplicationContextType>,
    > Router<ApplicationContextType, RequestContextType>
{
    pub async fn dispatch(
        &self,
        req: Request,
        app_context: Arc<ApplicationContextType>,
        request_context: RequestContextType,
    ) -> Result<Response, ErrorResponse> {
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

impl<
        ApplicationContextType: ApplicationContextTrait,
        RequestContextType: RequestContextTrait<ApplicationContextType>,
    > ApplicationContextTrait for Router<ApplicationContextType, RequestContextType>
{
}

impl<
        ApplicationContextType: ApplicationContextTrait,
        RequestContextType: RequestContextTrait<ApplicationContextType>,
    > RequestContextTrait<Router<ApplicationContextType, RequestContextType>>
    for RequestContextType
{
    fn create(app_context: Arc<Router<ApplicationContextType, RequestContextType>>) -> Self {
        Self::create(app_context.app_context.clone())
    }
}

pub async fn router_fn<
    ApplicationContextType: ApplicationContextTrait,
    RequestContextType: RequestContextTrait<ApplicationContextType>,
>(
    req: Request,
    router: Arc<Router<ApplicationContextType, RequestContextType>>,
    request_context: RequestContextType,
) -> Result<Response, ErrorResponse> {
    router
        .as_ref()
        .dispatch(req, router.app_context.clone(), request_context)
        .await
}
