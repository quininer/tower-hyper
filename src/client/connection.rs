use futures::{try_ready, Future, Async, Poll};
use http::{Request, Response};
use hyper::body::Payload;
use hyper::client::conn;
use tower_service::Service;
use crate::body::LiftBody;

/// The connection provided from `hyper`
///
/// This provides an interface for `DirectService` that will
/// drive the inner service via `poll_service` and can send
/// requests via `call`.
#[derive(Debug)]
pub struct Connection<B>
where
    B: Payload,
{
    sender: conn::SendRequest<B>,
}

impl<B> Connection<B>
where
    B: Payload,
{
    pub(super) fn new(sender: conn::SendRequest<B>) -> Self {
        Connection { sender }
    }
}

impl<B> Service<Request<B>> for Connection<B>
where
    B: Payload,
{
    type Response = Response<LiftBody<hyper::Body>>;
    type Error = hyper::Error;
    type Future = ResponseFuture;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        self.sender.poll_ready()
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        ResponseFuture(self.sender.send_request(req))
    }
}

/// A future of an HTTP response.
#[derive(Debug)]
pub struct ResponseFuture(conn::ResponseFuture);

impl Future for ResponseFuture {
    type Item = Response<LiftBody<hyper::Body>>;
    type Error = hyper::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let resp = try_ready!(self.0.poll());
        Ok(Async::Ready(resp.map(LiftBody::new)))
    }
}

#[allow(dead_code)]
fn assert_is_http_service(conn: Connection<hyper::Body>) {
    use tower_http_service::HttpService;

    fn assert<T: HttpService<hyper::Body>>(_: T) {}
    assert(conn);
}
