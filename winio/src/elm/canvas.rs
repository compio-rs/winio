use inherit_methods_macro::inherit_methods;

use crate::{
    BorrowedWindow, Component, ComponentSender, DrawingContext, Enable, Layoutable, MouseButton,
    Point, Size, Visible, ui,
};

/// A simple drawing canvas.
#[derive(Debug)]
pub struct Canvas {
    widget: ui::Canvas,
}

impl Canvas {
    /// Create the [`DrawingContext`] of the current canvas.
    pub fn context(&mut self) -> DrawingContext<'_> {
        DrawingContext::new(self.widget.context())
    }
}

#[inherit_methods(from = "self.widget")]
impl Visible for Canvas {
    fn is_visible(&self) -> bool;

    fn set_visible(&mut self, v: bool);
}

#[inherit_methods(from = "self.widget")]
impl Enable for Canvas {
    fn is_enabled(&self) -> bool;

    fn set_enabled(&mut self, v: bool);
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for Canvas {
    fn loc(&self) -> Point;

    fn set_loc(&mut self, p: Point);

    fn size(&self) -> Size;

    fn set_size(&mut self, v: Size);
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
}

impl Component for Canvas {
    type Event = CanvasEvent;
    type Init<'a> = BorrowedWindow<'a>;
    type Message = ();

    fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Self {
        let widget = ui::Canvas::new(init);
        Self { widget }
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
        futures_util::future::join3(fut_move, fut_down, fut_up)
            .await
            .0
    }

    async fn update(&mut self, _message: Self::Message, _sender: &ComponentSender<Self>) -> bool {
        false
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {}
}
