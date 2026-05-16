#[cfg(feature = "once_cell_try")]
use std::sync::OnceLock;

#[cfg(not(feature = "once_cell_try"))]
use once_cell::sync::OnceCell as OnceLock;
use windows::Win32::Graphics::Direct2D::{
    D2D1_FACTORY_TYPE_MULTI_THREADED, D2D1CreateFactory, ID2D1Factory2,
};

static D2D1_FACTORY: OnceLock<ID2D1Factory2> = const { OnceLock::new() };

pub fn d2d1_factory() -> crate::Result<ID2D1Factory2> {
    D2D1_FACTORY
        .get_or_try_init(|| unsafe { D2D1CreateFactory(D2D1_FACTORY_TYPE_MULTI_THREADED, None) })
        .cloned()
}
