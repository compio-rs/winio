use inherit_methods_macro::inherit_methods;
use winio_elm::{Child, Component, ComponentSender, PropSink, PropSinkEvent, start};
use winio_handle::BorrowedContainer;
use winio_primitive::{
    Enable, Failable, HAlign, Layoutable, Point, Size, TextWidget, ToolTip, Visible,
};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A simple single-line label.
#[derive(Debug)]
pub struct Label {
    widget: sys::Label,
    text_prop: Child<PropSink<String>>,
    halign_prop: Child<PropSink<HAlign>>,
    enabled_prop: Child<PropSink<bool>>,
    visible_prop: Child<PropSink<bool>>,
    tooltip_prop: Child<PropSink<String>>,
    #[cfg(win32)]
    transparent_prop: Child<PropSink<bool>>,
}

impl Failable for Label {
    type Error = Error;
}

#[inherit_methods(from = "self.widget")]
impl ToolTip for Label {
    fn tooltip(&self) -> Result<String>;

    fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl TextWidget for Label {
    fn text(&self) -> Result<String>;

    fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Label {
    /// The horizontal alignment.
    pub fn halign(&self) -> Result<HAlign>;

    /// Set the horizontal alignment.
    pub fn set_halign(&mut self, align: HAlign) -> Result<()>;

    /// If the label background is transparent.
    #[cfg(win32)]
    pub fn is_transparent(&self) -> Result<bool>;

    /// Set if the label background is transparent.
    #[cfg(win32)]
    pub fn set_transparent(&mut self, v: bool) -> Result<()>;

    /// Property for [`TextWidget::text`].
    pub fn text_prop(&self) -> &PropSink<String> {
        &self.text_prop
    }

    /// Property for [`Label::halign`].
    pub fn halign_prop(&self) -> &PropSink<HAlign> {
        &self.halign_prop
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

    #[cfg(win32)]
    #[allow(dead_code)]
    /// Property for [`Label::is_transparent`].
    pub fn transparent_prop(&self) -> &PropSink<bool> {
        &self.transparent_prop
    }
}

#[inherit_methods(from = "self.widget")]
impl Visible for Label {
    fn is_visible(&self) -> Result<bool>;

    fn set_visible(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Enable for Label {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for Label {
    fn loc(&self) -> Result<Point>;

    fn set_loc(&mut self, p: Point) -> Result<()>;

    fn size(&self) -> Result<Size>;

    fn set_size(&mut self, v: Size) -> Result<()>;

    fn preferred_size(&self) -> Result<Size>;
}

/// Events of [`Label`].
#[derive(Debug)]
#[non_exhaustive]
pub enum LabelEvent {}

/// Messages of [`Label`].
#[derive(Debug)]
#[non_exhaustive]
pub enum LabelMessage {
    /// No operation.
    Noop,
    /// The text has been changed.
    Change,
    /// The halign has been changed.
    ChangeHalign,
    /// The enabled state has been changed.
    ChangeEnabled,
    /// The visible state has been changed.
    ChangeVisible,
    /// The tooltip has been changed.
    ChangeTooltip,
    #[cfg(win32)]
    /// The transparent state has been changed.
    ChangeTransparent,
}

impl Component for Label {
    type Error = Error;
    type Event = LabelEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = LabelMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::Label::new(init)?;
        let Ok(text_prop) = Child::<PropSink<String>>::init(String::new()).await;
        let Ok(halign_prop) = Child::<PropSink<HAlign>>::init(HAlign::Left).await;
        let Ok(enabled_prop) = Child::<PropSink<bool>>::init(true).await;
        let Ok(visible_prop) = Child::<PropSink<bool>>::init(true).await;
        let Ok(tooltip_prop) = Child::<PropSink<String>>::init(String::new()).await;
        #[cfg(win32)]
        let Ok(transparent_prop) = Child::<PropSink<bool>>::init(false).await;
        Ok(Self {
            widget,
            text_prop,
            halign_prop,
            enabled_prop,
            visible_prop,
            tooltip_prop,
            #[cfg(win32)]
            transparent_prop,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        let fut_base = async {
            start! {
                sender, default: LabelMessage::Noop,
                self.text_prop => {
                    PropSinkEvent::Changed => LabelMessage::Change,
                },
                self.halign_prop => { PropSinkEvent::Changed => LabelMessage::ChangeHalign },
                self.enabled_prop => { PropSinkEvent::Changed => LabelMessage::ChangeEnabled },
                self.visible_prop => { PropSinkEvent::Changed => LabelMessage::ChangeVisible },
                self.tooltip_prop => { PropSinkEvent::Changed => LabelMessage::ChangeTooltip },
            }
        };
        #[cfg(win32)]
        let fut_transparent = async {
            start! {
                sender, default: LabelMessage::Noop,
                self.transparent_prop => { PropSinkEvent::Changed => LabelMessage::ChangeTransparent },
            }
        };
        #[cfg(not(win32))]
        let fut_transparent = std::future::pending::<()>();
        futures_util::future::join(fut_base, fut_transparent).await.0
    }

    async fn update_children(&mut self) -> Result<bool> {
        let Ok(r0) = self.text_prop.update().await;
        let Ok(r1) = self.halign_prop.update().await;
        let Ok(r2) = self.enabled_prop.update().await;
        let Ok(r3) = self.visible_prop.update().await;
        let Ok(r4) = self.tooltip_prop.update().await;
        #[cfg(win32)]
        let Ok(r5) = self.transparent_prop.update().await;
        #[cfg(not(win32))]
        let r5 = false;
        Ok(r0 || r1 || r2 || r3 || r4 || r5)
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            LabelMessage::Noop => Ok(false),
            LabelMessage::Change => {
                self.widget.set_text(self.text_prop.get())?;
                Ok(true)
            }
            LabelMessage::ChangeHalign => {
                self.widget.set_halign(*self.halign_prop.get())?;
                Ok(true)
            }
            LabelMessage::ChangeEnabled => {
                self.widget.set_enabled(**self.enabled_prop)?;
                Ok(true)
            }
            LabelMessage::ChangeVisible => {
                self.widget.set_visible(**self.visible_prop)?;
                Ok(true)
            }
            LabelMessage::ChangeTooltip => {
                self.widget.set_tooltip(self.tooltip_prop.get())?;
                Ok(true)
            }
            #[cfg(win32)]
            LabelMessage::ChangeTransparent => {
                self.widget.set_transparent(**self.transparent_prop)?;
                Ok(true)
            }
        }
    }
}

winio_handle::impl_as_widget!(Label, widget);
