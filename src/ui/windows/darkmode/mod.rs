cfg_if::cfg_if! {
    if #[cfg(feature = "windows-dark-mode")] {
        #[path = "hook.rs"]
        mod imp;
        pub use imp::*;
    } else {
        #[path = "fallback.rs"]
        mod imp;
        pub use imp::*;
    }
}

#[repr(u32)]
#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub enum PreferredAppMode {
    Default    = 0,
    AllowDark  = 1,
    ForceDark  = 2,
    ForceLight = 3,
}
