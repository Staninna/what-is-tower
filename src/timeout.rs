use futures::future::{ready, BoxFuture, Map, Ready};
use futures::{Future, TryFutureExt};
use hyper::service::make_service_fn;
use hyper::{Body, Method, Request, Response, Server};
use std::convert::Infallible;
use std::error::Error;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use tokio::time::Sleep;
use tower::{BoxError, Service};

#[derive(Clone, Copy)]
pub struct Timeout<S> {
    inner: S,
    timeout: Duration,
}

impl<S> Timeout<S> {
    pub fn new(inner: S, timeout: Duration) -> Self {
        Self { inner, timeout }
    }
}

impl<S, R> Service<R> for Timeout<S>
where
    S: Service<R>,
    S::Error: Error + Send + Sync + 'static,
{
    type Response = S::Response;
    type Error = BoxError;
    type Future = TimeoutFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(|err| err.into())
    }

    fn call(&mut self, req: R) -> Self::Future {
        TimeoutFuture {
            future: self.inner.call(req),
            sleep: tokio::time::sleep(self.timeout),
        }
    }
}

#[pin_project::pin_project]
pub struct TimeoutFuture<F> {
    #[pin]
    future: F,
    #[pin]
    sleep: Sleep,
}

// F == Wrapped Future
// T == Type to return Ok
// E == Error
impl<F, T, E> Future for TimeoutFuture<F>
where
    F: Future<Output = Result<T, E>>,
    E: Error + Send + Sync + 'static,
{
    type Output = Result<T, BoxError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        // Poll the future
        match this.future.poll(cx) {
            Poll::Pending => {}
            Poll::Ready(result) => {
                return match result {
                    Ok(res) => Poll::Ready(Ok(res)),
                    Err(err) => Poll::Ready(Err(err.into())),
                }
            }
        }

        // Poll the sleep
        match this.sleep.poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(_) => Poll::Ready(Err(Elapsed.into())),
        }
    }
}

#[derive(Debug)]
pub struct Elapsed;

impl std::error::Error for Elapsed {}

impl std::fmt::Display for Elapsed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "request timed out")
    }
}
