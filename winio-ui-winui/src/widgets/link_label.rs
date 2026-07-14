use std::rc::Rc;

use compio_log::info;
use inherit_methods_macro::inherit_methods;
use send_wrapper::SendWrapper;
use windows::{
    Foundation::Uri,
    core::{HSTRING, Interface},
};
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};
use winui3::Microsoft::UI::Xaml::{Controls as MUXC, RoutedEventHandler};

use crate::{GlobalRuntime, Result, Widget};

#[derive(Debug)]
pub struct LinkLabel {
    on_click: SendWrapper<Rc<Callback>>,
    handle: Widget,
    button: MUXC::HyperlinkButton,
    text: MUXC::TextBlock,
}

#[inherit_methods(from = "self.handle")]
impl LinkLabel {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let button = MUXC::HyperlinkButton::new()?;
        let on_click = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_click = on_click.clone();
            button.Click(&RoutedEventHandler::new(move |sender, _| {
                let button = sender.ok()?.cast::<MUXC::HyperlinkButton>()?;
                let uri = button.NavigateUri();
                if let Ok(_uri) = uri {
                    info!("Opening link: {}", _uri.ToString()?.to_string_lossy());
                } else {
                    on_click.signal::<GlobalRuntime>(());
                }
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

    pub fn uri(&self) -> Result<String> {
        if let Ok(uri) = self.button.NavigateUri() {
            Ok(uri.ToString()?.to_string_lossy())
        } else {
            Ok(String::new())
        }
    }

    pub fn set_uri(&mut self, s: impl AsRef<str>) -> Result<()> {
        let s = s.as_ref();
        if s.is_empty() {
            self.button.SetNavigateUri(None)?;
        } else {
            self.button
                .SetNavigateUri(&Uri::CreateUri(&HSTRING::from(s))?)?;
        }
        Ok(())
    }

    pub async fn wait_click(&self) {
        self.on_click.wait().await;
    }
}

winio_handle::impl_as_widget!(LinkLabel, handle);
