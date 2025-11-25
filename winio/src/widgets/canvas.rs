use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};
use winio_handle::BorrowedContainer;
use winio_primitive::{
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

    fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Visible for Canvas {
    fn is_visible(&self) -> Result<bool>;

    fn set_visible(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Enable for Canvas {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for Canvas {
    fn loc(&self) -> Result<Point>;

    fn set_loc(&mut self, p: Point) -> Result<()>;

    fn size(&self) -> Result<Size>;

    fn set_size(&mut self, v: Size) -> Result<()>;
}

/// Events of [`Canvas`].
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

impl Component for Canvas {
    type Event = CanvasEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = ();

    fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::Canvas::new(init)?;
        Ok(Self { widget })
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
        futures_util::future::join4(fut_move, fut_down, fut_up, fut_wheel)
            .await
            .0
    }
}

winio_handle::impl_as_widget!(Canvas, widget);
