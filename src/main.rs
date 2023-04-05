#![allow(dead_code, warnings, unused_imports, unused_variables)]
mod logger;

use futures::future::{ready, BoxFuture, Map, Ready};
use futures::{Future, TryFutureExt};
use hyper::service::make_service_fn;
use hyper::{Body, Method, Request, Response, Server};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::pin::Pin;
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
        let svc = logger::LoggingService::new(svc);
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
