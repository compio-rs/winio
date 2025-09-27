use std::rc::Rc;

use gtk4::{
    glib::object::Cast,
    prelude::{AdjustmentExt, OrientableExt, ScaleExt},
};
use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Orient, Point, Size, TickPosition};

use crate::{GlobalRuntime, Widget};

#[derive(Debug)]
pub struct Slider {
    on_scroll: Rc<Callback>,
    adjustment: gtk4::Adjustment,
    widget: gtk4::Scale,
    handle: Widget,
    freq: usize,
    tick_pos: TickPosition,
}

#[inherit_methods(from = "self.handle")]
impl Slider {
    pub fn new(parent: impl AsContainer) -> Self {
        let adjustment = gtk4::Adjustment::new(0.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        let widget = gtk4::Scale::new(gtk4::Orientation::Horizontal, Some(&adjustment));
        let handle = Widget::new(parent, unsafe { widget.clone().unsafe_cast() });
        let on_scroll = Rc::new(Callback::new());
        adjustment.connect_value_changed({
            let on_scroll = on_scroll.clone();
            move |_| {
                on_scroll.signal::<GlobalRuntime>(());
            }
        });
        Self {
            on_scroll,
            adjustment,
            widget,
            handle,
            freq: 1,
            tick_pos: TickPosition::Both,
        }
    }

    fn reset_marks(&mut self) {
        self.widget.clear_marks();
        if self.freq == 0 {
            return;
        }
        let mut value = self.minimum();
        let max = self.maximum();
        while value <= max {
            if matches!(self.tick_pos, TickPosition::TopLeft | TickPosition::Both) {
                self.widget.add_mark(
                    value as _,
                    match self.orient() {
                        Orient::Horizontal => gtk4::PositionType::Top,
                        Orient::Vertical => gtk4::PositionType::Left,
                    },
                    None,
                );
            }
            if matches!(
                self.tick_pos,
                TickPosition::BottomRight | TickPosition::Both
            ) {
                self.widget.add_mark(
                    value as _,
                    match self.orient() {
                        Orient::Horizontal => gtk4::PositionType::Bottom,
                        Orient::Vertical => gtk4::PositionType::Right,
                    },
                    None,
                );
            }
            value += self.freq;
        }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn preferred_size(&self) -> Size;

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);

    pub fn tooltip(&self) -> String;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>);

    pub fn tick_pos(&self) -> TickPosition {
        self.tick_pos
    }

    pub fn set_tick_pos(&mut self, v: TickPosition) {
        if self.tick_pos != v {
            self.tick_pos = v;
            self.reset_marks();
        }
    }

    pub fn orient(&self) -> Orient {
        match self.widget.orientation() {
            gtk4::Orientation::Horizontal => Orient::Horizontal,
            gtk4::Orientation::Vertical => Orient::Vertical,
            _ => unreachable!(),
        }
    }

    pub fn set_orient(&mut self, v: Orient) {
        let v = match v {
            Orient::Horizontal => gtk4::Orientation::Horizontal,
            Orient::Vertical => gtk4::Orientation::Vertical,
        };
        self.widget.set_orientation(v);
    }

    pub fn minimum(&self) -> usize {
        self.adjustment.lower() as _
    }

    pub fn set_minimum(&mut self, v: usize) {
        if self.minimum() != v {
            self.adjustment.set_lower(v as _);
            self.reset_marks();
        }
    }

    pub fn maximum(&self) -> usize {
        self.adjustment.upper() as _
    }

    pub fn set_maximum(&mut self, v: usize) {
        if self.maximum() != v {
            self.adjustment.set_upper(v as _);
            self.reset_marks();
        }
    }

    pub fn freq(&self) -> usize {
        self.freq
    }

    pub fn set_freq(&mut self, v: usize) {
        if self.freq != v {
            self.freq = v;
            self.reset_marks();
        }
    }

    pub fn pos(&self) -> usize {
        self.adjustment.value() as _
    }

    pub fn set_pos(&mut self, v: usize) {
        self.adjustment.set_value(v as _);
    }

    pub async fn wait_change(&self) {
        self.on_scroll.wait().await;
    }
}

winio_handle::impl_as_widget!(Slider, handle);
