use std::rc::Rc;

use gtk4::{gio, prelude::*};
use winio_callback::Callback;
use winio_handle::{AsToolBarItem, BorrowedToolBarItem};

use crate::{GlobalRuntime, Image, Result, tool_bar::ToolBar};

#[derive(Debug)]
pub(crate) struct ToolBarItem {
    widget: gtk4::Widget,
    image: gtk4::Image,
    label: gtk4::Label,
    on_click: Rc<Callback<()>>,
}

impl ToolBarItem {
    pub(crate) fn new_button(parent: &ToolBar) -> Result<Self> {
        let widget = gtk4::Button::new();
        widget.set_has_frame(false);
        let on_click = Rc::new(Callback::new());
        widget.connect_clicked({
            let on_click = on_click.clone();
            move |_| {
                on_click.signal::<GlobalRuntime>(());
            }
        });
        let (content, image, label) = Self::new_content();
        widget.set_child(Some(&content));
        Self::finish(parent, widget.upcast(), image, label, on_click)
    }

    pub(crate) fn new_toggle(parent: &ToolBar) -> Result<Self> {
        let widget = gtk4::ToggleButton::new();
        widget.set_has_frame(false);
        let on_click = Rc::new(Callback::new());
        widget.connect_clicked({
            let on_click = on_click.clone();
            move |_| {
                on_click.signal::<GlobalRuntime>(());
            }
        });
        let (content, image, label) = Self::new_content();
        widget.set_child(Some(&content));
        Self::finish(parent, widget.upcast(), image, label, on_click)
    }

    pub(crate) fn new_menu_button(parent: &ToolBar, menu: &gio::Menu) -> Result<Self> {
        let widget = gtk4::MenuButton::new();
        widget.set_has_frame(false);
        widget.set_always_show_arrow(true);
        let popover = gtk4::PopoverMenu::from_model(Some(menu));
        widget.set_popover(Some(&popover));
        crate::menu::register_menu_popover(&popover);
        let on_click = Rc::new(Callback::new());
        widget.connect_activate({
            let on_click = on_click.clone();
            move |_| {
                on_click.signal::<GlobalRuntime>(());
            }
        });
        let (content, image, label) = Self::new_content();
        widget.set_child(Some(&content));
        Self::finish(parent, widget.upcast(), image, label, on_click)
    }

    fn new_content() -> (gtk4::Box, gtk4::Image, gtk4::Label) {
        let content = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        content.set_halign(gtk4::Align::Center);
        content.set_valign(gtk4::Align::Center);
        content.set_hexpand(false);
        content.set_vexpand(false);
        let image = gtk4::Image::new();
        image.set_pixel_size(24);
        image.set_visible(false);
        let label = gtk4::Label::new(None);
        label.set_visible(false);
        content.append(&image);
        content.append(&label);
        (content, image, label)
    }

    fn finish(
        parent: &ToolBar,
        widget: gtk4::Widget,
        image: gtk4::Image,
        label: gtk4::Label,
        on_click: Rc<Callback<()>>,
    ) -> Result<Self> {
        parent.toolbar().append(&widget);
        Ok(Self {
            widget,
            image,
            label,
            on_click,
        })
    }

    pub(crate) fn toggle(&self) -> &gtk4::ToggleButton {
        self.widget
            .downcast_ref::<gtk4::ToggleButton>()
            .expect("tool bar item is a toggle button")
    }

    pub(crate) fn is_enabled(&self) -> Result<bool> {
        Ok(self.widget.is_sensitive())
    }

    pub(crate) fn set_enabled(&mut self, enabled: bool) -> Result<()> {
        self.widget.set_sensitive(enabled);
        Ok(())
    }

    pub(crate) fn text(&self) -> Result<String> {
        Ok(self.label.text().to_string())
    }

    pub(crate) fn set_text(&mut self, text: impl AsRef<str>) -> Result<()> {
        let text = text.as_ref();
        self.label.set_text(text);
        self.label.set_visible(!text.is_empty());
        Ok(())
    }

    pub(crate) fn set_icon(&mut self, icon: Option<&Image>) -> Result<()> {
        match icon {
            Some(icon) => {
                self.image.set_paintable(Some(icon.texture()));
                self.image.set_visible(true);
            }
            None => {
                self.image.set_paintable(gtk4::gdk::Paintable::NONE);
                self.image.set_visible(false);
            }
        }
        Ok(())
    }

    pub(crate) async fn wait_click(&self) {
        self.on_click.wait().await;
    }
}

impl Drop for ToolBarItem {
    fn drop(&mut self) {
        self.widget.unparent();
    }
}

impl AsToolBarItem for ToolBarItem {
    fn as_tool_bar_item(&self) -> BorrowedToolBarItem<'_> {
        BorrowedToolBarItem::gtk(&self.widget)
    }
}
