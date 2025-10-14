use inherit_methods_macro::inherit_methods;
#[cfg(windows)]
pub use sys::Backdrop;
use winio_elm::{Component, ComponentSender};
use winio_layout::{Layoutable, TextWidget, Visible};
use winio_primitive::{Point, Size};

use crate::sys;

/// A simple window.
///
/// ## Platform specific
/// * Qt: The desctruct order of Qt requires the window to be dropped last, and
///   you should better put it at the end of the struct.
#[derive(Debug)]
pub struct Window {
    widget: sys::Window,
}

#[inherit_methods(from = "self.widget")]
impl TextWidget for Window {
    fn text(&self) -> String;

    fn set_text(&mut self, s: impl AsRef<str>);
}

#[inherit_methods(from = "self.widget")]
impl Window {
    /// The inner client size.
    pub fn client_size(&self) -> Size;

    /// Set window icon by resource ID.
    #[cfg(windows)]
    pub fn set_icon_by_id(&mut self, id: u16);

    /// Get window style.
    #[cfg(all(windows, feature = "win32"))]
    pub fn style(&self) -> u32;

    /// Set window style.
    #[cfg(all(windows, feature = "win32"))]
    pub fn set_style(&mut self, s: u32);

    /// Get window extended style.
    #[cfg(all(windows, feature = "win32"))]
    pub fn ex_style(&self) -> u32;

    /// Set window extended style.
    #[cfg(all(windows, feature = "win32"))]
    pub fn set_ex_style(&mut self, s: u32);

    /// Get the backdrop effect of the window.
    #[cfg(windows)]
    pub fn backdrop(&self) -> Backdrop;

    /// Set the backdrop effect of the window.
    #[cfg(windows)]
    pub fn set_backdrop(&mut self, backdrop: Backdrop);
}

#[inherit_methods(from = "self.widget")]
impl Visible for Window {
    fn is_visible(&self) -> bool;

    fn set_visible(&mut self, v: bool);
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for Window {
    fn loc(&self) -> Point;

    fn set_loc(&mut self, p: Point);

    fn size(&self) -> Size;

    fn set_size(&mut self, v: Size);
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
    type Init<'a> = ();
    type Message = ();

    fn init(_init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Self {
        Self {
            widget: sys::Window::new(),
        }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
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
        futures_util::future::join3(fut_close, fut_move, fut_resize)
            .await
            .0
    }

    async fn update(&mut self, _message: Self::Message, _sender: &ComponentSender<Self>) -> bool {
        false
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {}
}

winio_handle::impl_as_window!(Window, widget);
winio_handle::impl_as_container!(Window, widget);
