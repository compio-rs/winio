//! Android window widget, based on JNI and native FrameLayout

use {
    winio_handle::{AsRawWindow, AsWindow, BorrowedWindow, RawWindow},
    winio_primitive::{Point, Size},
};

#[derive(Debug)]
pub struct Window {
    inner: RawWindow,
}

//noinspection SpellCheckingInspection
impl Window {
    pub fn new<W>(_parent: Option<W>) -> Self
    where
        W: AsWindow,
    {
        todo!()
    }

    pub async fn wait_close(&self) {
        std::future::pending().await
    }

    pub async fn wait_move(&self) {
        std::future::pending().await
    }

    pub async fn wait_size(&self) {
        std::future::pending().await
    }

    pub fn text(&self) -> String {
        todo!()
    }

    pub fn set_text<S>(&mut self, _title: S)
    where
        S: AsRef<str>,
    {
        todo!()
    }

    pub fn client_size(&self) -> Size {
        todo!()
    }

    pub fn is_visible(&self) -> bool {
        todo!()
    }

    pub fn set_visible(&mut self, _visible: bool) {
        todo!()
    }

    pub fn loc(&self) -> Point {
        todo!()
    }

    pub fn set_loc(&mut self, _p: Point) {
        todo!()
    }

    pub fn size(&self) -> Size {
        todo!()
    }

    pub fn set_size(&mut self, _size: Size) {
        todo!()
    }
}

impl AsRawWindow for Window {
    fn as_raw_window(&self) -> RawWindow {
        // Return pointer or handle to FrameLayout
        self.inner.clone()
    }
}

impl AsWindow for Window {
    fn as_window(&self) -> BorrowedWindow<'_> {
        unsafe { BorrowedWindow::borrow_raw(self.inner.clone()) }
    }
}
