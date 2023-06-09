// TODO: Include status code in log (https://youtu.be/16sU1q8OeeI?t=7038)
// TODO: But first comment everything again so i know i understand it

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

// Simple struct that looks at the request and response and logs them
#[derive(Clone, Copy)]
pub struct LoggingService<S> {
    inner: S,
}

impl<S> LoggingService<S> {
    pub fn new(inner: S) -> Self {
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
    // type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    // Custom/Own future
    type Future = LoggingFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        // Get info for logging
        let method = req.method().to_owned();
        let path = req.uri().path().to_string();

        // Log the request
        log::info!("request  {} {}", method, path);

        // Starts an timer to time request
        let start = Instant::now();

        // Create fututre
        let future = self.inner.call(req);

        LoggingFuture {
            future,
            start,
            method,
            path,
        }

        // // Clone inner
        // let mut inner = self.inner.clone();

        // Box::pin(async move {
        //     // Log the reqest
        //     log::info!("request  {} {}", method, path);

        //     // Starts an timer to time request
        //     let start = Instant::now();

        //     // Get an response
        //     let res = inner.call(req).await;

        //     // Log the response
        //     let time_spend = start.elapsed();
        //     log::info!("response {} {} time={:?}", method, path, time_spend);

        //     // Return the response
        //     res
        // })
    }
}

#[pin_project::pin_project] // Impl this when building an future wrapping an other future
pub struct LoggingFuture<F> {
    #[pin] // Pin the wrapped future
    future: F,

    // For logging purposes
    start: Instant,
    method: Method,
    path: String,
}

// F == Wrapped Future
impl<F> Future for LoggingFuture<F>
where
    F: Future,
{
    type Output = F::Output; // Doesnt mutate so outputs the same

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        let result = match this.future.poll(cx) {
            Poll::Ready(response) => response,
            Poll::Pending => return Poll::Pending,
        };

        // Get time spend
        let spend_time = this.start.elapsed();

        // Log response
        log::info!(
            "response {} {} time={:?}",
            this.method,
            this.path,
            spend_time
        );

        Poll::Ready(result)
    }
}
