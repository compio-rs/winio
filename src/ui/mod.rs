cfg_if::cfg_if! {
    if #[cfg(windows)] {
        mod windows;
        pub use windows::*;
    } else if #[cfg(target_os = "macos")] {
        mod mac;
        pub use mac::*;
    } else {
        mod gtk;
        pub use gtk::*;
    }
}

mod drawing;
pub use drawing::*;

mod msgbox;
pub use msgbox::*;

mod canvas;
pub use canvas::*;
