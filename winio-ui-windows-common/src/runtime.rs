use std::ops::Deref;
#[cfg(feature = "once_cell_try")]
use std::{cell::OnceCell, sync::OnceLock};

#[cfg(not(feature = "once_cell_try"))]
use once_cell::{sync::OnceCell as OnceLock, unsync::OnceCell};
use windows::Win32::{
    Graphics::{
        Direct2D::{D2D1_FACTORY_TYPE_MULTI_THREADED, D2D1CreateFactory, ID2D1Factory2},
        DirectWrite::{DWRITE_FACTORY_TYPE_SHARED, DWriteCreateFactory, IDWriteFactory},
        Imaging::{CLSID_WICImagingFactory, IWICImagingFactory},
    },
    System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance},
};

use crate::{CoInit, Result};

static D2D1_FACTORY: OnceLock<ID2D1Factory2> = OnceLock::new();

pub fn d2d1_factory() -> Result<&'static ID2D1Factory2> {
    D2D1_FACTORY
        .get_or_try_init(|| unsafe { D2D1CreateFactory(D2D1_FACTORY_TYPE_MULTI_THREADED, None) })
}

static DWRITE_FACTORY: OnceLock<IDWriteFactory> = OnceLock::new();

pub fn dwrite_factory() -> Result<&'static IDWriteFactory> {
    DWRITE_FACTORY.get_or_try_init(|| unsafe { DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED) })
}

struct WICImagingFactory(IWICImagingFactory, CoInit);

impl WICImagingFactory {
    fn new() -> Result<Self> {
        let co_init = CoInit::new()?;
        let factory =
            unsafe { CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)? };
        Ok(Self(factory, co_init))
    }
}

impl Deref for WICImagingFactory {
    type Target = IWICImagingFactory;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

thread_local! {
    static WIC_FACTORY: OnceCell<WICImagingFactory> = const { OnceCell::new() };
}

/// Call `f` with the thread-local WIC factory.
pub(crate) fn with_wic_factory<R>(f: impl FnOnce(&IWICImagingFactory) -> Result<R>) -> Result<R> {
    WIC_FACTORY.with(|cell| {
        let factory = cell.get_or_try_init(WICImagingFactory::new)?;
        f(factory)
    })
}
