use std::future::Future;

cfg_if::cfg_if! {
    if #[cfg(windows)] {
        mod windows;
        pub use windows::*;
    } else if #[cfg(target_os = "macos")] {
        mod mac;
        pub use mac::*;
    } else {
        #[cfg(all(not(feature = "gtk"), not(feature = "qt")))]
        compile_error!("You must choose one of these features: [\"gtk\", \"qt\"]");

        cfg_if::cfg_if! {
            if #[cfg(feature = "qt")] {
                mod qt;
                pub use qt::*;
            } else {
                mod gtk;
                pub use gtk::*;
            }
        }
    }
}

thread_local! {
    pub(crate) static RUNTIME: Runtime = Runtime::new();
}

pub fn block_on<F: Future>(future: F) -> F::Output {
    RUNTIME.with(|runtime| runtime.block_on(future))
}
