use std::rc::Rc;

use inherit_methods_macro::inherit_methods;
use send_wrapper::SendWrapper;
use windows::core::{HSTRING, Interface};
use winio_callback::Callback;
use winio_handle::{AsContainer, AsRawWidget, BorrowedContainer, RawContainer};
use winio_primitive::{HAlign, Point, Size};
use winui3::Microsoft::UI::Xaml::{
    Controls::{self as MUXC, ScrollBarVisibility, ScrollViewer, TextChangedEventHandler},
    RoutedEventHandler, TextWrapping, Visibility,
};

use crate::{GlobalRuntime, Widget, ui::Convertible};

#[derive(Debug)]
pub struct Edit {
    on_change: SendWrapper<Rc<Callback>>,
    handle: Widget,
    password: bool,
    halign: HAlign,
}

#[inherit_methods(from = "self.handle")]
impl Edit {
    pub fn new(parent: impl AsContainer) -> Self {
        let text_box = MUXC::TextBox::new().unwrap();
        let password_box = MUXC::PasswordBox::new().unwrap();
        password_box.SetVisibility(Visibility::Collapsed).unwrap();

        let on_change = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_change = on_change.clone();
            text_box
                .TextChanged(&TextChangedEventHandler::new(move |_, _| {
                    on_change.signal::<GlobalRuntime>(());
                    Ok(())
                }))
                .unwrap();
        }
        {
            let on_change = on_change.clone();
            password_box
                .PasswordChanged(&RoutedEventHandler::new(move |_, _| {
                    on_change.signal::<GlobalRuntime>(());
                    Ok(())
                }))
                .unwrap();
        }

        Self {
            on_change,
            handle: Widget::new(&parent, text_box.cast().unwrap()),
            password: false,
            halign: HAlign::Left,
        }
    }

    fn parent(&self) -> BorrowedContainer<'_> {
        let parent = self.handle.parent.clone();
        unsafe { BorrowedContainer::borrow_raw(RawContainer::WinUI(parent)) }
    }

    fn recreate(&mut self, password: bool) {
        let mut widget = if password {
            let password_box = MUXC::PasswordBox::new().unwrap();
            let text_box = self
                .handle
                .as_raw_widget()
                .as_winui()
                .cast::<MUXC::TextBox>()
                .unwrap();
            password_box.SetPassword(&text_box.Text().unwrap()).unwrap();
            Widget::new(self.parent(), password_box.cast().unwrap())
        } else {
            let text_box = MUXC::TextBox::new().unwrap();
            let password_box = self
                .handle
                .as_raw_widget()
                .as_winui()
                .cast::<MUXC::PasswordBox>()
                .unwrap();
            text_box.SetText(&password_box.Password().unwrap()).unwrap();
            text_box.SetTextAlignment(self.halign.to_native()).unwrap();
            Widget::new(self.parent(), text_box.cast().unwrap())
        };
        widget.set_visible(self.handle.is_visible());
        widget.set_enabled(self.handle.is_enabled());
        widget.set_loc(self.handle.loc());
        widget.set_size(self.handle.size());
        widget.set_tooltip(self.handle.tooltip());
        self.handle = widget;
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

    pub fn text(&self) -> String {
        if self.password {
            self.handle
                .as_raw_widget()
                .as_winui()
                .cast::<MUXC::PasswordBox>()
                .unwrap()
                .Password()
                .unwrap()
                .to_string_lossy()
        } else {
            self.handle
                .as_raw_widget()
                .as_winui()
                .cast::<MUXC::TextBox>()
                .unwrap()
                .Text()
                .unwrap()
                .to_string_lossy()
        }
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        let s = HSTRING::from(s.as_ref());
        if self.password {
            let password_box = self
                .handle
                .as_raw_widget()
                .as_winui()
                .cast::<MUXC::PasswordBox>()
                .unwrap();
            password_box.SetPassword(&s).unwrap();
        } else {
            let text_box = self
                .handle
                .as_raw_widget()
                .as_winui()
                .cast::<MUXC::TextBox>()
                .unwrap();
            text_box.SetText(&s).unwrap();
        }
    }

    pub fn is_password(&self) -> bool {
        self.password
    }

    pub fn set_password(&mut self, v: bool) {
        if self.password != v {
            self.recreate(v);
            self.password = v;
        }
    }

    pub fn halign(&self) -> HAlign {
        self.halign
    }

    pub fn set_halign(&mut self, align: HAlign) {
        self.halign = align;
        if let Ok(text_box) = self
            .handle
            .as_raw_widget()
            .as_winui()
            .cast::<MUXC::TextBox>()
        {
            text_box.SetTextAlignment(align.to_native()).unwrap();
        }
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
    pub fn new(parent: impl AsContainer) -> Self {
        let text_box = MUXC::TextBox::new().unwrap();
        text_box.SetAcceptsReturn(true).unwrap();
        text_box.SetTextWrapping(TextWrapping::Wrap).unwrap();
        ScrollViewer::SetVerticalScrollBarVisibility2(&text_box, ScrollBarVisibility::Auto)
            .unwrap();
        let on_change = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_change = on_change.clone();
            text_box
                .TextChanged(&TextChangedEventHandler::new(move |_, _| {
                    on_change.signal::<GlobalRuntime>(());
                    Ok(())
                }))
                .unwrap();
        }
        Self {
            on_change,
            handle: Widget::new(parent, text_box.cast().unwrap()),
            text_box,
        }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn preferred_size(&self) -> Size;

    pub fn min_size(&self) -> Size;

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);

    pub fn tooltip(&self) -> String;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>);

    pub fn text(&self) -> String {
        self.text_box.Text().unwrap().to_string_lossy()
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        self.text_box.SetText(&HSTRING::from(s.as_ref())).unwrap();
    }

    pub fn halign(&self) -> HAlign {
        HAlign::from_native(self.text_box.TextAlignment().unwrap())
    }

    pub fn set_halign(&mut self, align: HAlign) {
        self.text_box.SetTextAlignment(align.to_native()).unwrap();
    }

    pub async fn wait_change(&self) {
        self.on_change.wait().await
    }
}

winio_handle::impl_as_widget!(TextBox, handle);
