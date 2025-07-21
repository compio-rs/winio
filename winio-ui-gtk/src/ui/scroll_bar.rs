use std::rc::Rc;

use gtk4::{
    glib::object::Cast,
    prelude::{AdjustmentExt, OrientableExt},
};
use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::AsWindow;
use winio_primitive::{Orient, Point, Size};

use crate::{GlobalRuntime, Widget};

#[derive(Debug)]
pub struct ScrollBar {
    on_scroll: Rc<Callback>,
    adjustment: gtk4::Adjustment,
    widget: gtk4::Scrollbar,
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl ScrollBar {
    pub fn new(parent: impl AsWindow) -> Self {
        let adjustment = gtk4::Adjustment::new(0.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        let widget = gtk4::Scrollbar::new(gtk4::Orientation::Horizontal, Some(&adjustment));
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

    pub fn range(&self) -> (usize, usize) {
        let min = self.adjustment.lower();
        let max = self.adjustment.upper();
        (min as _, max as _)
    }

    pub fn set_range(&mut self, min: usize, max: usize) {
        self.adjustment.set_lower(min as _);
        self.adjustment.set_upper(max as _);
    }

    pub fn page(&self) -> usize {
        self.adjustment.page_size() as _
    }

    pub fn set_page(&mut self, v: usize) {
        self.adjustment.set_page_size(v as _);
    }

    pub fn pos(&self) -> usize {
        self.adjustment.value() as _
    }

    pub fn set_pos(&mut self, v: usize) {
        self.adjustment.set_value(v as _);
    }

    pub async fn wait_change(&self) {
        self.on_scroll.wait().await
    }
}

winio_handle::impl_as_widget!(ScrollBar, handle);
