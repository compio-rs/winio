use std::rc::Rc;

use inherit_methods_macro::inherit_methods;
use send_wrapper::SendWrapper;
use windows::core::Interface;
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Orient, Point, Size, TickPosition};
use winui3::Microsoft::UI::Xaml::Controls::{
    self as MUXC,
    Primitives::{RangeBaseValueChangedEventHandler, TickPlacement},
};

use crate::{GlobalRuntime, Widget, ui::Convertible};

#[derive(Debug)]
pub struct Slider {
    on_scroll: SendWrapper<Rc<Callback>>,
    handle: Widget,
    bar: MUXC::Slider,
}

#[inherit_methods(from = "self.handle")]
impl Slider {
    pub fn new(parent: impl AsContainer) -> Self {
        let bar = MUXC::Slider::new().unwrap();
        bar.SetTickPlacement(TickPlacement::Outside).unwrap();
        bar.SetIsThumbToolTipEnabled(false).unwrap();
        let on_scroll = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_scroll = on_scroll.clone();
            bar.ValueChanged(&RangeBaseValueChangedEventHandler::new(move |_, _| {
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

    pub fn tooltip(&self) -> String;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>);

    pub fn tick_pos(&self) -> TickPosition {
        match self.bar.TickPlacement().unwrap() {
            TickPlacement::None => TickPosition::None,
            TickPlacement::TopLeft => TickPosition::TopLeft,
            TickPlacement::BottomRight => TickPosition::BottomRight,
            _ => TickPosition::Both,
        }
    }

    pub fn set_tick_pos(&mut self, v: TickPosition) {
        let v = match v {
            TickPosition::None => TickPlacement::None,
            TickPosition::TopLeft => TickPlacement::TopLeft,
            TickPosition::BottomRight => TickPlacement::BottomRight,
            TickPosition::Both => TickPlacement::Outside,
        };
        self.bar.SetTickPlacement(v).unwrap();
    }

    pub fn orient(&self) -> Orient {
        Orient::from_native(self.bar.Orientation().unwrap())
    }

    pub fn set_orient(&mut self, v: Orient) {
        self.bar.SetOrientation(v.to_native()).unwrap();
    }

    pub fn minimum(&self) -> usize {
        self.bar.Minimum().unwrap() as usize
    }

    pub fn set_minimum(&mut self, v: usize) {
        self.bar.SetMinimum(v as _).unwrap();
    }

    pub fn maximum(&self) -> usize {
        self.bar.Maximum().unwrap() as usize
    }

    pub fn set_maximum(&mut self, v: usize) {
        self.bar.SetMaximum(v as _).unwrap()
    }

    pub fn freq(&self) -> usize {
        self.bar.TickFrequency().unwrap() as _
    }

    pub fn set_freq(&mut self, v: usize) {
        self.bar.SetTickFrequency(v as _).unwrap();
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

winio_handle::impl_as_widget!(Slider, handle);
