use compio::driver::syscall;
use inherit_methods_macro::inherit_methods;
use windows_sys::Win32::UI::{
    Controls::{SetScrollInfo, WC_SCROLLBARW},
    WindowsAndMessaging::{
        GetParent, GetScrollInfo, SB_CTL, SBS_HORZ, SBS_VERT, SCROLLINFO, SIF_PAGE, SIF_POS,
        SIF_RANGE, SIF_TRACKPOS, WM_HSCROLL, WM_VSCROLL, WS_CHILD, WS_VISIBLE,
    },
};
use winio_handle::{AsContainer, AsWidget, BorrowedContainer};
use winio_primitive::{Orient, Point, Size};

use crate::{Result, Widget};

#[derive(Debug)]
struct ScrollBarImpl {
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl ScrollBarImpl {
    pub fn new(parent: impl AsContainer, style: u32) -> Result<Self> {
        let handle = Widget::new(
            WC_SCROLLBARW,
            WS_CHILD | style,
            0,
            parent.as_container().as_win32(),
        )?;
        Ok(Self { handle })
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

    fn info(&self, mask: u32) -> Result<SCROLLINFO> {
        let mut info: SCROLLINFO = unsafe { std::mem::zeroed() };
        info.cbSize = std::mem::size_of::<SCROLLINFO>() as _;
        info.fMask = mask;
        syscall!(
            BOOL,
            GetScrollInfo(self.handle.as_widget().as_win32(), SB_CTL, &mut info)
        )?;
        Ok(info)
    }

    fn set_info(&mut self, mask: u32, f: impl FnOnce(&mut SCROLLINFO)) {
        let mut info: SCROLLINFO = unsafe { std::mem::zeroed() };
        info.cbSize = std::mem::size_of::<SCROLLINFO>() as _;
        info.fMask = mask;
        f(&mut info);
        unsafe {
            SetScrollInfo(self.handle.as_widget().as_win32(), SB_CTL, &info, 1);
        }
    }

    pub fn minimum(&self) -> Result<usize> {
        let info = self.info(SIF_RANGE)?;
        Ok(info.nMin as _)
    }

    pub fn maximum(&self) -> Result<usize> {
        let info = self.info(SIF_RANGE)?;
        Ok(info.nMax as _)
    }

    pub fn set_minimum(&mut self, v: usize) -> Result<()> {
        let max = self.maximum()?;
        self.set_info(SIF_RANGE, |info| {
            info.nMin = v as _;
            info.nMax = max as _;
        });
        Ok(())
    }

    pub fn set_maximum(&mut self, v: usize) -> Result<()> {
        let min = self.minimum()?;
        self.set_info(SIF_RANGE, |info| {
            info.nMin = min as _;
            info.nMax = v as _;
        });
        Ok(())
    }

    pub fn page(&self) -> Result<usize> {
        let info = self.info(SIF_PAGE)?;
        Ok(info.nPage as _)
    }

    pub fn set_page(&mut self, v: usize) -> Result<()> {
        self.set_info(SIF_PAGE, |info| {
            info.nPage = v as _;
        });
        Ok(())
    }

    pub fn pos(&self) -> Result<usize> {
        let info = self.info(SIF_TRACKPOS)?;
        Ok(info.nTrackPos as _)
    }

    pub fn set_pos(&mut self, v: usize) -> Result<()> {
        self.set_info(SIF_POS, |info| {
            info.nPos = v as _;
        });
        Ok(())
    }
}

winio_handle::impl_as_widget!(ScrollBarImpl, handle);

#[derive(Debug)]
pub struct ScrollBar {
    handle: ScrollBarImpl,
    vertical: bool,
}

#[inherit_methods(from = "self.handle")]
impl ScrollBar {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let handle = ScrollBarImpl::new(&parent, WS_VISIBLE | SBS_HORZ as u32)?;
        Ok(Self {
            handle,
            vertical: false,
        })
    }

    fn recreate(&mut self, vertical: bool) -> Result<()> {
        let parent = unsafe { GetParent(self.handle.as_widget().as_win32()) };
        let mut new_handle = ScrollBarImpl::new(
            unsafe { BorrowedContainer::win32(parent) },
            if vertical {
                SBS_VERT as u32
            } else {
                SBS_HORZ as u32
            } | WS_VISIBLE,
        )?;
        new_handle.set_visible(self.handle.is_visible()?)?;
        new_handle.set_enabled(self.handle.is_enabled()?)?;
        new_handle.set_loc(self.handle.loc()?)?;
        new_handle.set_size(self.handle.size()?)?;
        new_handle.set_tooltip(self.handle.tooltip()?)?;
        new_handle.set_minimum(self.handle.minimum()?)?;
        new_handle.set_maximum(self.handle.maximum()?)?;
        new_handle.set_page(self.handle.page()?)?;
        new_handle.set_pos(self.handle.pos()?)?;
        self.handle = new_handle;
        Ok(())
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size> {
        let size = if self.vertical {
            Size::new(20.0, 0.0)
        } else {
            Size::new(0.0, 20.0)
        };
        Ok(size)
    }

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

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
            self.recreate(v)?;
            self.vertical = v;
        }
        Ok(())
    }

    pub fn minimum(&self) -> Result<usize>;

    pub fn set_minimum(&mut self, v: usize) -> Result<()>;

    pub fn maximum(&self) -> Result<usize>;

    pub fn set_maximum(&mut self, v: usize) -> Result<()>;

    pub fn page(&self) -> Result<usize>;

    pub fn set_page(&mut self, v: usize) -> Result<()>;

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

winio_handle::impl_as_widget!(ScrollBar, handle);
