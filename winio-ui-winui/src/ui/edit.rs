use std::rc::Rc;

use inherit_methods_macro::inherit_methods;
use send_wrapper::SendWrapper;
use windows::core::{HSTRING, Interface};
use winio_callback::Callback;
use winio_handle::{AsContainer, AsWidget, BorrowedContainer};
use winio_primitive::{HAlign, Point, Size};
use winui3::Microsoft::UI::Xaml::{
    Controls::{self as MUXC, ScrollBarVisibility, ScrollViewer, TextChangedEventHandler},
    RoutedEventHandler, TextWrapping, Visibility,
};

use crate::{GlobalRuntime, Result, Widget, ui::Convertible};

#[derive(Debug)]
pub struct Edit {
    on_change: SendWrapper<Rc<Callback>>,
    handle: Widget,
    password: bool,
    halign: HAlign,
}

#[inherit_methods(from = "self.handle")]
impl Edit {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let text_box = MUXC::TextBox::new()?;
        let password_box = MUXC::PasswordBox::new()?;
        password_box.SetVisibility(Visibility::Collapsed)?;

        let on_change = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_change = on_change.clone();
            text_box.TextChanged(&TextChangedEventHandler::new(move |_, _| {
                on_change.signal::<GlobalRuntime>(());
                Ok(())
            }))?;
        }
        {
            let on_change = on_change.clone();
            password_box.PasswordChanged(&RoutedEventHandler::new(move |_, _| {
                on_change.signal::<GlobalRuntime>(());
                Ok(())
            }))?;
        }

        Ok(Self {
            on_change,
            handle: Widget::new(&parent, text_box.cast()?)?,
            password: false,
            halign: HAlign::Left,
        })
    }

    fn recreate(&mut self, password: bool) -> Result<()> {
        let parent = self.handle.parent()?;
        let mut widget = if password {
            let password_box = MUXC::PasswordBox::new()?;
            let text_box = self.handle.as_widget().as_winui().cast::<MUXC::TextBox>()?;
            password_box.SetPassword(&text_box.Text()?)?;
            Widget::new(BorrowedContainer::winui(&parent), password_box.cast()?)?
        } else {
            let text_box = MUXC::TextBox::new()?;
            let password_box = self
                .handle
                .as_widget()
                .as_winui()
                .cast::<MUXC::PasswordBox>()?;
            text_box.SetText(&password_box.Password()?)?;
            text_box.SetTextAlignment(self.halign.to_native())?;
            Widget::new(BorrowedContainer::winui(&parent), text_box.cast()?)?
        };
        widget.set_visible(self.handle.is_visible()?)?;
        widget.set_enabled(self.handle.is_enabled()?)?;
        widget.set_loc(self.handle.loc()?)?;
        widget.set_size(self.handle.size()?)?;
        widget.set_tooltip(self.handle.tooltip()?)?;
        self.handle = widget;
        Ok(())
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn text(&self) -> Result<String> {
        let text = if self.password {
            self.handle
                .as_widget()
                .as_winui()
                .cast::<MUXC::PasswordBox>()?
                .Password()?
                .to_string_lossy()
        } else {
            self.handle
                .as_widget()
                .as_winui()
                .cast::<MUXC::TextBox>()?
                .Text()?
                .to_string_lossy()
        };
        Ok(text)
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        let s = HSTRING::from(s.as_ref());
        if self.password {
            let password_box = self
                .handle
                .as_widget()
                .as_winui()
                .cast::<MUXC::PasswordBox>()?;
            password_box.SetPassword(&s)?;
        } else {
            let text_box = self.handle.as_widget().as_winui().cast::<MUXC::TextBox>()?;
            text_box.SetText(&s)?;
        }
        Ok(())
    }

    pub fn is_password(&self) -> Result<bool> {
        Ok(self.password)
    }

    pub fn set_password(&mut self, v: bool) -> Result<()> {
        if self.password != v {
            self.recreate(v)?;
            self.password = v;
        }
        Ok(())
    }

    pub fn halign(&self) -> Result<HAlign> {
        Ok(self.halign)
    }

    pub fn set_halign(&mut self, align: HAlign) -> Result<()> {
        self.halign = align;
        if let Ok(text_box) = self.handle.as_widget().as_winui().cast::<MUXC::TextBox>() {
            text_box.SetTextAlignment(align.to_native())?;
        }
        Ok(())
    }

    pub fn is_readonly(&self) -> Result<bool> {
        if self.password {
            Ok(false)
        } else {
            Ok(self
                .handle
                .as_widget()
                .as_winui()
                .cast::<MUXC::TextBox>()?
                .IsReadOnly()?)
        }
    }

    pub fn set_readonly(&mut self, v: bool) -> Result<()> {
        if !self.password {
            let text_box = self.handle.as_widget().as_winui().cast::<MUXC::TextBox>()?;
            text_box.SetIsReadOnly(v)?;
        }
        Ok(())
    }

    pub async fn wait_change(&self) {
        self.on_change.wait().await
    }
}

winio_handle::impl_as_widget!(Edit, handle);

#[derive(Debug)]
pub struct TextBox {
    on_change: SendWrapper<Rc<Callback>>,
    handle: Widget,
    text_box: MUXC::TextBox,
}

#[inherit_methods(from = "self.handle")]
impl TextBox {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let text_box = MUXC::TextBox::new()?;
        text_box.SetAcceptsReturn(true)?;
        text_box.SetTextWrapping(TextWrapping::Wrap)?;
        ScrollViewer::SetVerticalScrollBarVisibility2(&text_box, ScrollBarVisibility::Auto)?;
        let on_change = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_change = on_change.clone();
            text_box.TextChanged(&TextChangedEventHandler::new(move |_, _| {
                on_change.signal::<GlobalRuntime>(());
                Ok(())
            }))?;
        }
        Ok(Self {
            on_change,
            handle: Widget::new(parent, text_box.cast()?)?,
            text_box,
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn min_size(&self) -> Result<Size>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn text(&self) -> Result<String> {
        Ok(self.text_box.Text()?.to_string_lossy())
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.text_box.SetText(&HSTRING::from(s.as_ref()))?;
        Ok(())
    }

    pub fn halign(&self) -> Result<HAlign> {
        Ok(HAlign::from_native(self.text_box.TextAlignment()?))
    }

    pub fn set_halign(&mut self, align: HAlign) -> Result<()> {
        self.text_box.SetTextAlignment(align.to_native())?;
        Ok(())
    }

    pub fn is_readonly(&self) -> Result<bool> {
        self.text_box.IsReadOnly()
    }

    pub fn set_readonly(&mut self, v: bool) -> Result<()> {
        self.text_box.SetIsReadOnly(v)?;
        Ok(())
    }

    pub async fn wait_change(&self) {
        self.on_change.wait().await
    }
}

winio_handle::impl_as_widget!(TextBox, handle);
