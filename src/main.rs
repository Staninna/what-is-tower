#![allow(dead_code, warnings, unused_imports, unused_variables)]
use futures::future::{ready, BoxFuture, Ready};
use hyper::service::make_service_fn;
use hyper::{Body, Request, Response, Server};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::task::{Context, Poll};
use std::time::Instant;
use tower::Service;

// Function service ------------------------------------------------

// async fn handle(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
//     Ok(Response::new(Body::from("Hello World")))
// }

// Main -----------------------------------------------------------

#[tokio::main]
async fn main() {
    // Set up logging
    env_logger::init();

    // Construct our SocketAddr to listen on
    let addr = SocketAddr::from(([127, 0, 0, 1], 3678));

    // And a MakeService to handle each connection
    let make_service = make_service_fn(|_conn| async {
        let svc = HelloWorld;
        let svc = LoggingService::new(svc);
        Ok::<_, Infallible>(svc)
    });

    // Then bind and serve
    let server = Server::bind(&addr).serve(make_service);

    // And run forever
    println!("Listening on http://{}", addr);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

// Hello World Service ---------------------------------------------

// Simple struct that is an Service that response over HTTP with Hello, World!
#[derive(Clone, Copy)]
struct HelloWorld;

// Implement the service trait with the request type an HTTP request
impl Service<Request<Body>> for HelloWorld {
    // Response type is an HTTP response
    type Response = Response<Body>;

    // This service cannot fail
    type Error = Infallible;

    // It is instant ready (must be an future)
    // Ready is an Future that isnt an future but is (makes sense?)
    type Future = Ready<Result<Self::Response, Self::Error>>;

    // BoxeFuture
    // Boxed future is an future that is boxed this comes with an performance cost but is more flexible
    // type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    // Ask serves OK to reply (this cause always yes)
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    // Respond to aqual request
    fn call(&mut self, _req: Request<Body>) -> Self::Future {
        // Ready future
        // Construct an Ready future
        ready(Ok(Response::new(Body::from("Hello, World!"))))

        // Boxed future
        // Construct an Boxed future
        // Box::pin(async { Ok(Response::new(Body::from("Hello, World!"))) })
    }
}

// Logging Service ------------------------------------------------

// Simple struct that looks at the request and response and logs them
#[derive(Clone, Copy)]
struct LoggingService<S> {
    inner: S,
}

impl<S> LoggingService<S> {
    fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<S, B> Service<Request<B>> for LoggingService<S>
where
    S: Service<Request<B>> + Clone + Send + 'static,
    B: Send + 'static,
    S::Future: Send,
{
    type Response = S::Response; // Same as S: Service this case inner
    type Error = S::Error; // Same as S: Service this case inner

    // type Future = S::Future;

    // BoxeFuture
    // Boxed future is an future that is boxed this comes with an performance cost but is more flexible
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        // Get info for logging
        let method = req.method().to_owned();
        let path = req.uri().path().to_string();

        // Clone inner
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Log the reqest
            log::info!("request  {} {}", method, path);

            // Starts an timer to time request
            let start = Instant::now();

            // Get an response
            let res = inner.call(req).await;

            // Log the response
            let time_spend = start.elapsed();
            log::info!("response {} {} time={:?}", method, path, time_spend);

            // Return the response
            res
        })
    }
}

// TODO: https://youtu.be/16sU1q8OeeI?t=2906
