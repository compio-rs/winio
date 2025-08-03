use std::rc::Rc;

use inherit_methods_macro::inherit_methods;
use send_wrapper::SendWrapper;
use windows::core::{HSTRING, Interface};
use winio_callback::Callback;
use winio_handle::{AsRawWidget, AsWindow, RawWidget};
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
    phandle: Widget,
    text_box: MUXC::TextBox,
    password_box: MUXC::PasswordBox,
    password: bool,
}

impl Edit {
    pub fn new(parent: impl AsWindow) -> Self {
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
            handle: Widget::new(parent.as_window(), text_box.cast().unwrap()),
            phandle: Widget::new(parent.as_window(), password_box.cast().unwrap()),
            text_box,
            password_box,
            password: false,
        }
    }

    pub fn is_visible(&self) -> bool {
        if self.password {
            &self.phandle
        } else {
            &self.handle
        }
        .is_visible()
    }

    pub fn set_visible(&mut self, v: bool) {
        if self.password {
            &mut self.phandle
        } else {
            &mut self.handle
        }
        .set_visible(v);
    }

    pub fn is_enabled(&self) -> bool {
        self.handle.is_enabled()
    }

    pub fn set_enabled(&mut self, v: bool) {
        self.handle.set_enabled(v);
        self.phandle.set_enabled(v);
    }

    pub fn preferred_size(&self) -> Size {
        self.phandle
            .preferred_size()
            .max(self.handle.preferred_size())
    }

    pub fn loc(&self) -> Point {
        self.handle.loc()
    }

    pub fn set_loc(&mut self, p: Point) {
        self.handle.set_loc(p);
        self.phandle.set_loc(p);
    }

    pub fn size(&self) -> Size {
        self.handle.size()
    }

    pub fn set_size(&mut self, v: Size) {
        self.handle.set_size(v);
        self.phandle.set_size(v);
    }

    pub fn text(&self) -> String {
        if self.password {
            self.password_box.Password().unwrap().to_string_lossy()
        } else {
            self.text_box.Text().unwrap().to_string_lossy()
        }
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        let s = HSTRING::from(s.as_ref());
        if self.password {
            self.password_box.SetPassword(&s).unwrap();
        } else {
            self.text_box.SetText(&s).unwrap();
        }
    }

    pub fn is_password(&self) -> bool {
        self.password
    }

    pub fn set_password(&mut self, v: bool) {
        if self.password != v {
            if v {
                self.password_box
                    .SetPassword(&self.text_box.Text().unwrap())
                    .unwrap();
                self.phandle.set_visible(self.handle.is_visible());
                self.handle.set_visible(false);
            } else {
                self.text_box
                    .SetText(&self.password_box.Password().unwrap())
                    .unwrap();
                self.handle.set_visible(self.phandle.is_visible());
                self.phandle.set_visible(false);
            }
            self.password = v;
        }
    }

    pub fn halign(&self) -> HAlign {
        HAlign::from_native(self.text_box.TextAlignment().unwrap())
    }

    pub fn set_halign(&mut self, align: HAlign) {
        let align = align.to_native();
        self.text_box.SetTextAlignment(align).unwrap();
    }

    pub async fn wait_change(&self) {
        self.on_change.wait().await
    }
}

impl AsRawWidget for Edit {
    fn as_raw_widget(&self) -> RawWidget {
        if self.password {
            &self.phandle
        } else {
            &self.handle
        }
        .as_raw_widget()
    }

    fn iter_raw_widgets(&self) -> impl Iterator<Item = RawWidget> {
        [self.handle.as_raw_widget(), self.phandle.as_raw_widget()].into_iter()
    }
}

winio_handle::impl_as_widget!(Edit);

#[derive(Debug)]
pub struct TextBox {
    on_change: SendWrapper<Rc<Callback>>,
    handle: Widget,
    text_box: MUXC::TextBox,
}

#[inherit_methods(from = "self.handle")]
impl TextBox {
    pub fn new(parent: impl AsWindow) -> Self {
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
