use crate::{
    Component, ComponentSender, DrawingContext, Layoutable, MouseButton, Point, Size, Visible,
    Window, ui,
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

impl Visible for Canvas {
    fn is_visible(&self) -> bool {
        self.widget.is_visible()
    }

    fn set_visible(&mut self, v: bool) {
        self.widget.set_visible(v);
    }
}

impl Layoutable for Canvas {
    fn loc(&self) -> Point {
        self.widget.loc()
    }

    fn set_loc(&mut self, p: Point) {
        self.widget.set_loc(p)
    }

    fn size(&self) -> Size {
        self.widget.size()
    }

    fn set_size(&mut self, v: Size) {
        self.widget.set_size(v)
    }
}

/// Events of [`Canvas`].
#[non_exhaustive]
pub enum CanvasEvent {
    /// The canvas needs redraw.
    Redraw,
    /// The mouse moves.
    MouseMove(Point),
    /// The mouse button pressed down.
    MouseDown(MouseButton),
    /// The mouse button released.
    MouseUp(MouseButton),
}

impl Component for Canvas {
    type Event = CanvasEvent;
    type Init = ();
    type Message = ();
    type Root = Window;

    fn init(_counter: Self::Init, root: &Self::Root, _sender: &ComponentSender<Self>) -> Self {
        let widget = ui::Canvas::new(root);
        Self { widget }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) {
        let fut_redraw = async {
            loop {
                self.widget.wait_redraw().await;
                sender.output(CanvasEvent::Redraw);
            }
        };
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
        futures_util::future::join4(fut_redraw, fut_move, fut_down, fut_up).await;
    }

    async fn update(&mut self, _message: Self::Message, _sender: &ComponentSender<Self>) -> bool {
        false
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {}
}
