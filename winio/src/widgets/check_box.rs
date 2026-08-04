use inherit_methods_macro::inherit_methods;
use winio_elm::{
    Child, Component, ComponentSender, Prop, PropSink, PropSinkEvent, PropSinkMessage, start,
};
use winio_handle::BorrowedContainer;
use winio_primitive::{Rect, Enable, Failable, Layoutable, Point, Size, TextWidget, ToolTip, Visible};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A simple check box.
#[derive(Debug)]
pub struct CheckBox {
    widget: sys::CheckBox,
    checked_prop: Child<Prop<bool>>,
    text_prop: Child<PropSink<String>>,
    enabled_prop: Child<PropSink<bool>>,
    visible_prop: Child<PropSink<bool>>,
    tooltip_prop: Child<PropSink<String>>,
    rect_prop: Child<PropSink<Rect>>,
}

impl Failable for CheckBox {
    type Error = Error;
}

#[inherit_methods(from = "self.widget")]
impl ToolTip for CheckBox {
    fn tooltip(&self) -> Result<String>;

    fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.tooltip_prop.set(s.as_ref().to_owned());
        Ok(())
    }
}

#[inherit_methods(from = "self.widget")]
impl TextWidget for CheckBox {
    fn text(&self) -> Result<String>;

    fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.text_prop.set(s.as_ref().to_owned());
        Ok(())
    }
}

#[inherit_methods(from = "self.widget")]
impl CheckBox {
    /// If the box is checked.
    pub fn is_checked(&self) -> Result<bool>;

    /// Set the checked state.
    pub fn set_checked(&mut self, v: bool) -> Result<()> {
        self.checked_prop.set(v);
        Ok(())
    }

    /// Property for [`CheckBox::is_checked`].
    pub fn checked_prop(&mut self) -> &mut Prop<bool> {
        &mut self.checked_prop
    }

    /// Property for [`TextWidget::text`].
    pub fn text_prop(&self) -> &PropSink<String> {
        &self.text_prop
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

    /// Property for [`Layoutable::rect`].
    pub fn rect_prop(&self) -> &PropSink<Rect> {
        &self.rect_prop
    }
}

#[inherit_methods(from = "self.widget")]
impl Visible for CheckBox {
    fn is_visible(&self) -> Result<bool>;

    fn set_visible(&mut self, v: bool) -> Result<()> {
        self.visible_prop.set(v);
        Ok(())
    }
}

#[inherit_methods(from = "self.widget")]
impl Enable for CheckBox {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, v: bool) -> Result<()> {
        self.enabled_prop.set(v);
        Ok(())
    }
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for CheckBox {
    fn loc(&self) -> Result<Point>;

    fn set_loc(&mut self, p: Point) -> Result<()> {
        let rect = *self.rect_prop.get();
        self.rect_prop.set(Rect::new(p, rect.size));
        Ok(())
    }

    fn size(&self) -> Result<Size>;

    fn set_size(&mut self, s: Size) -> Result<()> {
        let rect = *self.rect_prop.get();
        self.rect_prop.set(Rect::new(rect.origin, s));
        Ok(())
    }

    fn preferred_size(&self) -> Result<Size>;
}

/// Events of [`CheckBox`].
#[derive(Debug)]
#[non_exhaustive]
pub enum CheckBoxEvent {
    /// The check box has been clicked.
    Click,
}

/// Messages of [`CheckBox`].
#[derive(Debug)]
#[non_exhaustive]
pub enum CheckBoxMessage {
    /// No operation.
    Noop,
    /// The checked state has been changed by user click.
    ChangeInputChecked,
    /// The checked prop has been changed.
    ChangePropChecked,
    /// The text has been changed.
    ChangeText,
    /// The enabled state has been changed.
    ChangeEnabled,
    /// The visible state has been changed.
    ChangeVisible,
    /// The tooltip has been changed.
    ChangeTooltip,
    /// The rect has been changed.
    ChangeRect,
}

impl Component for CheckBox {
    type Error = Error;
    type Event = CheckBoxEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = CheckBoxMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::CheckBox::new(init)?;
        let Ok(checked_prop) = Child::<Prop<bool>>::init(false).await;
        let Ok(text_prop) = Child::<PropSink<String>>::init(String::new()).await;
        let Ok(enabled_prop) = Child::<PropSink<bool>>::init(true).await;
        let Ok(visible_prop) = Child::<PropSink<bool>>::init(true).await;
        let Ok(tooltip_prop) = Child::<PropSink<String>>::init(String::new()).await;
        let loc = widget.loc()?;
        let size = widget.size()?;
        let rect = Rect::new(loc, size);
        let Ok(rect_prop) = Child::<PropSink<Rect>>::init(rect).await;
        Ok(Self {
            widget,
            checked_prop,
            text_prop,
            enabled_prop,
            visible_prop,
        rect_prop,
            tooltip_prop,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        let fut_click = async {
            loop {
                self.widget.wait_click().await;
                sender.post(CheckBoxMessage::ChangeInputChecked);
            }
        };
        let fut_props = async {
            start! {
                sender, default: CheckBoxMessage::Noop,
                self.checked_prop => { PropSinkEvent::Changed => CheckBoxMessage::ChangePropChecked },
                self.text_prop => { PropSinkEvent::Changed => CheckBoxMessage::ChangeText },
                self.enabled_prop => { PropSinkEvent::Changed => CheckBoxMessage::ChangeEnabled },
                self.visible_prop => { PropSinkEvent::Changed => CheckBoxMessage::ChangeVisible },
                self.tooltip_prop => { PropSinkEvent::Changed => CheckBoxMessage::ChangeTooltip },
                self.rect_prop => { PropSinkEvent::Changed => CheckBoxMessage::ChangeRect },
            }
        };
        futures_util::future::join(fut_click, fut_props).await.0
    }

    async fn update_children(&mut self) -> Result<bool> {
        let Ok(r0) = self.checked_prop.update().await;
        let Ok(r1) = self.text_prop.update().await;
        let Ok(r2) = self.enabled_prop.update().await;
        let Ok(r3) = self.visible_prop.update().await;
        let Ok(r4) = self.tooltip_prop.update().await;
        let Ok(r5) = self.rect_prop.update().await;
        Ok(r0 || r1 || r2 || r3 || r4 || r5)
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            CheckBoxMessage::Noop => Ok(false),
            CheckBoxMessage::ChangeInputChecked => {
                let checked = self.widget.is_checked()?;
                self.checked_prop.post(PropSinkMessage::Set(checked));
                sender.output(CheckBoxEvent::Click);
                Ok(false)
            }
            CheckBoxMessage::ChangePropChecked => {
                let current = self.widget.is_checked()?;
                let prop_val = self.checked_prop.get();
                if current != *prop_val {
                    self.widget.set_checked(*prop_val)?;
                }
                Ok(true)
            }
            CheckBoxMessage::ChangeText => {
                self.widget.set_text(self.text_prop.get())?;
                Ok(true)
            }
            CheckBoxMessage::ChangeEnabled => {
                self.widget.set_enabled(**self.enabled_prop)?;
                Ok(true)
            }
            CheckBoxMessage::ChangeVisible => {
                self.widget.set_visible(**self.visible_prop)?;
                Ok(true)
            }
            CheckBoxMessage::ChangeTooltip => {
                self.widget.set_tooltip(self.tooltip_prop.get())?;
                Ok(true)
            }
            CheckBoxMessage::ChangeRect => {
                let rect = *self.rect_prop.get();
                self.widget.set_loc(rect.origin)?;
                self.widget.set_size(rect.size)?;
                Ok(true)
            }
        }
    }
}

winio_handle::impl_as_widget!(CheckBox, widget);
