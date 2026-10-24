use inherit_methods_macro::inherit_methods;
use winio_handle::{AsContainer, AsMenuContainer, BorrowedMenuContainer};
use winio_primitive::{Point, Size};

use crate::{
    Image, Result,
    menu::QMenu,
    widgets::{Widget, impl_static_cast},
};

#[derive(Debug)]
pub struct MenuButton {
    widget: Widget<ffi::QToolButton>,
    menu: *mut QMenu,
}

#[inherit_methods(from = "self.widget")]
impl MenuButton {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let widget = unsafe { ffi::new_menu_button(parent.as_container().as_qt()) }?;
        let mut widget = Widget::new(widget)?;
        widget.set_visible(true)?;
        let menu = widget.as_ref().menu()?;
        Ok(Self { widget, menu })
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
        Ok(self.widget.as_ref().text()?.try_into()?)
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.widget.pin_mut().setText(&s.as_ref().try_into()?)?;
        Ok(())
    }

    pub fn set_icon(&mut self, icon: Option<&Image>) -> Result<()> {
        match icon {
            Some(icon) => ffi::menu_button_set_icon(self.widget.pin_mut(), icon.as_qimage())?,
            None => ffi::menu_button_clear_icon(self.widget.pin_mut())?,
        }
        Ok(())
    }

    pub async fn start(&self) -> ! {
        std::future::pending().await
    }
}

impl AsMenuContainer for MenuButton {
    fn as_menu_container(&self) -> BorrowedMenuContainer<'_> {
        unsafe { BorrowedMenuContainer::qt(self.menu) }
    }
}

winio_handle::impl_as_widget!(MenuButton, widget);

impl_static_cast!(ffi::QToolButton, ffi::QWidget);

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/widgets/menu_button.hpp");

        type QWidget = crate::widgets::QWidget;
        type QString = crate::common::QString;
        type QImage = crate::platform::QImage;
        type QToolButton;
        type QMenu = crate::menu::QMenu;

        unsafe fn new_menu_button(parent: *mut QWidget) -> Result<UniquePtr<QToolButton>>;

        fn menu(self: &QToolButton) -> Result<*mut QMenu>;
        fn text(self: &QToolButton) -> Result<QString>;
        fn setText(self: Pin<&mut QToolButton>, s: &QString) -> Result<()>;

        fn menu_button_set_icon(w: Pin<&mut QToolButton>, icon: &QImage) -> Result<()>;
        fn menu_button_clear_icon(w: Pin<&mut QToolButton>) -> Result<()>;
    }
}
