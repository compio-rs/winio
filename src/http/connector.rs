use std::{
    future::Future,
    io,
    pin::Pin,
    task::{Context, Poll},
};

use hyper::Uri;
use send_wrapper::SendWrapper;
use tower_service::Service;

use crate::{http::HttpStream, ui::AsRawWindow};

/// An HTTP connector service.
///
/// It panics when called in a different thread other than the thread creates
/// it.
#[derive(Debug, Clone)]
pub struct Connector<W: AsRawWindow> {
    parent: SendWrapper<W>,
}

impl<W: AsRawWindow> Connector<W> {
    /// Creates the connector with specific TLS backend.
    pub fn new(parent: W) -> Self {
        Self {
            parent: SendWrapper::new(parent),
        }
    }
}

impl<W: AsRawWindow + Clone + 'static> Service<Uri> for Connector<W> {
    type Error = io::Error;
    type Future = Pin<Box<dyn Future<Output = io::Result<Self::Response>> + Send>>;
    type Response = HttpStream;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Uri) -> Self::Future {
        Box::pin(SendWrapper::new(HttpStream::connect(
            req,
            self.parent.clone().take(),
        )))
    }
}
