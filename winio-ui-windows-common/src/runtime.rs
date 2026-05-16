#[cfg(feature = "once_cell_try")]
use std::cell::OnceCell;

#[cfg(not(feature = "once_cell_try"))]
use once_cell::sync::OnceCell;
use windows::Win32::Graphics::Direct2D::{
    D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1CreateFactory, ID2D1Factory2,
};

thread_local! {
    static D2D1_FACTORY: OnceCell<ID2D1Factory2> = const { OnceCell::new() };
}

pub fn d2d1_factory() -> crate::Result<ID2D1Factory2> {
    D2D1_FACTORY.with(|d2d1| {
        d2d1.get_or_try_init(|| unsafe {
            D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)
        })
        .cloned()
    })
}
