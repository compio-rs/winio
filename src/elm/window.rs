use raw_window_handle::{HandleError, HasWindowHandle, WindowHandle};

use crate::{Component, ComponentSender, Point, Size, ui};

pub struct Window {
    widget: ui::Window,
}

impl Window {
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

    pub fn text(&self) -> String {
        self.widget.text()
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        self.widget.set_text(s)
    }

    pub fn client_size(&self) -> Size {
        self.widget.client_size()
    }

    #[cfg(windows)]
    pub fn set_icon_by_id(&mut self, id: u16) {
        self.widget.set_icon_by_id(id);
    }
}

impl HasWindowHandle for Window {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        self.widget.window_handle()
    }
}

#[non_exhaustive]
pub enum WindowEvent {
    Close,
    Move,
    Resize,
}

impl Component for Window {
    type Event = WindowEvent;
    type Init = ();
    type Message = ();
    type Root = ();

    fn init(_counter: Self::Init, _root: &(), _sender: ComponentSender<Self>) -> Self {
        Self {
            widget: ui::Window::new(),
        }
    }

    async fn start(&mut self, sender: ComponentSender<Self>) {
        let fut_close = async {
            loop {
                self.widget.wait_close().await;
                sender.output(WindowEvent::Close).await;
            }
        };
        let fut_move = async {
            loop {
                self.widget.wait_move().await;
                sender.output(WindowEvent::Move).await;
            }
        };
        let fut_resize = async {
            loop {
                self.widget.wait_size().await;
                sender.output(WindowEvent::Resize).await;
            }
        };
        futures_util::future::join3(fut_close, fut_move, fut_resize).await;
    }

    async fn update(&mut self, _message: Self::Message, _sender: ComponentSender<Self>) -> bool {
        false
    }

    fn render(&mut self, _sender: ComponentSender<Self>) {}
}
