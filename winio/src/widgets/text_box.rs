use inherit_methods_macro::inherit_methods;
use winio_elm::{Child, Component, ComponentSender, Prop, PropSink, PropSinkEvent, PropSinkMessage, start};
use winio_handle::BorrowedContainer;
use winio_primitive::{
    Enable, Failable, HAlign, Layoutable, Point, Size, TextWidget, ToolTip, Visible,
};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A simple multi-line text input box.
#[derive(Debug)]
pub struct TextBox {
    widget: sys::TextBox,
    text_prop: Child<Prop<String>>,
    halign_prop: Child<PropSink<HAlign>>,
    readonly_prop: Child<PropSink<bool>>,
    enabled_prop: Child<PropSink<bool>>,
    visible_prop: Child<PropSink<bool>>,
    tooltip_prop: Child<PropSink<String>>,
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
    /// The multi-line text, using LF as line separator.
    fn text(&self) -> Result<String>;

    /// Set the entire text content.
    fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl TextBox {
    /// The horizontal alignment.
    pub fn halign(&self) -> Result<HAlign>;

    /// Set the horizontal alignment.
    pub fn set_halign(&mut self, align: HAlign) -> Result<()>;

    /// If the text input is read-only.
    pub fn is_readonly(&self) -> Result<bool>;

    /// Set if the text input is read-only.
    pub fn set_readonly(&mut self, v: bool) -> Result<()>;

    /// Property for [`TextWidget::text`].
    pub fn text_prop(&mut self) -> &mut Prop<String> {
        &mut self.text_prop
    }

    /// Property for [`TextBox::halign`].
    pub fn halign_prop(&self) -> &PropSink<HAlign> {
        &self.halign_prop
    }

    /// Property for [`TextBox::is_readonly`].
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

    fn set_size(&mut self, v: Size) -> Result<()>;

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
    /// No operation.
    Noop,
    /// The text has been changed by user input.
    ChangeInput,
    /// The text prop has been changed.
    ChangeProp,
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

impl Component for TextBox {
    type Error = Error;
    type Event = TextBoxEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = TextBoxMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::TextBox::new(init)?;
        let Ok(text_prop) = Child::<Prop<String>>::init(String::new()).await;
        let Ok(halign_prop) = Child::<PropSink<HAlign>>::init(HAlign::Left).await;
        let Ok(readonly_prop) = Child::<PropSink<bool>>::init(false).await;
        let Ok(enabled_prop) = Child::<PropSink<bool>>::init(true).await;
        let Ok(visible_prop) = Child::<PropSink<bool>>::init(true).await;
        let Ok(tooltip_prop) = Child::<PropSink<String>>::init(String::new()).await;
        Ok(Self {
            widget,
            text_prop,
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
                sender.post(TextBoxMessage::ChangeInput);
            }
        };
        let fut_props = async {
            start! {
                sender, default: TextBoxMessage::Noop,
                self.text_prop => { PropSinkEvent::Changed => TextBoxMessage::ChangeProp },
                self.halign_prop => { PropSinkEvent::Changed => TextBoxMessage::ChangeHalign },
                self.readonly_prop => { PropSinkEvent::Changed => TextBoxMessage::ChangeReadonly },
                self.enabled_prop => { PropSinkEvent::Changed => TextBoxMessage::ChangeEnabled },
                self.visible_prop => { PropSinkEvent::Changed => TextBoxMessage::ChangeVisible },
                self.tooltip_prop => { PropSinkEvent::Changed => TextBoxMessage::ChangeTooltip },
            }
        };
        futures_util::future::join(fut_listen, fut_props).await.0
    }

    async fn update_children(&mut self) -> Result<bool> {
        let Ok(r0) = self.text_prop.update().await;
        let Ok(r1) = self.halign_prop.update().await;
        let Ok(r2) = self.readonly_prop.update().await;
        let Ok(r3) = self.enabled_prop.update().await;
        let Ok(r4) = self.visible_prop.update().await;
        let Ok(r5) = self.tooltip_prop.update().await;
        Ok(r0 || r1 || r2 || r3 || r4 || r5)
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            TextBoxMessage::Noop => Ok(false),
            TextBoxMessage::ChangeInput => {
                let text = self.widget.text()?;
                self.text_prop.post(PropSinkMessage::Set(text));
                Ok(false)
            }
            TextBoxMessage::ChangeProp => {
                let text = self.widget.text()?;
                if &text != self.text_prop.get() {
                    self.widget.set_text(self.text_prop.get())?;
                }
                Ok(true)
            }
            TextBoxMessage::ChangeHalign => {
                self.widget.set_halign(*self.halign_prop.get())?;
                Ok(true)
            }
            TextBoxMessage::ChangeReadonly => {
                self.widget.set_readonly(**self.readonly_prop)?;
                Ok(true)
            }
            TextBoxMessage::ChangeEnabled => {
                self.widget.set_enabled(**self.enabled_prop)?;
                Ok(true)
            }
            TextBoxMessage::ChangeVisible => {
                self.widget.set_visible(**self.visible_prop)?;
                Ok(true)
            }
            TextBoxMessage::ChangeTooltip => {
                self.widget.set_tooltip(self.tooltip_prop.get())?;
                Ok(true)
            }
        }
    }
}

winio_handle::impl_as_widget!(TextBox, widget);
