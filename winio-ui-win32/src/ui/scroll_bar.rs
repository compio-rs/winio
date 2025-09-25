use compio::driver::syscall;
use inherit_methods_macro::inherit_methods;
use windows_sys::Win32::UI::{
    Controls::{SetScrollInfo, WC_SCROLLBARW},
    WindowsAndMessaging::{
        GetParent, GetScrollInfo, SB_CTL, SBS_HORZ, SBS_VERT, SCROLLINFO, SIF_PAGE, SIF_POS,
        SIF_RANGE, SIF_TRACKPOS, WM_HSCROLL, WS_CHILD, WS_VISIBLE,
    },
};
use winio_handle::{AsContainer, AsRawWidget, BorrowedContainer, RawContainer, RawWidget};
use winio_primitive::{Orient, Point, Size};

use crate::Widget;

#[derive(Debug)]
struct ScrollBarImpl {
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl ScrollBarImpl {
    pub fn new(parent: impl AsContainer, style: u32) -> Self {
        let mut handle = Widget::new(
            WC_SCROLLBARW,
            WS_CHILD | style,
            0,
            parent.as_container().as_win32(),
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

    pub fn tooltip(&self) -> String;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>);

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

    pub fn minimum(&self) -> usize {
        let info = self.info(SIF_RANGE);
        info.nMin as _
    }

    pub fn maximum(&self) -> usize {
        let info = self.info(SIF_RANGE);
        info.nMax as _
    }

    pub fn set_minimum(&mut self, v: usize) {
        let max = self.maximum();
        self.set_info(SIF_RANGE, |info| {
            info.nMin = v as _;
            info.nMax = max as _;
        });
    }

    pub fn set_maximum(&mut self, v: usize) {
        let min = self.minimum();
        self.set_info(SIF_RANGE, |info| {
            info.nMin = min as _;
            info.nMax = v as _;
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
    vertical: bool,
}

#[inherit_methods(from = "self.handle")]
impl ScrollBar {
    pub fn new(parent: impl AsContainer) -> Self {
        let handle = ScrollBarImpl::new(&parent, WS_VISIBLE | SBS_HORZ as u32);
        Self {
            handle,
            vertical: false,
        }
    }

    fn recreate(&mut self, vertical: bool) {
        let parent = unsafe { GetParent(self.handle.as_raw_widget().as_win32()) };
        let mut new_handle = ScrollBarImpl::new(
            unsafe { BorrowedContainer::borrow_raw(RawContainer::Win32(parent)) },
            if vertical {
                SBS_VERT as u32
            } else {
                SBS_HORZ as u32
            } | WS_VISIBLE,
        );
        new_handle.set_visible(self.handle.is_visible());
        new_handle.set_enabled(self.handle.is_enabled());
        new_handle.set_loc(self.handle.loc());
        new_handle.set_size(self.handle.size());
        new_handle.set_tooltip(self.handle.tooltip());
        new_handle.set_minimum(self.handle.minimum());
        new_handle.set_maximum(self.handle.maximum());
        new_handle.set_page(self.handle.page());
        new_handle.set_pos(self.handle.pos());
        self.handle = new_handle;
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn preferred_size(&self) -> Size {
        if self.vertical {
            Size::new(20.0, 0.0)
        } else {
            Size::new(0.0, 20.0)
        }
    }

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);

    pub fn tooltip(&self) -> String;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>);

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
            self.recreate(v);
            self.vertical = v;
        }
    }

    pub fn minimum(&self) -> usize;

    pub fn set_minimum(&mut self, v: usize);

    pub fn maximum(&self) -> usize;

    pub fn set_maximum(&mut self, v: usize);

    pub fn page(&self) -> usize;

    pub fn set_page(&mut self, v: usize);

    pub fn pos(&self) -> usize;

    pub fn set_pos(&mut self, v: usize);

    pub async fn wait_change(&self) {
        self.handle.handle.wait_parent(WM_HSCROLL).await;
    }
}

winio_handle::impl_as_widget!(ScrollBar, handle);
