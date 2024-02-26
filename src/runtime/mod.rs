use std::future::Future;

cfg_if::cfg_if! {
    if #[cfg(windows)] {
        mod windows;
        pub use windows::*;
    } else if #[cfg(target_os = "macos")] {
        mod mac;
        pub use mac::*;
    }
}

thread_local! {
    static RUNTIME: Runtime = Runtime::new();
}

pub fn block_on<F: Future>(future: F) -> F::Output {
    RUNTIME.with(|runtime| runtime.block_on(future))
}
