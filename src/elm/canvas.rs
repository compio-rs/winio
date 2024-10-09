use crate::{Component, ComponentSender, DrawingContext, MouseButton, Point, Size, Window, ui};

pub struct Canvas {
    widget: ui::Canvas,
}

impl Canvas {
    pub fn loc(&self) -> Point {
        self.widget.loc()
    }

    pub fn set_loc(&mut self, p: Point) {
        self.widget.set_loc(p)
    }

    pub fn size(&self) -> Size {
        self.widget.size()
    }

    pub fn set_size(&mut self, v: Size) {
        self.widget.set_size(v)
    }

    pub fn context(&mut self) -> DrawingContext<'_> {
        self.widget.context()
    }
}

#[non_exhaustive]
pub enum CanvasEvent {
    MouseMove(Point),
    MouseDown(MouseButton),
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
        futures_util::future::join3(fut_move, fut_down, fut_up).await;
    }

    async fn update(&mut self, _message: Self::Message, _sender: &ComponentSender<Self>) -> bool {
        false
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {}
}
