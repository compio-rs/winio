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
    System::Com::{
        CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
        CoUninitialize,
    },
};

static D2D1_FACTORY: OnceLock<ID2D1Factory2> = OnceLock::new();

pub fn d2d1_factory() -> crate::Result<&'static ID2D1Factory2> {
    D2D1_FACTORY
        .get_or_try_init(|| unsafe { D2D1CreateFactory(D2D1_FACTORY_TYPE_MULTI_THREADED, None) })
}

static DWRITE_FACTORY: OnceLock<IDWriteFactory> = OnceLock::new();

pub fn dwrite_factory() -> crate::Result<&'static IDWriteFactory> {
    DWRITE_FACTORY.get_or_try_init(|| unsafe { DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED) })
}

struct WICImagingFactory(IWICImagingFactory);

impl WICImagingFactory {
    fn new() -> crate::Result<Self> {
        unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()? };
        let factory =
            unsafe { CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)? };
        Ok(Self(factory))
    }
}

impl Drop for WICImagingFactory {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
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
pub(crate) fn with_wic_factory<R>(
    f: impl FnOnce(&IWICImagingFactory) -> crate::Result<R>,
) -> crate::Result<R> {
    WIC_FACTORY.with(|cell| {
        let factory = cell.get_or_try_init(WICImagingFactory::new)?;
        f(factory)
    })
}
