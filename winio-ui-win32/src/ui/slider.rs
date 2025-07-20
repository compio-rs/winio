use inherit_methods_macro::inherit_methods;
use windows_sys::Win32::{
    Foundation::HWND,
    UI::{
        Controls::{
            TBM_GETRANGEMAX, TBM_GETRANGEMIN, TBM_GETTOOLTIPS, TBM_SETPOSNOTIFY, TBM_SETRANGEMAX,
            TBM_SETRANGEMIN, TBM_SETTICFREQ, TBS_AUTOTICKS, TBS_BOTH, TBS_HORZ, TBS_TOOLTIPS,
            TBS_VERT, TRACKBAR_CLASSW,
        },
        WindowsAndMessaging::{WM_HSCROLL, WM_USER, WM_VSCROLL, WS_CHILD, WS_TABSTOP, WS_VISIBLE},
    },
};
use winio_handle::{AsRawWidget, AsWindow, RawWidget};
use winio_primitive::{Orient, Point, Size};
use winio_ui_windows_common::control_use_dark_mode;

use crate::Widget;

#[derive(Debug)]
struct SliderImpl {
    handle: Widget,
    freq: usize,
}

#[inherit_methods(from = "self.handle")]
impl SliderImpl {
    pub fn new(parent: impl AsWindow, style: u32) -> Self {
        let mut handle = Widget::new(
            TRACKBAR_CLASSW,
            WS_CHILD | WS_TABSTOP | TBS_AUTOTICKS | TBS_BOTH | TBS_TOOLTIPS | style,
            0,
            parent.as_window().as_win32(),
        );
        handle.set_size(handle.size_d2l((50, 14)));
        let tooltip_handle = handle.send_message(TBM_GETTOOLTIPS, 0, 0) as HWND;
        unsafe {
            control_use_dark_mode(tooltip_handle, false);
            crate::runtime::refresh_font(tooltip_handle);
        }
        Self { handle, freq: 1 }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);

    pub fn minimum(&self) -> usize {
        self.handle.send_message(TBM_GETRANGEMIN, 0, 0) as _
    }

    pub fn set_minimum(&mut self, v: usize) {
        self.handle.send_message(TBM_SETRANGEMIN, 1, v as _);
    }

    pub fn maximum(&self) -> usize {
        self.handle.send_message(TBM_GETRANGEMAX, 0, 0) as _
    }

    pub fn set_maximum(&mut self, v: usize) {
        self.handle.send_message(TBM_SETRANGEMAX, 1, v as _);
    }

    pub fn freq(&self) -> usize {
        self.freq
    }

    pub fn set_freq(&mut self, v: usize) {
        self.freq = v;
        self.handle.send_message(TBM_SETTICFREQ, v, 0);
    }

    pub fn pos(&self) -> usize {
        // Why is it not in `windows-sys`?
        const TBM_GETPOS: u32 = WM_USER;
        self.handle.send_message(TBM_GETPOS, 0, 0) as _
    }

    pub fn set_pos(&mut self, v: usize) {
        self.handle.send_message(TBM_SETPOSNOTIFY, 0, v as _);
    }
}

impl AsRawWidget for SliderImpl {
    fn as_raw_widget(&self) -> RawWidget {
        self.handle.as_raw_widget()
    }
}

#[derive(Debug)]
pub struct Slider {
    handle: SliderImpl,
    vhandle: SliderImpl,
    vertical: bool,
}

impl Slider {
    pub fn new(parent: impl AsWindow) -> Self {
        let parent = parent.as_window();
        let handle = SliderImpl::new(&parent, WS_VISIBLE | TBS_HORZ);
        let vhandle = SliderImpl::new(&parent, TBS_VERT);
        Self {
            handle,
            vhandle,
            vertical: false,
        }
    }

    pub fn is_visible(&self) -> bool {
        if self.vertical {
            &self.vhandle
        } else {
            &self.handle
        }
        .is_visible()
    }

    pub fn set_visible(&mut self, v: bool) {
        if self.vertical {
            &mut self.vhandle
        } else {
            &mut self.handle
        }
        .set_visible(v);
    }

    pub fn is_enabled(&self) -> bool {
        self.handle.is_enabled()
    }

    pub fn set_enabled(&mut self, v: bool) {
        self.handle.set_enabled(v);
        self.vhandle.set_enabled(v);
    }

    pub fn preferred_size(&self) -> Size {
        if self.vertical {
            Size::new(40.0, 0.0)
        } else {
            Size::new(0.0, 40.0)
        }
    }

    pub fn loc(&self) -> Point {
        self.handle.loc()
    }

    pub fn set_loc(&mut self, p: Point) {
        self.handle.set_loc(p);
        self.vhandle.set_loc(p);
    }

    pub fn size(&self) -> Size {
        self.handle.size()
    }

    pub fn set_size(&mut self, v: Size) {
        self.handle.set_size(v);
        self.vhandle.set_size(v);
    }

    pub fn orient(&self) -> Orient {
        if self.vertical {
            Orient::Vertical
        } else {
            Orient::Horizontal
        }
    }

    pub fn set_orient(&mut self, v: Orient) {
        let v = matches!(v, Orient::Vertical);
        if self.vertical != v {
            if v {
                self.vhandle.set_pos(self.handle.pos());
                self.vhandle.set_visible(self.handle.is_visible());
                self.handle.set_visible(false);
            } else {
                self.handle.set_pos(self.vhandle.pos());
                self.handle.set_visible(self.vhandle.is_visible());
                self.vhandle.set_visible(false);
            }
            self.vertical = v;
        }
    }

    pub fn minimum(&self) -> usize {
        self.handle.minimum()
    }

    pub fn set_minimum(&mut self, v: usize) {
        self.handle.set_minimum(v);
        self.vhandle.set_minimum(v);
    }

    pub fn maximum(&self) -> usize {
        self.handle.maximum()
    }

    pub fn set_maximum(&mut self, v: usize) {
        self.handle.set_maximum(v);
        self.vhandle.set_maximum(v);
    }

    pub fn freq(&self) -> usize {
        self.handle.freq()
    }

    pub fn set_freq(&mut self, v: usize) {
        self.handle.set_freq(v);
        self.vhandle.set_freq(v);
    }

    pub fn pos(&self) -> usize {
        if self.vertical {
            &self.vhandle
        } else {
            &self.handle
        }
        .pos()
    }

    pub fn set_pos(&mut self, v: usize) {
        self.handle.set_pos(v);
        self.vhandle.set_pos(v);
    }

    pub async fn wait_change(&self) {
        if self.vertical {
            self.vhandle.handle.wait_parent(WM_VSCROLL).await;
        } else {
            self.handle.handle.wait_parent(WM_HSCROLL).await;
        }
    }
}

impl AsRawWidget for Slider {
    fn as_raw_widget(&self) -> RawWidget {
        if self.vertical {
            &self.vhandle
        } else {
            &self.handle
        }
        .as_raw_widget()
    }

    fn iter_raw_widgets(&self) -> impl Iterator<Item = RawWidget> {
        [self.handle.as_raw_widget(), self.vhandle.as_raw_widget()].into_iter()
    }
}

winio_handle::impl_as_widget!(Slider);
