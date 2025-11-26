use std::rc::Rc;

use inherit_methods_macro::inherit_methods;
use send_wrapper::SendWrapper;
use windows::core::{HSTRING, Interface};
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};
use winui3::Microsoft::UI::Xaml::{Controls as MUXC, RoutedEventHandler};

use crate::{GlobalRuntime, Result, Widget, ui::ToIReference};

#[derive(Debug)]
pub struct CheckBox {
    on_click: SendWrapper<Rc<Callback>>,
    handle: Widget,
    button: MUXC::CheckBox,
    text: MUXC::TextBlock,
}

#[inherit_methods(from = "self.handle")]
impl CheckBox {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let button = MUXC::CheckBox::new()?;
        let on_click = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_click = on_click.clone();
            button.Click(&RoutedEventHandler::new(move |_, _| {
                on_click.signal::<GlobalRuntime>(());
                Ok(())
            }))?;
        }
        let text = MUXC::TextBlock::new()?;
        button.SetContent(&text)?;
        Ok(Self {
            on_click,
            handle: Widget::new(parent, button.cast()?)?,
            button,
            text,
        })
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
        Ok(self.text.Text()?.to_string_lossy())
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.text.SetText(&HSTRING::from(s.as_ref()))?;
        Ok(())
    }

    pub fn is_checked(&self) -> Result<bool> {
        Ok(self.button.IsChecked()?.GetBoolean()?)
    }

    pub fn set_checked(&mut self, v: bool) -> Result<()> {
        self.button.SetIsChecked(&v.to_reference()?)?;
        Ok(())
    }

    pub async fn wait_click(&self) {
        self.on_click.wait().await
    }
}

winio_handle::impl_as_widget!(CheckBox, handle);
