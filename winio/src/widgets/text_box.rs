use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender, Prop, PropSource};
use winio_handle::BorrowedContainer;
use winio_primitive::{
    Enable, Failable, HAlign, Layoutable, Point, Rect, Size, TextWidget, ToolTip, Visible,
};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A simple multi-line text input box.
#[derive(Debug)]
pub struct TextBox {
    widget: sys::TextBox,
    text_prop: PropSource<String>,
}

impl Failable for TextBox {
    type Error = Error;
}

#[inherit_methods(from = "self.widget")]
impl ToolTip for TextBox {
    fn tooltip(&self) -> Result<String>;

    fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl TextWidget for TextBox {
    fn text(&self) -> Result<String>;

    fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        let s = s.as_ref();
        if s != self.text()? {
            self.widget.set_text(s)?;
            self.text_prop.notify(s.to_owned());
        }
        Ok(())
    }
}

#[inherit_methods(from = "self.widget")]
impl TextBox {
    /// The horizontal alignment.
    pub fn halign(&self) -> Result<HAlign>;

    /// Set the horizontal alignment.
    pub fn set_halign(&mut self, align: HAlign) -> Result<()>;

    /// If the text box is read-only.
    pub fn is_readonly(&self) -> Result<bool>;

    /// Set if the text box is read-only.
    pub fn set_readonly(&mut self, v: bool) -> Result<()>;

    /// Property for [`TextWidget::text`].
    pub fn text_prop(&mut self) -> Result<Prop<'_, String>> {
        Ok(self.text_prop.as_prop(self.text()?))
    }
}

#[inherit_methods(from = "self.widget")]
impl Visible for TextBox {
    fn is_visible(&self) -> Result<bool>;

    fn set_visible(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Enable for TextBox {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for TextBox {
    fn loc(&self) -> Result<Point>;

    fn set_loc(&mut self, p: Point) -> Result<()>;

    fn size(&self) -> Result<Size>;

    fn set_size(&mut self, s: Size) -> Result<()>;

    fn preferred_size(&self) -> Result<Size>;

    fn min_size(&self) -> Result<Size>;
}

/// Events of [`TextBox`].
#[derive(Debug)]
#[non_exhaustive]
pub enum TextBoxEvent {}

/// Messages of [`TextBox`].
#[derive(Debug)]
#[non_exhaustive]
pub enum TextBoxMessage {
    /// The text has been changed by user input.
    #[doc(hidden)]
    ChangeInput,
    /// Set the rect.
    SetRect(Rect),
    /// Set the halign.
    SetHAlign(HAlign),
    /// Set the readonly state.
    SetReadonly(bool),
    /// Set the enabled state.
    SetEnabled(bool),
    /// Set the visible state.
    SetVisible(bool),
    /// Set the tooltip.
    SetTooltip(String),
    /// Set the text.
    SetText(String),
}

impl Component for TextBox {
    type Error = Error;
    type Event = TextBoxEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = TextBoxMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::TextBox::new(init)?;
        let text_prop = PropSource::new();
        Ok(Self { widget, text_prop })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        loop {
            self.widget.wait_change().await;
            sender.post(TextBoxMessage::ChangeInput);
        }
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            TextBoxMessage::ChangeInput => {
                let text = self.widget.text()?;
                self.text_prop.notify(text);
                Ok(false)
            }
            TextBoxMessage::SetRect(rect) => {
                self.set_rect(rect)?;
                Ok(true)
            }
            TextBoxMessage::SetHAlign(halign) => {
                self.set_halign(halign)?;
                Ok(false)
            }
            TextBoxMessage::SetReadonly(readonly) => {
                self.set_readonly(readonly)?;
                Ok(false)
            }
            TextBoxMessage::SetEnabled(enabled) => {
                self.set_enabled(enabled)?;
                Ok(false)
            }
            TextBoxMessage::SetVisible(visible) => {
                self.set_visible(visible)?;
                Ok(true)
            }
            TextBoxMessage::SetTooltip(tooltip) => {
                self.set_tooltip(tooltip)?;
                Ok(false)
            }
            TextBoxMessage::SetText(text) => {
                self.set_text(text)?;
                Ok(true)
            }
        }
    }
}

winio_handle::impl_as_widget!(TextBox, widget);
