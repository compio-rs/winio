use std::{
    cell::RefCell,
    ffi::c_void,
    io,
    ops::Deref,
    os::fd::AsRawFd,
    pin::Pin,
    task::{Context, Poll, Waker},
    time::Duration,
};

use compio::{compat::Adapter, runtime::Runtime};
use objc2_core_foundation::{
    CFFileDescriptor, CFRetained, CFRunLoop, CFRunLoopSource, kCFAllocatorDefault,
    kCFFileDescriptorReadCallBack, kCFRunLoopDefaultMode,
};

pub struct CompioAdapter {
    runtime: Runtime,
    fd_source: CFRetained<CFFileDescriptor>,
    source: CFRetained<CFRunLoopSource>,
    run_loop: CFRetained<CFRunLoop>,
}

impl Deref for CompioAdapter {
    type Target = Runtime;

    fn deref(&self) -> &Self::Target {
        &self.runtime
    }
}

impl Adapter for CompioAdapter {
    fn new(runtime: Runtime) -> io::Result<Self> {
        unsafe extern "C-unwind" fn fd_callback(
            _fdref: *mut CFFileDescriptor,
            _callback_types: usize,
            _info: *mut c_void,
        ) {
        }

        let fd_source = unsafe {
            CFFileDescriptor::new(
                kCFAllocatorDefault,
                runtime.as_raw_fd(),
                false,
                Some(fd_callback),
                std::ptr::null(),
            )
        }
        .ok_or(io::ErrorKind::InvalidData)?;
        let source = unsafe {
            CFFileDescriptor::new_run_loop_source(kCFAllocatorDefault, Some(&fd_source), 0)
        }
        .ok_or(io::ErrorKind::InvalidData)?;

        let run_loop = CFRunLoop::current().ok_or(io::ErrorKind::InvalidData)?;
        run_loop.add_source(Some(&source), unsafe { kCFRunLoopDefaultMode });

        Ok(Self {
            runtime,
            fd_source,
            source,
            run_loop,
        })
    }

    async fn wait(&self, timeout: Option<Duration>) -> io::Result<()> {
        self.fd_source
            .enable_call_backs(kCFFileDescriptorReadCallBack);

        WaitFuture::new(timeout).await
    }

    fn clear(&self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for CompioAdapter {
    fn drop(&mut self) {
        self.fd_source
            .disable_call_backs(kCFFileDescriptorReadCallBack);
        self.run_loop
            .remove_source(Some(&self.source), unsafe { kCFRunLoopDefaultMode });
    }
}

thread_local! {
    static CONTEXT: RefCell<Option<(Option<Duration>, Waker)>> = const { RefCell::new(None) };
}

pub(crate) fn get_context() -> (Option<Duration>, Option<Waker>) {
    CONTEXT.with_borrow(|ctx| {
        if let Some((timeout, waker)) = ctx.as_ref() {
            (*timeout, Some(waker.clone()))
        } else {
            (None, None)
        }
    })
}

fn set_context(timeout: Option<Duration>, waker: Waker) {
    CONTEXT.with_borrow_mut(|ctx| ctx.replace((timeout, waker)));
}

fn reset_context() {
    CONTEXT.with_borrow_mut(|ctx| ctx.take());
}

struct WaitFuture {
    timeout: Option<Duration>,
    polled: bool,
}

impl WaitFuture {
    fn new(timeout: Option<Duration>) -> Self {
        Self {
            timeout,
            polled: false,
        }
    }
}

impl Future for WaitFuture {
    type Output = io::Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.polled {
            Poll::Ready(Ok(()))
        } else {
            set_context(self.timeout, cx.waker().clone());
            self.polled = true;
            Poll::Pending
        }
    }
}

impl Drop for WaitFuture {
    fn drop(&mut self) {
        reset_context();
    }
}
