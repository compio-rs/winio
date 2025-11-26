use std::{io, mem::MaybeUninit, sync::Arc, time::Duration};

use compio::driver::syscall;
use compio_log::error;
use inherit_methods_macro::inherit_methods;
use windows::{
    Win32::{
        Media::MediaFoundation::{
            CLSID_MFMediaEngineClassFactory, IMFMediaEngine, IMFMediaEngineClassFactory,
            IMFMediaEngineEx, IMFMediaEngineNotify, IMFMediaEngineNotify_Impl,
            MF_MEDIA_ENGINE_CALLBACK, MF_MEDIA_ENGINE_EVENT, MF_MEDIA_ENGINE_EVENT_CANPLAY,
            MF_MEDIA_ENGINE_EVENT_ERROR, MF_MEDIA_ENGINE_PLAYBACK_HWND, MF_VERSION,
            MFCreateAttributes, MFSTARTUP_FULL, MFShutdown, MFStartup,
        },
        System::Com::{
            CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
            CoUninitialize,
        },
    },
    core::{BSTR, HRESULT, Interface, implement},
};
use windows_core::Error;
use windows_sys::Win32::{
    System::SystemServices::SS_OWNERDRAW,
    UI::{
        Controls::WC_STATICW,
        WindowsAndMessaging::{GetClientRect, WS_CHILD, WS_VISIBLE},
    },
};
use winio_callback::SyncCallback;
use winio_handle::{AsContainer, AsRawWidget, AsRawWindow};
use winio_primitive::{Point, Size};

use crate::{Result, Widget, ui::with_u16c};

#[derive(Debug)]
pub struct Media {
    handle: Widget,
    engine: IMFMediaEngine,
    notify: Arc<SyncCallback<Result<()>>>,
    #[allow(dead_code)]
    callback: IMFMediaEngineNotify,
    _guard: MFGuard,
}

#[inherit_methods(from = "self.handle")]
impl Media {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let _guard = MFGuard::init()?;

        let parent = parent.as_container().as_win32();

        let handle = Widget::new(WC_STATICW, WS_VISIBLE | WS_CHILD | SS_OWNERDRAW, 0, parent)?;

        unsafe {
            let factory: IMFMediaEngineClassFactory =
                CoCreateInstance(&CLSID_MFMediaEngineClassFactory, None, CLSCTX_INPROC_SERVER)?;

            let notify = Arc::new(SyncCallback::new());
            let callback: IMFMediaEngineNotify = MediaNotify::new(notify.clone()).into();

            let mut attrs = None;
            MFCreateAttributes(&mut attrs, 1)?;
            let attrs = attrs.ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "cannot create IMFAttributes")
            })?;
            attrs.SetUnknown(&MF_MEDIA_ENGINE_CALLBACK, &callback)?;
            attrs.SetUINT64(
                &MF_MEDIA_ENGINE_PLAYBACK_HWND,
                handle.as_raw_window().as_win32() as _,
            )?;

            let engine = factory.CreateInstance(0, &attrs)?;

            Ok(Self {
                handle,
                callback,
                notify,
                engine,
                _guard,
            })
        }
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size> {
        Ok(Size::zero())
    }

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()> {
        self.handle.set_size(v)?;
        self.update_rect()
    }

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    fn update_rect(&mut self) -> Result<()> {
        let handle = self.as_raw_widget().as_win32();
        let mut rect = MaybeUninit::uninit();
        syscall!(BOOL, unsafe { GetClientRect(handle, rect.as_mut_ptr()) })?;
        let rect = unsafe { rect.assume_init() };

        unsafe {
            self.engine.cast::<IMFMediaEngineEx>()?.UpdateVideoStream(
                None,
                Some(std::ptr::addr_of!(rect).cast()),
                None,
            )?;
        }
        Ok(())
    }

    pub fn url(&self) -> Result<String> {
        unsafe { Ok(self.engine.GetCurrentSource()?.to_string()) }
    }

    pub async fn load(&mut self, url: impl AsRef<str>) -> Result<()> {
        unsafe {
            with_u16c(url.as_ref(), |s| {
                self.engine.SetSource(&BSTR::from_wide(s.as_slice()))?;
                Ok(())
            })?;
        }
        self.notify.wait().await
    }

    pub fn play(&mut self) -> Result<()> {
        unsafe { self.engine.Play()? };
        Ok(())
    }

    pub fn pause(&mut self) -> Result<()> {
        unsafe { self.engine.Pause()? };
        Ok(())
    }

    pub fn full_time(&self) -> Result<Option<Duration>> {
        unsafe { Ok(Duration::try_from_secs_f64(self.engine.GetDuration()).ok()) }
    }

    pub fn current_time(&self) -> Result<Duration> {
        unsafe { Ok(Duration::from_secs_f64(self.engine.GetCurrentTime())) }
    }

    pub fn set_current_time(&mut self, t: Duration) -> Result<()> {
        unsafe { self.engine.SetCurrentTime(t.as_secs_f64())? };
        Ok(())
    }

    pub fn volume(&self) -> Result<f64> {
        unsafe { Ok(self.engine.GetVolume()) }
    }

    pub fn set_volume(&mut self, v: f64) -> Result<()> {
        unsafe { self.engine.SetVolume(v)? };
        Ok(())
    }

    pub fn is_muted(&self) -> Result<bool> {
        unsafe { Ok(self.engine.GetMuted().as_bool()) }
    }

    pub fn set_muted(&mut self, v: bool) -> Result<()> {
        unsafe { self.engine.SetMuted(v)? };
        Ok(())
    }
}

winio_handle::impl_as_widget!(Media, handle);

#[derive(Debug)]
struct MFGuard;

impl MFGuard {
    pub fn init() -> Result<Self> {
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
            MFStartup(MF_VERSION, MFSTARTUP_FULL)?;
        }
        Ok(Self)
    }
}

impl Drop for MFGuard {
    fn drop(&mut self) {
        unsafe {
            if let Err(_e) = MFShutdown() {
                error!("MFShutdown: {:?}", _e);
            }
            CoUninitialize();
        }
    }
}

#[implement(IMFMediaEngineNotify)]
struct MediaNotify {
    notify: Arc<SyncCallback<Result<()>>>,
}

impl MediaNotify {
    pub fn new(notify: Arc<SyncCallback<Result<()>>>) -> Self {
        Self { notify }
    }
}

impl IMFMediaEngineNotify_Impl for MediaNotify_Impl {
    fn EventNotify(&self, event: u32, _param1: usize, param2: u32) -> Result<()> {
        let msg = match MF_MEDIA_ENGINE_EVENT(event as _) {
            MF_MEDIA_ENGINE_EVENT_CANPLAY => Some(Ok(())),
            MF_MEDIA_ENGINE_EVENT_ERROR => Some(Err(Error::from_hresult(HRESULT(param2 as _)))),
            _ => None,
        };
        if let Some(msg) = msg {
            self.notify.signal(msg);
        }
        Ok(())
    }
}
