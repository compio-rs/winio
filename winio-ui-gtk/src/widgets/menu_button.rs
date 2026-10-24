use gtk4::{gio, glib::object::Cast, prelude::*};
use inherit_methods_macro::inherit_methods;
use winio_handle::{AsContainer, AsMenuContainer, AsWidget, BorrowedMenuContainer};
use winio_primitive::{Point, Size};

use crate::{Image, Result, widgets::Widget};

#[derive(Debug)]
pub struct MenuButton {
    handle: Widget,
    image: gtk4::Image,
    label: gtk4::Label,
    menu: gio::Menu,
}

#[inherit_methods(from = "self.handle")]
impl MenuButton {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let widget = gtk4::MenuButton::new();
        widget.set_always_show_arrow(true);

        let content = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        content.set_halign(gtk4::Align::Center);
        content.set_valign(gtk4::Align::Center);
        content.set_hexpand(false);
        content.set_vexpand(false);
        let image = gtk4::Image::new();
        image.set_visible(false);
        let label = gtk4::Label::new(None);
        label.set_visible(false);
        content.append(&image);
        content.append(&label);
        widget.set_child(Some(&content));

        let menu = gio::Menu::new();
        let popover = gtk4::PopoverMenu::from_model(Some(&menu));
        widget.set_popover(Some(&popover));
        crate::menu::register_menu_popover(&popover);

        let handle = Widget::new(parent, unsafe { widget.clone().unsafe_cast() })?;
        let this = Self {
            handle,
            image,
            label,
            menu,
        };
        this.register_actions();
        Ok(this)
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, s: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn text(&self) -> Result<String> {
        Ok(self.label.text().to_string())
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        let s = s.as_ref();
        self.label.set_text(s);
        self.label.set_visible(!s.is_empty());
        self.handle.reset_preferred_size();
        Ok(())
    }

    pub fn set_icon(&mut self, icon: Option<&Image>) -> Result<()> {
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
        self.handle.reset_preferred_size();
        Ok(())
    }

    pub async fn start(&self) -> ! {
        std::future::pending().await
    }
}

impl MenuButton {
    fn register_actions(&self) {
        let widget = self.handle.as_widget().to_gtk();
        if let Some(root) = widget.root()
            && let Some(window) = root.downcast_ref::<gtk4::Window>()
        {
            crate::menu::register_actions(window);
        }
    }
}

impl AsMenuContainer for MenuButton {
    fn as_menu_container(&self) -> BorrowedMenuContainer<'_> {
        BorrowedMenuContainer::gtk(&self.menu)
    }
}

winio_handle::impl_as_widget!(MenuButton, handle);
