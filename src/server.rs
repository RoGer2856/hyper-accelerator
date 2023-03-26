use std::{future::Future, io, sync::Arc};

use hyper::{server::conn::http1, service::service_fn};
use tokio::{
    net::{TcpListener, ToSocketAddrs},
    task::JoinHandle,
};

use crate::{
    application_context_trait::ApplicationContextTrait,
    body::Body,
    error::Error,
    request_context_trait::RequestContextTrait,
    request_handler::{ErrorResponse, RequestHandlerFn},
};

pub async fn run_http1_tcp_server<
    SocketAddressType: ToSocketAddrs,
    ApplicationContextType: ApplicationContextTrait,
    RequestContextType: RequestContextTrait,
    ReturnType: Future<Output = Result<hyper::Response<Body>, ErrorResponse>> + Send + Sync + 'static,
    RequestHandlerFnType: RequestHandlerFn<ApplicationContextType, RequestContextType, ReturnType>,
>(
    listener_address: SocketAddressType,
    request_handler: RequestHandlerFnType,
    app_context: ApplicationContextType,
) -> Result<JoinHandle<Result<(), io::Error>>, io::Error> {
    let listener = TcpListener::bind(listener_address).await?;

    Ok(tokio::spawn(async move {
        let request_handler = Arc::new(request_handler);
        let application_context = Arc::new(app_context);

        loop {
            let (stream, _) = listener.accept().await?;

            let request_handler = request_handler.clone();
            let application_context = application_context.clone();

            tokio::task::spawn(async move {
                let service = service_fn(move |req: hyper::Request<hyper::body::Incoming>| {
                    service_helper(request_handler(
                        req,
                        application_context.clone(),
                        RequestContextType::default(),
                    ))
                });

                if let Err(err) = http1::Builder::new()
                    .serve_connection(stream, service)
                    .await
                {
                    println!("Error serving connection: {:?}", err);
                }
            });
        }
    }))
}

async fn service_helper(
    request_handler_task: impl Future<Output = Result<hyper::Response<Body>, ErrorResponse>>,
) -> Result<hyper::Response<Body>, Error> {
    match request_handler_task.await {
        Ok(resp) => Ok(resp),
        Err(resp) => Ok(resp.0),
    }
}
