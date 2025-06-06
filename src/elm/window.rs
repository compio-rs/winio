use crate::{AsRawWindow, Component, ComponentSender, Layoutable, Point, RawWindow, Size, ui};

/// A simple window.
#[derive(Debug)]
pub struct Window {
    widget: ui::Window,
}

impl Window {
    /// The title.
    pub fn text(&self) -> String {
        self.widget.text()
    }

    /// Set the title.
    pub fn set_text(&mut self, s: impl AsRef<str>) {
        self.widget.set_text(s)
    }

    /// The inner client size.
    pub fn client_size(&self) -> Size {
        self.widget.client_size()
    }

    /// Set window icon by resource ID.
    #[cfg(windows)]
    pub fn set_icon_by_id(&mut self, id: u16) {
        self.widget.set_icon_by_id(id);
    }
}

impl Layoutable for Window {
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

impl AsRawWindow for Window {
    fn as_raw_window(&self) -> RawWindow {
        self.widget.as_raw_window()
    }
}

/// Trait for a widget to set visibility.
pub trait Visible {
    /// If the widget is visible.
    fn is_visible(&self) -> bool;

    /// Set the visibility.
    fn set_visible(&mut self, v: bool);

    /// Show the widget.
    fn show(&mut self) {
        self.set_visible(true);
    }

    /// Hide the widget.
    fn hide(&mut self) {
        self.set_visible(false);
    }
}

impl Visible for Window {
    fn is_visible(&self) -> bool {
        self.widget.is_visible()
    }

    fn set_visible(&mut self, v: bool) {
        self.widget.set_visible(v);
    }
}

/// Events of [`Window`].
#[non_exhaustive]
pub enum WindowEvent {
    /// The window is about to close. If it is ignored, the window WILL NOT
    /// close.
    Close,
    /// The window has been moved.
    Move,
    /// The window has been resized.
    Resize,
}

impl Component for Window {
    type Event = WindowEvent;
    type Init = ();
    type Message = ();
    type Root = ();

    fn init(_counter: Self::Init, _root: &(), _sender: &ComponentSender<Self>) -> Self {
        Self {
            widget: ui::Window::new(),
        }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) {
        let fut_close = async {
            loop {
                self.widget.wait_close().await;
                sender.output(WindowEvent::Close);
            }
        };
        let fut_move = async {
            loop {
                self.widget.wait_move().await;
                sender.output(WindowEvent::Move);
            }
        };
        let fut_resize = async {
            loop {
                self.widget.wait_size().await;
                sender.output(WindowEvent::Resize);
            }
        };
        futures_util::future::join3(fut_close, fut_move, fut_resize).await;
    }

    async fn update(&mut self, _message: Self::Message, _sender: &ComponentSender<Self>) -> bool {
        false
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {}
}
