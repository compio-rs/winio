use std::rc::Rc;

use inherit_methods_macro::inherit_methods;
use send_wrapper::SendWrapper;
use windows::{
    Foundation::IReference,
    core::{HSTRING, Interface},
};
use winio_callback::Callback;
use winio_handle::AsWindow;
use winio_primitive::{Point, Size};
use winui3::Microsoft::UI::Xaml::{Controls as WUXC, RoutedEventHandler};

use crate::{GlobalRuntime, Widget, hstring_ref};

#[derive(Debug)]
pub struct Button {
    on_click: SendWrapper<Rc<Callback>>,
    handle: Widget,
    button: WUXC::Button,
}

#[inherit_methods(from = "self.handle")]
impl Button {
    pub fn new(parent: impl AsWindow) -> Self {
        let button = WUXC::Button::new().unwrap();
        let on_click = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_click = on_click.clone();
            button
                .Click(&RoutedEventHandler::new(move |_, _| {
                    on_click.signal::<GlobalRuntime>(());
                    Ok(())
                }))
                .unwrap();
        }
        Self {
            on_click,
            handle: Widget::new(parent, button.cast().unwrap()),
            button,
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

    pub fn text(&self) -> String {
        self.button
            .Content()
            .unwrap()
            .cast::<IReference<HSTRING>>()
            .unwrap()
            .Value()
            .unwrap()
            .to_string_lossy()
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        self.button
            .SetContent(&hstring_ref(&HSTRING::from(s.as_ref())))
            .unwrap();
    }

    pub async fn wait_click(&self) {
        self.on_click.wait().await
    }
}
