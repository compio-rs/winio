use std::sync::OnceLock;

use inherit_methods_macro::inherit_methods;
use windows_sys::Win32::UI::{
    Controls::{
        TBM_GETRANGEMAX, TBM_GETRANGEMIN, TBM_SETPOS, TBM_SETPOSNOTIFY, TBM_SETRANGE,
        TBM_SETRANGEMAX, TBM_SETRANGEMIN, TBM_SETTICFREQ, TBS_AUTOTICKS, TBS_BOTH, TBS_BOTTOM,
        TBS_HORZ, TBS_TOP, TBS_VERT, TKP_THUMB, TKP_THUMBVERT, TKP_TICS, TRACKBAR_CLASSW,
        TUS_NORMAL, VSCLASS_TRACKBAR,
    },
    HiDpi::GetSystemMetricsForDpi,
    WindowsAndMessaging::{
        GetParent, SM_CYVSCROLL, USER_DEFAULT_SCREEN_DPI, WM_HSCROLL, WM_USER, WM_VSCROLL,
        WS_CHILD, WS_TABSTOP, WS_VISIBLE,
    },
};
use winio_handle::{AsContainer, AsWidget, BorrowedContainer};
use winio_primitive::{Orient, Point, Size, TickPosition};

use crate::{Result, Widget, platform::theme::Theme};

#[derive(Debug, Clone, Copy)]
struct Metrics {
    thumb_width: f64,
    thumb_height: f64,
    tick: f64,
}

#[derive(Debug)]
struct SliderImpl {
    handle: Widget,
    freq: usize,
}

#[inherit_methods(from = "self.handle")]
impl SliderImpl {
    pub fn new(parent: impl AsContainer, style: u32) -> Result<Self> {
        let handle = Widget::new(
            TRACKBAR_CLASSW,
            WS_CHILD | WS_TABSTOP | TBS_AUTOTICKS | style,
            0,
            parent.as_container().as_win32(),
        )?;
        // Set the range to 0..10, frequency to 1, and position to 0.
        handle.send_message(TBM_SETRANGE, 0, 0x000a0000);
        handle.send_message(TBM_SETTICFREQ, 1, 0);
        handle.send_message(TBM_SETPOS, 1, 0);
        Ok(Self { handle, freq: 1 })
    }

    fn set_range(&mut self, min: usize, max: usize) {
        let old_max = self.maximum().unwrap_or(0);
        if min > old_max {
            self.handle.send_message(TBM_SETRANGEMAX, 1, max as _);
            self.handle.send_message(TBM_SETRANGEMIN, 1, min as _);
        } else {
            self.handle.send_message(TBM_SETRANGEMIN, 1, min as _);
            self.handle.send_message(TBM_SETRANGEMAX, 1, max as _);
        }
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

    pub fn minimum(&self) -> Result<usize> {
        Ok(self.handle.send_message(TBM_GETRANGEMIN, 0, 0) as _)
    }

    pub fn set_minimum(&mut self, v: usize) -> Result<()> {
        let max = self.maximum()?.max(v.saturating_add(1));
        self.set_range(v, max);
        Ok(())
    }

    pub fn maximum(&self) -> Result<usize> {
        Ok(self.handle.send_message(TBM_GETRANGEMAX, 0, 0) as _)
    }

    pub fn set_maximum(&mut self, v: usize) -> Result<()> {
        let min = self.minimum()?;
        let min = if v < min.saturating_add(1) {
            v.saturating_sub(1)
        } else {
            min
        };
        let max = v.max(min.saturating_add(1));
        self.set_range(min, max);
        Ok(())
    }

    pub fn freq(&self) -> Result<usize> {
        Ok(self.freq)
    }

    pub fn set_freq(&mut self, v: usize) -> Result<()> {
        let v = v.max(1);
        self.freq = v;
        self.handle.send_message(TBM_SETTICFREQ, v, 0);
        Ok(())
    }

    pub fn pos(&self) -> Result<usize> {
        // Why isn't it in `windows-sys`?
        const TBM_GETPOS: u32 = WM_USER;
        Ok(self.handle.send_message(TBM_GETPOS, 0, 0) as _)
    }

    pub fn set_pos(&mut self, v: usize) -> Result<()> {
        self.handle.send_message(TBM_SETPOSNOTIFY, 0, v as _);
        Ok(())
    }
}

winio_handle::impl_as_widget!(SliderImpl, handle);

#[derive(Debug)]
pub struct Slider {
    handle: SliderImpl,
    vertical: bool,
    tick_pos: TickPosition,
}

#[inherit_methods(from = "self.handle")]
impl Slider {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let handle = SliderImpl::new(&parent, WS_VISIBLE | TBS_BOTH | TBS_HORZ)?;
        Ok(Self {
            handle,
            vertical: false,
            tick_pos: TickPosition::Both,
        })
    }

    fn recreate(&mut self, vertical: bool, tick_pos: TickPosition) -> Result<()> {
        let parent = unsafe { GetParent(self.handle.as_widget().as_win32()) };
        let mut style = WS_VISIBLE;
        style |= match tick_pos {
            TickPosition::None => 0,
            TickPosition::TopLeft => TBS_TOP,
            TickPosition::BottomRight => TBS_BOTTOM,
            TickPosition::Both => TBS_BOTH,
        };
        style |= if vertical { TBS_VERT } else { TBS_HORZ };
        let mut new_handle = SliderImpl::new(unsafe { BorrowedContainer::win32(parent) }, style)?;
        new_handle.set_visible(self.handle.is_visible()?)?;
        new_handle.set_enabled(self.handle.is_enabled()?)?;
        new_handle.set_loc(self.handle.loc()?)?;
        new_handle.set_size(self.handle.size()?)?;
        new_handle.set_tooltip(self.handle.tooltip()?)?;
        new_handle.set_minimum(self.handle.minimum()?)?;
        new_handle.set_maximum(self.handle.maximum()?)?;
        new_handle.set_freq(self.handle.freq()?)?;
        new_handle.set_pos(self.handle.pos()?)?;
        self.handle = new_handle;
        Ok(())
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size> {
        let m = self.metrics();
        let thumb = if self.vertical {
            m.thumb_width
        } else {
            m.thumb_height
        };
        let ticks = match self.tick_pos {
            TickPosition::None => 0.0,
            TickPosition::TopLeft | TickPosition::BottomRight => 2.0 * m.tick,
            TickPosition::Both => 4.0 * m.tick,
        };
        let base_length = thumb + ticks;
        let size = if self.vertical {
            Size::new(base_length, 0.0)
        } else {
            Size::new(0.0, base_length)
        };
        Ok(size)
    }

    /// The sizes of the thumb and the tick marks, read once from the style.
    fn metrics(&self) -> Metrics {
        static METRICS: OnceLock<Metrics> = OnceLock::new();
        *METRICS.get_or_init(|| {
            let hwnd = self.handle.as_widget().as_win32();
            let theme = Theme::open(hwnd, VSCLASS_TRACKBAR);
            let thumb = theme
                .as_ref()
                .and_then(|theme| theme.part_size(TKP_THUMB, TUS_NORMAL));
            let thumb_vert = theme
                .as_ref()
                .and_then(|theme| theme.part_size(TKP_THUMBVERT, TUS_NORMAL));
            let scroll =
                unsafe { GetSystemMetricsForDpi(SM_CYVSCROLL, USER_DEFAULT_SCREEN_DPI) as f64 };
            Metrics {
                thumb_height: thumb.map(|s| s.height).unwrap_or(scroll),
                thumb_width: thumb_vert.map(|s| s.width).unwrap_or(scroll),
                tick: theme
                    .as_ref()
                    .and_then(|theme| theme.part_size(TKP_TICS, TUS_NORMAL))
                    .map(|s| s.height)
                    .unwrap_or_default(),
            }
        })
    }

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn tick_pos(&self) -> Result<TickPosition> {
        Ok(self.tick_pos)
    }

    pub fn set_tick_pos(&mut self, v: TickPosition) -> Result<()> {
        if self.tick_pos != v {
            self.recreate(self.vertical, v)?;
            self.tick_pos = v;
        }
        Ok(())
    }

    pub fn orient(&self) -> Result<Orient> {
        let orient = if self.vertical {
            Orient::Vertical
        } else {
            Orient::Horizontal
        };
        Ok(orient)
    }

    pub fn set_orient(&mut self, v: Orient) -> Result<()> {
        let v = matches!(v, Orient::Vertical);
        if self.vertical != v {
            self.recreate(v, self.tick_pos)?;
            self.vertical = v;
        }
        Ok(())
    }

    pub fn minimum(&self) -> Result<usize>;

    pub fn set_minimum(&mut self, v: usize) -> Result<()>;

    pub fn maximum(&self) -> Result<usize>;

    pub fn set_maximum(&mut self, v: usize) -> Result<()>;

    pub fn freq(&self) -> Result<usize>;

    pub fn set_freq(&mut self, v: usize) -> Result<()>;

    pub fn pos(&self) -> Result<usize>;

    pub fn set_pos(&mut self, v: usize) -> Result<()>;

    pub async fn wait_change(&self) {
        if self.vertical {
            self.handle.handle.wait_parent(WM_VSCROLL).await;
        } else {
            self.handle.handle.wait_parent(WM_HSCROLL).await;
        }
    }
}

winio_handle::impl_as_widget!(Slider, handle);
