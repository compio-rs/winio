use inherit_methods_macro::inherit_methods;
use winio_elm::{Child, Component, ComponentSender, Prop, PropSinkEvent, PropSinkMessage, start};
use winio_handle::BorrowedContainer;
use winio_primitive::{
    Enable, Failable, HAlign, Layoutable, Point, Size, TextWidget, ToolTip, Visible,
};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A simple single-line text input box.
#[derive(Debug)]
pub struct Edit {
    widget: sys::Edit,
    text_prop: Child<Prop<String>>,
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

    fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;
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

    /// Property for [`Edit::text`].
    pub fn text_prop(&self) -> &Prop<String> {
        &self.text_prop
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

    fn set_size(&mut self, v: Size) -> Result<()>;

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
    /// The text has been changed.
    ChangeInput,
    /// The text has been changed, and the new text is provided.
    ChangeProp(String),
}

impl Component for Edit {
    type Error = Error;
    type Event = EditEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = EditMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::Edit::new(init)?;
        let Ok(text_prop) = Child::<Prop<String>>::init(()).await;
        Ok(Self { widget, text_prop })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        let fut_listen = async {
            loop {
                self.widget.wait_change().await;
                sender.output(EditEvent::Change);
                sender.post(EditMessage::ChangeInput);
            }
        };
        let fut_start = async {
            start! {
                sender, default: EditMessage::Noop,
                self.text_prop => {
                    PropSinkEvent::Changed(text) => EditMessage::ChangeProp(text),
                }
            }
        };
        futures_util::future::join(fut_listen, fut_start).await.0
    }

    async fn update_children(&mut self) -> Result<bool> {
        let Ok(res) = self.text_prop.update().await;
        Ok(res)
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            EditMessage::Noop => Ok(false),
            EditMessage::ChangeInput => {
                let text = self.widget.text()?;
                self.text_prop.post(PropSinkMessage::Set(text));
                Ok(false)
            }
            EditMessage::ChangeProp(text) => {
                let old_text = self.widget.text()?;
                if old_text != text {
                    self.widget.set_text(&text)?;
                    self.text_prop.post(PropSinkMessage::Set(text));
                }
                Ok(false)
            }
        }
    }
}

winio_handle::impl_as_widget!(Edit, widget);
