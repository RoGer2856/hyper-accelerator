use std::sync::{atomic::AtomicBool, Arc};

use crate::{
    application_context_trait::ApplicationContextTrait,
    create_request_handler_call_chain,
    request_context_trait::RequestContextTrait,
    request_handler::{
        ErrorResponse, Request, RequestHandlerFn, RequestHandlerReturnTrait, Response,
    },
    response_body::ResponseBody,
    server::run_http1_tcp_server,
};

struct TestApplicationContext;

impl ApplicationContextTrait for TestApplicationContext {}

#[derive(Default)]
struct TestRequestContext {
    middleware_called: Arc<AtomicBool>,
}

impl RequestContextTrait for TestRequestContext {}

async fn test_request_handler(
    _req: Request,
    _app_context: Arc<TestApplicationContext>,
    _request_context: TestRequestContext,
) -> Result<Response, ErrorResponse> {
    Ok(Response::new("test_request_handler".into()))
}

trait TestMiddlewareTrait {
    fn set_called(&mut self);
}

impl TestMiddlewareTrait for TestRequestContext {
    fn set_called(&mut self) {
        self.middleware_called
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

async fn test_middleware<
    ApplicationContextType: ApplicationContextTrait,
    RequestContextType: RequestContextTrait + TestMiddlewareTrait,
    NextReturnType: RequestHandlerReturnTrait,
>(
    req: Request,
    app_context: Arc<ApplicationContextType>,
    mut request_context: RequestContextType,
    next: impl RequestHandlerFn<ApplicationContextType, RequestContextType, NextReturnType>,
) -> Result<Response, ErrorResponse> {
    request_context.set_called();
    match next(req, app_context, request_context).await {
        Ok(mut response) => {
            let mut payload = response.body_mut().read_all().await.unwrap();
            let mut new_payload = Vec::from("test_middleware.".as_bytes());
            new_payload.append(&mut payload);
            *response.body_mut() = ResponseBody::from(new_payload);
            Ok(response)
        }
        Err(response) => Err(response),
    }
}

#[tokio::test]
#[serial_test::serial]
async fn aborting_server() {
    let server_task = run_http1_tcp_server(
        ("127.0.0.1", 30000),
        test_request_handler,
        TestApplicationContext,
    )
    .await
    .unwrap();

    server_task.abort();
}

#[tokio::test]
#[serial_test::serial]
async fn request_handler_called() {
    let server_task = run_http1_tcp_server(
        ("127.0.0.1", 30000),
        test_request_handler,
        TestApplicationContext,
    )
    .await
    .unwrap();

    assert_eq!(
        reqwest::get("http://localhost:30000")
            .await
            .unwrap()
            .text()
            .await
            .unwrap()
            .as_str(),
        "test_request_handler"
    );

    server_task.abort();
}

#[tokio::test]
#[serial_test::serial]
async fn middleware_called() {
    let server_task = run_http1_tcp_server(
        ("127.0.0.1", 30000),
        create_request_handler_call_chain!(test_middleware, test_request_handler),
        TestApplicationContext,
    )
    .await
    .unwrap();

    assert_eq!(
        reqwest::get("http://localhost:30000")
            .await
            .unwrap()
            .text()
            .await
            .unwrap()
            .as_str(),
        "test_middleware.test_request_handler"
    );

    server_task.abort();
}
