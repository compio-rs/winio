use std::rc::Rc;

use inherit_methods_macro::inherit_methods;
use send_wrapper::SendWrapper;
use windows::core::Interface;
use winio_callback::Callback;
use winio_handle::AsWindow;
use winio_primitive::{Orient, Point, Size};
use winui3::Microsoft::UI::Xaml::Controls::{
    self as MUXC,
    Primitives::{ScrollEventHandler, ScrollingIndicatorMode},
};

use crate::{GlobalRuntime, Widget, ui::Convertible};

#[derive(Debug)]
pub struct ScrollBar {
    on_scroll: SendWrapper<Rc<Callback>>,
    handle: Widget,
    bar: MUXC::Primitives::ScrollBar,
}

#[inherit_methods(from = "self.handle")]
impl ScrollBar {
    pub fn new(parent: impl AsWindow) -> Self {
        let bar = MUXC::Primitives::ScrollBar::new().unwrap();
        bar.SetIndicatorMode(ScrollingIndicatorMode::MouseIndicator)
            .unwrap();
        let on_scroll = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_scroll = on_scroll.clone();
            bar.Scroll(&ScrollEventHandler::new(move |_, _| {
                on_scroll.signal::<GlobalRuntime>(());
                Ok(())
            }))
            .unwrap();
        }
        let handle = Widget::new(parent, bar.cast().unwrap());
        Self {
            on_scroll,
            handle,
            bar,
        }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn preferred_size(&self) -> Size {
        let size = self.handle.preferred_size();
        match self.orient() {
            Orient::Horizontal => Size::new(0.0, size.height),
            Orient::Vertical => Size::new(size.width, 0.0),
        }
    }

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);

    pub fn orient(&self) -> Orient {
        Orient::from_native(self.bar.Orientation().unwrap())
    }

    pub fn set_orient(&mut self, v: Orient) {
        self.bar.SetOrientation(v.to_native()).unwrap();
    }

    pub fn range(&self) -> (usize, usize) {
        let min = self.bar.Minimum().unwrap();
        let max = self.bar.Maximum().unwrap();
        (min as _, max as usize + self.page())
    }

    pub fn set_range(&mut self, min: usize, max: usize) {
        self.bar.SetMinimum(min as _).unwrap();
        self.bar
            .SetMaximum(max.saturating_sub(self.page()) as _)
            .unwrap();
    }

    pub fn page(&self) -> usize {
        self.bar.ViewportSize().unwrap() as _
    }

    pub fn set_page(&mut self, v: usize) {
        self.bar.SetViewportSize(v as _).unwrap();
    }

    pub fn pos(&self) -> usize {
        self.bar.Value().unwrap() as _
    }

    pub fn set_pos(&mut self, v: usize) {
        self.bar.SetValue(v as _).unwrap();
    }

    pub async fn wait_change(&self) {
        self.on_scroll.wait().await
    }
}

winio_handle::impl_as_widget!(ScrollBar, handle);
