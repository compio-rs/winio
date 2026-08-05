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

/// A simple single-line text input box.
#[derive(Debug)]
pub struct Edit {
    widget: sys::Edit,
    text_prop: PropSource<String>,
}

impl Failable for Edit {
    type Error = Error;
}

#[inherit_methods(from = "self.widget")]
impl ToolTip for Edit {
    fn tooltip(&self) -> Result<String>;

    fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl TextWidget for Edit {
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
impl Edit {
    /// If the text input is password.
    pub fn is_password(&self) -> Result<bool>;

    /// Set if the text input is password.
    pub fn set_password(&mut self, v: bool) -> Result<()>;

    /// The horizontal alignment.
    pub fn halign(&self) -> Result<HAlign>;

    /// Set the horizontal alignment.
    pub fn set_halign(&mut self, align: HAlign) -> Result<()>;

    /// If the text input is read-only.
    /// A password edit cannot be read-only.
    pub fn is_readonly(&self) -> Result<bool>;

    /// Set if the text input is read-only.
    /// A password edit cannot be read-only.
    pub fn set_readonly(&mut self, v: bool) -> Result<()>;

    /// Property for [`TextWidget::text`].
    pub fn text_prop(&mut self) -> Result<Prop<'_, String>> {
        Ok(self.text_prop.as_prop(self.text()?))
    }
}

#[inherit_methods(from = "self.widget")]
impl Visible for Edit {
    fn is_visible(&self) -> Result<bool>;

    fn set_visible(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Enable for Edit {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for Edit {
    fn loc(&self) -> Result<Point>;

    fn set_loc(&mut self, p: Point) -> Result<()>;

    fn size(&self) -> Result<Size>;

    fn set_size(&mut self, s: Size) -> Result<()>;

    fn preferred_size(&self) -> Result<Size>;
}

/// Events of [`Edit`].
#[derive(Debug)]
#[non_exhaustive]
pub enum EditEvent {
    /// The text has been changed.
    Change,
}

/// Messages of [`Edit`].
#[derive(Debug)]
#[non_exhaustive]
pub enum EditMessage {
    /// No operation.
    Noop,
    /// The input has been changed.
    ChangeInput,
    /// Set the rect.
    SetRect(Rect),
    /// Set the enabled state.
    SetEnabled(bool),
    /// Set the visible state.
    SetVisible(bool),
    /// Set the tooltip.
    SetTooltip(String),
    /// Set the text.
    SetText(String),
    /// Set the password state.
    SetPassword(bool),
    /// Set the halign.
    SetHAlign(HAlign),
    /// Set the readonly state.
    SetReadonly(bool),
}

impl Component for Edit {
    type Error = Error;
    type Event = EditEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = EditMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::Edit::new(init)?;
        let text_prop = PropSource::new();
        Ok(Self { widget, text_prop })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        loop {
            self.widget.wait_change().await;
            sender.post(EditMessage::ChangeInput);
        }
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            EditMessage::Noop => Ok(false),
            EditMessage::ChangeInput => {
                let text = self.widget.text()?;
                self.text_prop.notify(text);
                sender.output(EditEvent::Change);
                Ok(false)
            }
            EditMessage::SetRect(rect) => {
                self.set_rect(rect)?;
                Ok(true)
            }
            EditMessage::SetEnabled(enabled) => {
                self.set_enabled(enabled)?;
                Ok(false)
            }
            EditMessage::SetVisible(visible) => {
                self.set_visible(visible)?;
                Ok(true)
            }
            EditMessage::SetTooltip(tooltip) => {
                self.set_tooltip(tooltip)?;
                Ok(false)
            }
            EditMessage::SetText(text) => {
                self.set_text(text)?;
                Ok(true)
            }
            EditMessage::SetPassword(password) => {
                self.set_password(password)?;
                Ok(true)
            }
            EditMessage::SetHAlign(halign) => {
                self.set_halign(halign)?;
                Ok(false)
            }
            EditMessage::SetReadonly(readonly) => {
                self.set_readonly(readonly)?;
                Ok(false)
            }
        }
    }
}

winio_handle::impl_as_widget!(Edit, widget);
