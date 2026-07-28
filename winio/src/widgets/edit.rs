use inherit_methods_macro::inherit_methods;
use winio_elm::{
    Child, Component, ComponentSender, Prop, PropSink, PropSinkEvent, PropSinkMessage, start,
};
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
    password_prop: Child<PropSink<bool>>,
    halign_prop: Child<PropSink<HAlign>>,
    readonly_prop: Child<PropSink<bool>>,
    enabled_prop: Child<PropSink<bool>>,
    visible_prop: Child<PropSink<bool>>,
    tooltip_prop: Child<PropSink<String>>,
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

    /// Property for [`TextWidget::text`].
    pub fn text_prop(&mut self) -> &mut Prop<String> {
        &mut self.text_prop
    }

    /// Property for [`Edit::is_password`].
    pub fn password_prop(&self) -> &PropSink<bool> {
        &self.password_prop
    }

    /// Property for [`Edit::halign`].
    pub fn halign_prop(&self) -> &PropSink<HAlign> {
        &self.halign_prop
    }

    /// Property for [`Edit::is_readonly`].
    pub fn readonly_prop(&self) -> &PropSink<bool> {
        &self.readonly_prop
    }

    /// Property for [`Enable::set_enabled`].
    pub fn enabled_prop(&self) -> &PropSink<bool> {
        &self.enabled_prop
    }

    /// Property for [`Visible::set_visible`].
    pub fn visible_prop(&self) -> &PropSink<bool> {
        &self.visible_prop
    }

    /// Property for [`ToolTip::set_tooltip`].
    pub fn tooltip_prop(&self) -> &PropSink<String> {
        &self.tooltip_prop
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
pub enum EditEvent {}

/// Messages of [`Edit`].
#[derive(Debug)]
#[non_exhaustive]
pub enum EditMessage {
    /// No operation.
    Noop,
    /// The input has been changed.
    ChangeInput,
    /// The text property has been changed.
    ChangeProp,
    /// The password state has been changed.
    ChangePassword,
    /// The halign has been changed.
    ChangeHalign,
    /// The readonly state has been changed.
    ChangeReadonly,
    /// The enabled state has been changed.
    ChangeEnabled,
    /// The visible state has been changed.
    ChangeVisible,
    /// The tooltip has been changed.
    ChangeTooltip,
}

impl Component for Edit {
    type Error = Error;
    type Event = EditEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = EditMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::Edit::new(init)?;
        let Ok(text_prop) = Child::<Prop<String>>::init(String::new()).await;
        let Ok(password_prop) = Child::<PropSink<bool>>::init(false).await;
        let Ok(halign_prop) = Child::<PropSink<HAlign>>::init(HAlign::Left).await;
        let Ok(readonly_prop) = Child::<PropSink<bool>>::init(false).await;
        let Ok(enabled_prop) = Child::<PropSink<bool>>::init(true).await;
        let Ok(visible_prop) = Child::<PropSink<bool>>::init(true).await;
        let Ok(tooltip_prop) = Child::<PropSink<String>>::init(String::new()).await;
        Ok(Self {
            widget,
            text_prop,
            password_prop,
            halign_prop,
            readonly_prop,
            enabled_prop,
            visible_prop,
            tooltip_prop,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        let fut_listen = async {
            loop {
                self.widget.wait_change().await;
                sender.post(EditMessage::ChangeInput);
            }
        };
        let fut_start = async {
            start! {
                sender, default: EditMessage::Noop,
                self.text_prop => {
                    PropSinkEvent::Changed => EditMessage::ChangeProp,
                },
                self.password_prop => { PropSinkEvent::Changed => EditMessage::ChangePassword },
                self.halign_prop => { PropSinkEvent::Changed => EditMessage::ChangeHalign },
                self.readonly_prop => { PropSinkEvent::Changed => EditMessage::ChangeReadonly },
                self.enabled_prop => { PropSinkEvent::Changed => EditMessage::ChangeEnabled },
                self.visible_prop => { PropSinkEvent::Changed => EditMessage::ChangeVisible },
                self.tooltip_prop => { PropSinkEvent::Changed => EditMessage::ChangeTooltip },
            }
        };
        futures_util::future::join(fut_listen, fut_start).await.0
    }

    async fn update_children(&mut self) -> Result<bool> {
        let Ok(r0) = self.text_prop.update().await;
        let Ok(r1) = self.password_prop.update().await;
        let Ok(r2) = self.halign_prop.update().await;
        let Ok(r3) = self.readonly_prop.update().await;
        let Ok(r4) = self.enabled_prop.update().await;
        let Ok(r5) = self.visible_prop.update().await;
        let Ok(r6) = self.tooltip_prop.update().await;
        Ok(r0 || r1 || r2 || r3 || r4 || r5 || r6)
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
            EditMessage::ChangeProp => {
                let text = self.widget.text()?;
                if &text != self.text_prop.get() {
                    self.widget.set_text(self.text_prop.get())?;
                }
                Ok(true)
            }
            EditMessage::ChangePassword => {
                self.widget.set_password(**self.password_prop)?;
                Ok(true)
            }
            EditMessage::ChangeHalign => {
                self.widget.set_halign(*self.halign_prop.get())?;
                Ok(true)
            }
            EditMessage::ChangeReadonly => {
                self.widget.set_readonly(**self.readonly_prop)?;
                Ok(true)
            }
            EditMessage::ChangeEnabled => {
                self.widget.set_enabled(**self.enabled_prop)?;
                Ok(true)
            }
            EditMessage::ChangeVisible => {
                self.widget.set_visible(**self.visible_prop)?;
                Ok(true)
            }
            EditMessage::ChangeTooltip => {
                self.widget.set_tooltip(self.tooltip_prop.get())?;
                Ok(true)
            }
        }
    }
}

winio_handle::impl_as_widget!(Edit, widget);
