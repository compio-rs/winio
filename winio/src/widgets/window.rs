use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};
use winio_layout::Layoutable;
use winio_primitive::{Failable, Point, Size, TextWidget, Visible};

#[cfg(windows)]
pub use crate::sys::Backdrop;
#[cfg(target_os = "macos")]
pub use crate::sys::Vibrancy;
use crate::{
    sys,
    sys::{Error, Result},
};

/// A simple window.
///
/// ## Platform specific
/// * Qt: The desctruct order of Qt requires the window to be dropped last, and
///   you should better put it at the end of the struct.
#[derive(Debug)]
pub struct Window {
    widget: sys::Window,
}

impl Failable for Window {
    type Error = Error;
}

#[inherit_methods(from = "self.widget")]
impl TextWidget for Window {
    fn text(&self) -> Result<String>;

    fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Window {
    /// The inner client size.
    pub fn client_size(&self) -> Result<Size>;

    /// Set window icon by resource ID.
    #[cfg(windows)]
    pub fn set_icon_by_id(&mut self, id: u16) -> Result<()>;

    /// Get window style.
    #[cfg(win32)]
    pub fn style(&self) -> Result<u32>;

    /// Set window style.
    #[cfg(win32)]
    pub fn set_style(&mut self, s: u32) -> Result<()>;

    /// Get window extended style.
    #[cfg(win32)]
    pub fn ex_style(&self) -> Result<u32>;

    /// Set window extended style.
    #[cfg(win32)]
    pub fn set_ex_style(&mut self, s: u32) -> Result<()>;

    /// Get the backdrop effect of the window.
    ///
    /// Returns an error if the platform does not support it.
    ///
    /// # Platform specific
    /// * Win32: Supported on Windows 11 22H2 and later; some controls might
    ///   look weird.
    /// * WinUI: Supported on 1.3 and later; the color of the title bar might be
    ///   different from the client area.
    #[cfg(windows)]
    pub fn backdrop(&self) -> Result<Backdrop>;

    /// Set the backdrop effect of the window.
    #[cfg(windows)]
    pub fn set_backdrop(&mut self, backdrop: Backdrop) -> Result<()>;

    /// Get the visual effect of the window.
    #[cfg(target_os = "macos")]
    pub fn vibrancy(&self) -> Result<Option<Vibrancy>>;

    /// Set the visual effect of the window.
    #[cfg(target_os = "macos")]
    pub fn set_vibrancy(&mut self, v: Option<Vibrancy>) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Visible for Window {
    fn is_visible(&self) -> Result<bool>;

    fn set_visible(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for Window {
    fn loc(&self) -> Result<Point>;

    fn set_loc(&mut self, p: Point) -> Result<()>;

    fn size(&self) -> Result<Size>;

    fn set_size(&mut self, v: Size) -> Result<()>;
}

/// Events of [`Window`].
#[derive(Debug)]
#[non_exhaustive]
pub enum WindowEvent {
    /// The window is about to close. If it is ignored, the window WILL NOT
    /// close.
    Close,
    /// The window has been moved.
    Move,
    /// The window has been resized.
    Resize,
    /// The window theme has been changed.
    ThemeChanged,
}

/// Messages of [`Window`].
#[derive(Debug)]
#[non_exhaustive]
pub enum WindowMessage {}

impl Component for Window {
    type Error = Error;
    type Event = WindowEvent;
    type Init<'a> = ();
    type Message = WindowMessage;

    async fn init(_init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::Window::new()?;
        Ok(Self { widget })
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
        let fut_theme = async {
            loop {
                self.widget.wait_theme_changed().await;
                sender.output(WindowEvent::ThemeChanged);
            }
        };
        futures_util::future::join4(fut_close, fut_move, fut_resize, fut_theme)
            .await
            .0
    }
}

winio_handle::impl_as_window!(Window, widget);
winio_handle::impl_as_container!(Window, widget);
