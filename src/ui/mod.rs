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

mod drawing;
pub use drawing::*;

mod msgbox;
pub use msgbox::*;

mod canvas;
pub use canvas::*;

#[cfg(not(windows))]
mod callback;
#[cfg(not(windows))]
pub(crate) use callback::*;
