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

// Simple struct that is an Service that response over HTTP with Hello, World!
#[derive(Clone, Copy)]
pub struct HelloWorld;

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
