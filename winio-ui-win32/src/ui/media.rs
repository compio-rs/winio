use std::{mem::MaybeUninit, time::Duration};

use compio::driver::syscall;
use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures_util::StreamExt;
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
    core::{BSTR, Interface, Result, implement},
};
use windows_sys::Win32::{
    System::SystemServices::SS_OWNERDRAW,
    UI::{
        Controls::WC_STATICW,
        WindowsAndMessaging::{GetClientRect, WS_CHILD, WS_VISIBLE},
    },
};
use winio_handle::{AsContainer, AsRawWidget, AsRawWindow};
use winio_primitive::{Point, Size};

use crate::{Widget, ui::with_u16c};

#[derive(Debug)]
pub struct Media {
    handle: Widget,
    engine: IMFMediaEngine,
    notify: UnboundedReceiver<bool>,
    #[allow(dead_code)]
    callback: IMFMediaEngineNotify,
    _guard: MFGuard,
}

#[inherit_methods(from = "self.handle")]
impl Media {
    pub fn new(parent: impl AsContainer) -> Self {
        let _guard = MFGuard::init();

        let mut handle = Widget::new(
            WC_STATICW,
            WS_VISIBLE | WS_CHILD | SS_OWNERDRAW,
            0,
            parent.as_container().as_win32(),
        );
        handle.set_size(handle.size_d2l((50, 14)));

        unsafe {
            let factory: IMFMediaEngineClassFactory =
                CoCreateInstance(&CLSID_MFMediaEngineClassFactory, None, CLSCTX_INPROC_SERVER)
                    .unwrap();

            let (tx, rx) = futures_channel::mpsc::unbounded();
            let callback: IMFMediaEngineNotify = MediaNotify { notify: tx }.into();

            let mut attrs = None;
            MFCreateAttributes(&mut attrs, 1).unwrap();
            let attrs = attrs.unwrap();
            attrs
                .SetUnknown(&MF_MEDIA_ENGINE_CALLBACK, &callback)
                .unwrap();
            attrs
                .SetUINT64(
                    &MF_MEDIA_ENGINE_PLAYBACK_HWND,
                    handle.as_raw_window().as_win32() as _,
                )
                .unwrap();

            let engine = factory.CreateInstance(0, &attrs).unwrap();

            Self {
                handle,
                callback,
                notify: rx,
                engine,
                _guard,
            }
        }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn preferred_size(&self) -> Size {
        Size::zero()
    }

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size) {
        self.handle.set_size(v);
        self.update_rect();
    }

    pub fn tooltip(&self) -> String;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>);

    fn update_rect(&mut self) {
        let handle = self.as_raw_widget().as_win32();
        let mut rect = MaybeUninit::uninit();
        syscall!(BOOL, unsafe { GetClientRect(handle, rect.as_mut_ptr()) }).unwrap();
        let rect = unsafe { rect.assume_init() };

        unsafe {
            self.engine
                .cast::<IMFMediaEngineEx>()
                .unwrap()
                .UpdateVideoStream(None, Some(std::ptr::addr_of!(rect).cast()), None)
                .unwrap();
        }
    }

    pub fn url(&self) -> String {
        unsafe { self.engine.GetCurrentSource().unwrap().to_string() }
    }

    pub async fn load(&mut self, url: impl AsRef<str>) -> bool {
        unsafe {
            with_u16c(url.as_ref(), |s| {
                self.engine
                    .SetSource(&BSTR::from_wide(s.as_slice()))
                    .unwrap();
            })
        }
        self.notify.next().await.unwrap_or_default()
    }

    pub fn play(&mut self) {
        unsafe { self.engine.Play().unwrap() }
    }

    pub fn pause(&mut self) {
        unsafe { self.engine.Pause().unwrap() }
    }

    pub fn full_time(&self) -> Option<Duration> {
        unsafe { Duration::try_from_secs_f64(self.engine.GetDuration()).ok() }
    }

    pub fn current_time(&self) -> Duration {
        unsafe { Duration::from_secs_f64(self.engine.GetCurrentTime()) }
    }

    pub fn set_current_time(&mut self, t: Duration) {
        unsafe { self.engine.SetCurrentTime(t.as_secs_f64()).unwrap() }
    }

    pub fn volume(&self) -> f64 {
        unsafe { self.engine.GetVolume() }
    }

    pub fn set_volume(&mut self, v: f64) {
        unsafe { self.engine.SetVolume(v).unwrap() }
    }

    pub fn is_muted(&self) -> bool {
        unsafe { self.engine.GetMuted().as_bool() }
    }

    pub fn set_muted(&mut self, v: bool) {
        unsafe { self.engine.SetMuted(v).unwrap() }
    }
}

winio_handle::impl_as_widget!(Media, handle);

#[derive(Debug)]
struct MFGuard;

impl MFGuard {
    pub fn init() -> Self {
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED).unwrap();
            MFStartup(MF_VERSION, MFSTARTUP_FULL).unwrap();
        }
        Self
    }
}

impl Drop for MFGuard {
    fn drop(&mut self) {
        unsafe {
            MFShutdown().unwrap();
            CoUninitialize();
        }
    }
}

#[implement(IMFMediaEngineNotify)]
struct MediaNotify {
    notify: UnboundedSender<bool>,
}

impl IMFMediaEngineNotify_Impl for MediaNotify_Impl {
    fn EventNotify(&self, event: u32, _param1: usize, _param2: u32) -> Result<()> {
        match MF_MEDIA_ENGINE_EVENT(event as _) {
            MF_MEDIA_ENGINE_EVENT_CANPLAY => {
                self.notify.unbounded_send(true).ok();
            }
            MF_MEDIA_ENGINE_EVENT_ERROR => {
                self.notify.unbounded_send(false).ok();
            }
            _ => {}
        }
        Ok(())
    }
}
