use inherit_methods_macro::inherit_methods;
use winio_elm::{Child, Component, ComponentSender, PropSink, PropSinkEvent, start};
use winio_handle::BorrowedContainer;
use winio_primitive::{Rect, 
    Enable, Failable, Layoutable, MouseButton, Point, Size, ToolTip, Vector, Visible,
};

use crate::{
    sys,
    sys::{Error, Result},
    ui::DrawingContext,
};

/// A simple drawing canvas.
#[derive(Debug)]
pub struct Canvas {
    widget: sys::Canvas,
    enabled_prop: Child<PropSink<bool>>,
    visible_prop: Child<PropSink<bool>>,
    tooltip_prop: Child<PropSink<String>>,
    rect_prop: Child<PropSink<Rect>>,
}

impl Canvas {
    /// Create the [`DrawingContext`] of the current canvas.
    pub fn context(&mut self) -> Result<DrawingContext<'_>> {
        Ok(DrawingContext::new(self.widget.context()?))
    }
}

impl Failable for Canvas {
    type Error = Error;
}

#[inherit_methods(from = "self.widget")]
impl ToolTip for Canvas {
    fn tooltip(&self) -> Result<String>;

    fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.tooltip_prop.set(s.as_ref().to_owned());
        Ok(())
    }
}

impl Canvas {
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
impl Visible for Canvas {
    fn is_visible(&self) -> Result<bool>;

    fn set_visible(&mut self, v: bool) -> Result<()> {
        self.visible_prop.set(v);
        Ok(())
    }
}

#[inherit_methods(from = "self.widget")]
impl Enable for Canvas {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, v: bool) -> Result<()> {
        self.enabled_prop.set(v);
        Ok(())
    }
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for Canvas {
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
}

/// Events of [`Canvas`].
#[derive(Debug)]
#[non_exhaustive]
pub enum CanvasEvent {
    /// The mouse moves.
    MouseMove(Point),
    /// The mouse button pressed down.
    MouseDown(MouseButton),
    /// The mouse button released.
    MouseUp(MouseButton),
    /// The mouse wheel rotated.
    /// * `x`: Positive is right.
    /// * `y`: Positive is up/forward.
    MouseWheel(Vector),
}

/// Messages of [`Canvas`].
#[derive(Debug)]
#[non_exhaustive]
pub enum CanvasMessage {
    /// No operation.
    Noop,
    /// The enabled state has been changed.
    ChangeEnabled,
    /// The visible state has been changed.
    ChangeVisible,
    /// The tooltip has been changed.
    ChangeTooltip,
    /// The rect has been changed.
    ChangeRect,
}

impl Component for Canvas {
    type Error = Error;
    type Event = CanvasEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = CanvasMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::Canvas::new(init)?;
        let Ok(enabled_prop) = Child::<PropSink<bool>>::init(true).await;
        let Ok(visible_prop) = Child::<PropSink<bool>>::init(true).await;
        let Ok(tooltip_prop) = Child::<PropSink<String>>::init(String::new()).await;
        let loc = widget.loc()?;
        let size = widget.size()?;
        let rect = Rect::new(loc, size);
        let Ok(rect_prop) = Child::<PropSink<Rect>>::init(rect).await;
        Ok(Self {
            widget,
            enabled_prop,
            visible_prop,
        rect_prop,
            tooltip_prop,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        let fut_move = async {
            loop {
                let p = self.widget.wait_mouse_move().await;
                sender.output(CanvasEvent::MouseMove(p));
            }
        };
        let fut_down = async {
            loop {
                let b = self.widget.wait_mouse_down().await;
                sender.output(CanvasEvent::MouseDown(b));
            }
        };
        let fut_up = async {
            loop {
                let b = self.widget.wait_mouse_up().await;
                sender.output(CanvasEvent::MouseUp(b));
            }
        };
        let fut_wheel = async {
            loop {
                let w = self.widget.wait_mouse_wheel().await;
                sender.output(CanvasEvent::MouseWheel(w));
            }
        };
        let fut_props = async {
            start! {
                sender, default: CanvasMessage::Noop,
                self.enabled_prop => { PropSinkEvent::Changed => CanvasMessage::ChangeEnabled },
                self.visible_prop => { PropSinkEvent::Changed => CanvasMessage::ChangeVisible },
                self.tooltip_prop => { PropSinkEvent::Changed => CanvasMessage::ChangeTooltip },
                self.rect_prop => { PropSinkEvent::Changed => CanvasMessage::ChangeRect },
            }
        };
        futures_util::future::join5(fut_move, fut_down, fut_up, fut_wheel, fut_props)
            .await
            .0
    }

    async fn update_children(&mut self) -> Result<bool> {
        let Ok(r0) = self.enabled_prop.update().await;
        let Ok(r1) = self.visible_prop.update().await;
        let Ok(r2) = self.tooltip_prop.update().await;
        let Ok(r3) = self.rect_prop.update().await;
        Ok(r0 || r1 || r2 || r3)
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            CanvasMessage::Noop => Ok(false),
            CanvasMessage::ChangeEnabled => {
                self.widget.set_enabled(**self.enabled_prop)?;
                Ok(true)
            }
            CanvasMessage::ChangeVisible => {
                self.widget.set_visible(**self.visible_prop)?;
                Ok(true)
            }
            CanvasMessage::ChangeTooltip => {
                self.widget.set_tooltip(self.tooltip_prop.get())?;
                Ok(true)
            }
            CanvasMessage::ChangeRect => {
                let rect = *self.rect_prop.get();
                self.widget.set_loc(rect.origin)?;
                self.widget.set_size(rect.size)?;
                Ok(true)
            }
        }
    }
}

winio_handle::impl_as_widget!(Canvas, widget);
