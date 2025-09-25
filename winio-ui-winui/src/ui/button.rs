use std::rc::Rc;

use inherit_methods_macro::inherit_methods;
use send_wrapper::SendWrapper;
use windows::core::{HSTRING, Interface};
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};
use winui3::Microsoft::UI::Xaml::{Controls as MUXC, RoutedEventHandler};

use crate::{GlobalRuntime, Widget};

#[derive(Debug)]
pub struct Button {
    on_click: SendWrapper<Rc<Callback>>,
    handle: Widget,
    text: MUXC::TextBlock,
}

#[inherit_methods(from = "self.handle")]
impl Button {
    pub fn new(parent: impl AsContainer) -> Self {
        let button = MUXC::Button::new().unwrap();
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
        let text = MUXC::TextBlock::new().unwrap();
        button.SetContent(&text).unwrap();
        Self {
            on_click,
            handle: Widget::new(parent, button.cast().unwrap()),
            text,
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

    pub fn text(&self) -> String {
        self.text.Text().unwrap().to_string_lossy()
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        self.text.SetText(&HSTRING::from(s.as_ref())).unwrap()
    }

    pub async fn wait_click(&self) {
        self.on_click.wait().await
    }
}

winio_handle::impl_as_widget!(Button, handle);
