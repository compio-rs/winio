//! Windows common methods for winio.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(feature = "once_cell_try", feature(once_cell_try))]
#![cfg(windows)]

pub use windows::core::{Error, Result};

mod accent;
pub use accent::*;

mod filebox;
pub use filebox::*;

mod msgbox;
pub use msgbox::*;

mod monitor;
pub use monitor::*;

mod canvas;
pub use canvas::*;

mod darkmode;
pub use darkmode::*;

mod resource;
pub use resource::*;

mod backdrop;
pub use backdrop::*;

mod runtime;
pub use runtime::*;

pub(crate) async fn spawn_blocking<F, R>(f: F) -> R
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let (tx, rx) = oneshot::channel();
    windows_threading::submit(move || {
        tx.send(f()).ok();
    });
    rx.await.expect("thread panicked or was cancelled")
}

#[macro_export]
macro_rules! syscall {
    (BOOL, $e:expr) => {
        $crate::syscall!($e, == 0)
    };
    (SOCKET, $e:expr) => {
        $crate::syscall!($e, != 0)
    };
    (HANDLE, $e:expr) => {
        $crate::syscall!($e, == ::windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE)
    };
    ($e:expr, $op: tt $rhs: expr) => {{
        #[allow(unused_unsafe, clippy::macro_metavars_in_unsafe)]
        let res = unsafe { $e };
        if res $op $rhs {
            Err($crate::Error::from_thread())
        } else {
            Ok(res)
        }
    }};
}
