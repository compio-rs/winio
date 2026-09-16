use std::rc::Rc;

use inherit_methods_macro::inherit_methods;
use send_wrapper::SendWrapper;
use windows_core::{HSTRING, Interface};
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};
use winui3::Microsoft::UI::Xaml::{
    Controls as MUXC, Controls::Orientation, Media::ImageSource, Visibility,
};

use crate::{GlobalRuntime, Image, Result, Widget};

const DISABLED_OPACITY: f64 = 0.4;

#[derive(Debug)]
pub struct Button {
    on_click: SendWrapper<Rc<Callback>>,
    handle: Widget,
    image: MUXC::Image,
    text: MUXC::TextBlock,
}

#[inherit_methods(from = "self.handle")]
impl Button {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let button = MUXC::Button::new()?;
        let content = MUXC::StackPanel::new()?;
        content.SetOrientation(Orientation::Horizontal)?;
        content.SetSpacing(6.0)?;
        let image = MUXC::Image::new()?;
        let icon_size = button.FontSize()?;
        image.SetWidth(icon_size)?;
        image.SetHeight(icon_size)?;
        image.SetVisibility(Visibility::Collapsed)?;
        let text = MUXC::TextBlock::new()?;
        text.SetVisibility(Visibility::Collapsed)?;
        content.Children()?.Append(&image)?;
        content.Children()?.Append(&text)?;
        button.SetContent(&content)?;
        let on_click = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_click = on_click.clone();
            button
                .Click(move |_, _| {
                    on_click.signal::<GlobalRuntime>(());
                    Ok(())
                })?
                .forget();
        }
        Ok(Self {
            on_click,
            handle: Widget::new(parent, button.cast()?)?,
            image,
            text,
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()> {
        self.handle.set_enabled(v)?;
        self.update_icon_opacity()?;
        Ok(())
    }

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
        let s = s.as_ref();
        self.text.SetText(&HSTRING::from(s))?;
        self.text.SetVisibility(if s.is_empty() {
            Visibility::Collapsed
        } else {
            Visibility::Visible
        })?;
        Ok(())
    }

    pub fn set_icon(&mut self, icon: Option<&Image>) -> Result<()> {
        match icon {
            Some(icon) => {
                self.image.SetSource(icon.as_ref())?;
                self.image.SetVisibility(Visibility::Visible)?;
            }
            None => {
                self.image.SetSource(None::<&ImageSource>)?;
                self.image.SetVisibility(Visibility::Collapsed)?;
            }
        }
        self.update_icon_opacity()?;
        Ok(())
    }

    fn update_icon_opacity(&self) -> Result<()> {
        self.image.SetOpacity(if self.handle.is_enabled()? {
            1.0
        } else {
            DISABLED_OPACITY
        })?;
        Ok(())
    }

    pub async fn wait_click(&self) {
        self.on_click.wait().await
    }
}

winio_handle::impl_as_widget!(Button, handle);
