use std::rc::Rc;

use gtk4::{
    Justification, WrapMode,
    glib::object::Cast,
    prelude::{TextBufferExt, TextViewExt},
};
use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{HAlign, Point, Size};

use crate::{GlobalRuntime, Result, ui::Widget};

#[derive(Debug)]
pub struct TextBox {
    on_changed: Rc<Callback<()>>,
    widget: gtk4::TextView,
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl TextBox {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let container = gtk4::ScrolledWindow::new();
        let widget = gtk4::TextView::new();
        container.set_child(Some(&widget));

        widget.set_wrap_mode(WrapMode::Char);
        let handle = Widget::new(parent, unsafe { container.clone().unsafe_cast() })?;

        let buffer = widget.buffer();
        let on_changed = Rc::new(Callback::new());
        buffer.connect_changed({
            let on_changed = on_changed.clone();
            move |_| {
                on_changed.signal::<GlobalRuntime>(());
            }
        });
        Ok(Self {
            on_changed,
            widget,
            handle,
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn min_size(&self) -> Result<Size> {
        let size = self.preferred_size()?;
        Ok(Size::new(size.width, 0.0))
    }

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, s: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn text(&self) -> Result<String> {
        let buffer = self.widget.buffer();
        Ok(buffer
            .text(&buffer.start_iter(), &buffer.end_iter(), true)
            .to_string())
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.widget.buffer().set_text(s.as_ref());
        self.handle.reset_preferred_size();
        Ok(())
    }

    pub fn halign(&self) -> Result<HAlign> {
        let align = self.widget.justification();
        let align = match align {
            Justification::Center => HAlign::Center,
            Justification::Right => HAlign::Right,
            Justification::Fill => HAlign::Stretch,
            _ => HAlign::Left,
        };
        Ok(align)
    }

    pub fn set_halign(&mut self, align: HAlign) -> Result<()> {
        let align = match align {
            HAlign::Left => Justification::Left,
            HAlign::Center => Justification::Center,
            HAlign::Right => Justification::Right,
            HAlign::Stretch => Justification::Fill,
        };
        self.widget.set_justification(align);
        Ok(())
    }

    pub fn is_readonly(&self) -> Result<bool> {
        Ok(!self.widget.is_editable())
    }

    pub fn set_readonly(&mut self, r: bool) -> Result<()> {
        self.widget.set_editable(!r);
        Ok(())
    }

    pub async fn wait_change(&self) {
        self.on_changed.wait().await
    }
}

winio_handle::impl_as_widget!(TextBox, handle);
