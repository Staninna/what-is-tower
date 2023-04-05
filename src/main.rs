#![allow(dead_code, warnings, unused_imports, unused_variables)]
mod hello_world;
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
        let svc = hello_world::HelloWorld;
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
