use std::future::Future;

use hyper::rt::Executor;

/// An executor service based on [`compio::runtime`]. It uses
/// [`compio::runtime::spawn`] interally.
#[derive(Debug, Default, Clone)]
pub struct WinioExecutor;

impl<F: Future<Output = ()> + Send + 'static> Executor<F> for WinioExecutor {
    fn execute(&self, fut: F) {
        crate::spawn(fut).detach();
    }
}
