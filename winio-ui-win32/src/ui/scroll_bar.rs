use compio::driver::syscall;
use inherit_methods_macro::inherit_methods;
use windows_sys::Win32::UI::{
    Controls::{SetScrollInfo, WC_SCROLLBARW},
    WindowsAndMessaging::{
        GetScrollInfo, SB_CTL, SBS_HORZ, SBS_VERT, SCROLLINFO, SIF_PAGE, SIF_POS, SIF_RANGE,
        SIF_TRACKPOS, WM_HSCROLL, WM_VSCROLL, WS_CHILD, WS_VISIBLE,
    },
};
use winio_handle::{AsRawWidget, AsWindow, RawWidget};
use winio_primitive::{Orient, Point, Size};

use crate::Widget;

#[derive(Debug)]
struct ScrollBarImpl {
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl ScrollBarImpl {
    pub fn new(parent: impl AsWindow, style: u32) -> Self {
        let mut handle = Widget::new(
            WC_SCROLLBARW,
            WS_CHILD | style,
            0,
            parent.as_window().as_win32(),
        );
        handle.set_size(handle.size_d2l((50, 14)));
        Self { handle }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);

    fn info(&self, mask: u32) -> SCROLLINFO {
        let mut info: SCROLLINFO = unsafe { std::mem::zeroed() };
        info.cbSize = std::mem::size_of::<SCROLLINFO>() as _;
        info.fMask = mask;
        syscall!(
            BOOL,
            GetScrollInfo(self.handle.as_raw_widget().as_win32(), SB_CTL, &mut info)
        )
        .unwrap();
        info
    }

    fn set_info(&mut self, mask: u32, f: impl FnOnce(&mut SCROLLINFO)) {
        let mut info: SCROLLINFO = unsafe { std::mem::zeroed() };
        info.cbSize = std::mem::size_of::<SCROLLINFO>() as _;
        info.fMask = mask;
        f(&mut info);
        unsafe {
            SetScrollInfo(self.handle.as_raw_widget().as_win32(), SB_CTL, &info, 1);
        }
    }

    pub fn range(&self) -> (usize, usize) {
        let info = self.info(SIF_RANGE);
        (info.nMin as _, info.nMax as _)
    }

    pub fn set_range(&mut self, min: usize, max: usize) {
        self.set_info(SIF_RANGE, |info| {
            info.nMin = min as _;
            info.nMax = max as _;
        });
    }

    pub fn page(&self) -> usize {
        let info = self.info(SIF_PAGE);
        info.nPage as _
    }

    pub fn set_page(&mut self, v: usize) {
        self.set_info(SIF_PAGE, |info| {
            info.nPage = v as _;
        });
    }

    pub fn pos(&self) -> usize {
        let info = self.info(SIF_TRACKPOS);
        info.nTrackPos as _
    }

    pub fn set_pos(&mut self, v: usize) {
        self.set_info(SIF_POS, |info| {
            info.nPos = v as _;
        })
    }
}

impl AsRawWidget for ScrollBarImpl {
    fn as_raw_widget(&self) -> RawWidget {
        self.handle.as_raw_widget()
    }
}

#[derive(Debug)]
pub struct ScrollBar {
    handle: ScrollBarImpl,
    vhandle: ScrollBarImpl,
    vertical: bool,
}

impl ScrollBar {
    pub fn new(parent: impl AsWindow) -> Self {
        let parent = parent.as_window();
        let handle = ScrollBarImpl::new(&parent, WS_VISIBLE | SBS_HORZ as u32);
        let vhandle = ScrollBarImpl::new(&parent, SBS_VERT as u32);
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
            Size::new(20.0, 0.0)
        } else {
            Size::new(0.0, 20.0)
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

    pub fn range(&self) -> (usize, usize) {
        self.handle.range()
    }

    pub fn set_range(&mut self, (min, max): (usize, usize)) {
        self.handle.set_range(min, max);
        self.vhandle.set_range(min, max);
    }

    pub fn page(&self) -> usize {
        self.handle.page()
    }

    pub fn set_page(&mut self, v: usize) {
        self.handle.set_page(v);
        self.vhandle.set_page(v);
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

impl AsRawWidget for ScrollBar {
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

winio_handle::impl_as_widget!(ScrollBar);
