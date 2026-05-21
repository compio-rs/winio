use std::time::Duration;

use inherit_methods_macro::inherit_methods;
use objc2::{
    MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
};
use objc2_foundation::{MainThreadMarker, NSObject, NSObjectProtocol};
use objc2_ui_kit::UIView;
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{Result, Widget, catch};

#[derive(Debug)]
pub struct Media {
    handle: Widget,
    delegate: Retained<PlayerDelegate>,
}

#[inherit_methods(from = "self.handle")]
impl Media {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let parent = parent.as_container();
        let mtm = parent.as_ui_kit().mtm();

        catch(|| {
            let view = UIView::new(mtm);
            let handle =
                Widget::from_uiview(parent, unsafe { Retained::cast_unchecked(view.clone()) })?;

            let delegate = PlayerDelegate::new(mtm);

            Ok(Self { handle, delegate })
        })
        .flatten()
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn url(&self) -> Result<String> {
        Ok(String::new())
    }

    pub async fn load(&mut self, _url: impl AsRef<str>) -> Result<()> {
        Ok(())
    }

    pub fn play(&mut self) -> Result<()> {
        Ok(())
    }

    pub fn pause(&mut self) -> Result<()> {
        Ok(())
    }

    pub fn full_time(&self) -> Result<Option<Duration>> {
        Ok(None)
    }

    pub fn current_time(&self) -> Result<Duration> {
        Ok(Duration::ZERO)
    }

    pub fn set_current_time(&mut self, _t: Duration) -> Result<()> {
        Ok(())
    }

    pub fn volume(&self) -> Result<f64> {
        Ok(1.0)
    }

    pub fn set_volume(&mut self, _v: f64) -> Result<()> {
        Ok(())
    }

    pub fn is_muted(&self) -> Result<bool> {
        Ok(false)
    }

    pub fn set_muted(&mut self, _v: bool) -> Result<()> {
        Ok(())
    }

    pub fn is_looped(&self) -> Result<bool> {
        Ok(false)
    }

    pub fn set_looped(&mut self, _v: bool) -> Result<()> {
        Ok(())
    }

    pub fn playback_rate(&self) -> Result<f64> {
        Ok(1.0)
    }

    pub fn set_playback_rate(&mut self, _v: f64) -> Result<()> {
        Ok(())
    }
}

winio_handle::impl_as_widget!(Media, handle);

#[derive(Debug, Default)]
struct PlayerDelegateIvars {
    notify: Callback,
}

define_class! {
    #[unsafe(super(NSObject))]
    #[name = "WinioPlayerDelegateUIKit"]
    #[ivars = PlayerDelegateIvars]
    #[thread_kind = MainThreadOnly]
    #[derive(Debug)]
    struct PlayerDelegate;

    #[allow(non_snake_case)]
    impl PlayerDelegate {
        #[unsafe(method_id(init))]
        fn init(this: Allocated<Self>) -> Option<Retained<Self>> {
            let this = this.set_ivars(PlayerDelegateIvars::default());
            unsafe { msg_send![super(this), init] }
        }
    }

    unsafe impl NSObjectProtocol for PlayerDelegate {}
}

impl PlayerDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), init] }
    }
}
