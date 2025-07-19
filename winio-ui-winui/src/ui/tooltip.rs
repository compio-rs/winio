use std::ops::{Deref, DerefMut};

use windows::core::HSTRING;
use winio_handle::AsWidget;
use winui3::Microsoft::UI::Xaml::Controls as MUXC;

pub struct ToolTip<T: AsWidget> {
    inner: T,
    #[allow(dead_code)]
    handle: MUXC::ToolTip,
    text: MUXC::TextBlock,
}

impl<T: AsWidget> ToolTip<T> {
    pub fn new(inner: T) -> Self {
        let handle = MUXC::ToolTip::new().unwrap();
        let text = MUXC::TextBlock::new().unwrap();
        handle.SetContent(&text).unwrap();
        for w in inner.iter_widgets() {
            MUXC::ToolTipService::SetToolTip(w.as_winui(), &handle).unwrap();
        }
        Self {
            inner,
            handle,
            text,
        }
    }

    pub fn tooltip(&self) -> String {
        self.text.Text().unwrap().to_string_lossy()
    }

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) {
        self.text.SetText(&HSTRING::from(s.as_ref())).unwrap()
    }
}

impl<T: AsWidget> Drop for ToolTip<T> {
    fn drop(&mut self) {
        for w in self.inner.iter_widgets() {
            MUXC::ToolTipService::SetToolTip(w.as_winui(), None).unwrap();
        }
    }
}

impl<T: AsWidget> Deref for ToolTip<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: AsWidget> DerefMut for ToolTip<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
