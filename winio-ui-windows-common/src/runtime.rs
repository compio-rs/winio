#[cfg(feature = "once_cell_try")]
use std::sync::OnceLock;

#[cfg(not(feature = "once_cell_try"))]
use once_cell::sync::OnceCell as OnceLock;
use windows_subset::Win32::{D2D1_FACTORY_TYPE_MULTI_THREADED, D2D1CreateFactory, ID2D1Factory2};

#[derive(Clone)]
struct D2D1FactoryWrap(ID2D1Factory2);

unsafe impl Send for D2D1FactoryWrap {}
unsafe impl Sync for D2D1FactoryWrap {}

static D2D1_FACTORY: OnceLock<D2D1FactoryWrap> = OnceLock::new();

pub fn d2d1_factory() -> crate::Result<ID2D1Factory2> {
    D2D1_FACTORY
        .get_or_try_init(|| {
            unsafe { D2D1CreateFactory(D2D1_FACTORY_TYPE_MULTI_THREADED, None) }
                .map(D2D1FactoryWrap)
        })
        .cloned()
        .map(|wrap| wrap.0)
}
