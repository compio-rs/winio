use inherit_methods_macro::inherit_methods;
use winio_elm::{Child, Component, ComponentSender, PropSink, PropSinkEvent, start};
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
    text_prop: Child<PropSink<String>>,
    visible_prop: Child<PropSink<bool>>,
    #[cfg(win32)]
    style_prop: Child<PropSink<u32>>,
    #[cfg(win32)]
    ex_style_prop: Child<PropSink<u32>>,
    #[cfg(windows)]
    backdrop_prop: Child<PropSink<Backdrop>>,
    #[cfg(target_os = "macos")]
    vibrancy_prop: Child<PropSink<Option<Vibrancy>>>,
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
    /// ## Platform specific
    /// * Win32: Supported on Windows 11 22H2 and later; some controls might
    ///   look weird.
    /// * WinUI: Supported on 1.3 and later; the color of the title bar might be
    ///   different from the client area.
    #[cfg(windows)]
    pub fn backdrop(&self) -> Result<Backdrop>;

    /// Set the backdrop effect of the window.
    ///
    /// ## Platform specific
    /// * Win32: backdrop effects may cause rendering artifacts.
    #[cfg(windows)]
    pub fn set_backdrop(&mut self, backdrop: Backdrop) -> Result<()>;

    /// Get the visual effect of the window.
    #[cfg(target_os = "macos")]
    pub fn vibrancy(&self) -> Result<Option<Vibrancy>>;

    /// Set the visual effect of the window.
    #[cfg(target_os = "macos")]
    pub fn set_vibrancy(&mut self, v: Option<Vibrancy>) -> Result<()>;

    /// Property for [`TextWidget::text`].
    pub fn text_prop(&self) -> &PropSink<String> {
        &self.text_prop
    }

    /// Property for [`Visible::set_visible`].
    pub fn visible_prop(&self) -> &PropSink<bool> {
        &self.visible_prop
    }

    /// Property for [`Window::style`].
    #[cfg(win32)]
    pub fn style_prop(&self) -> &PropSink<u32> {
        &self.style_prop
    }

    /// Property for [`Window::ex_style`].
    #[cfg(win32)]
    pub fn ex_style_prop(&self) -> &PropSink<u32> {
        &self.ex_style_prop
    }

    /// Property for [`Window::backdrop`].
    #[cfg(windows)]
    pub fn backdrop_prop(&self) -> &PropSink<Backdrop> {
        &self.backdrop_prop
    }

    /// Property for [`Window::vibrancy`].
    #[cfg(target_os = "macos")]
    pub fn vibrancy_prop(&self) -> &PropSink<Option<Vibrancy>> {
        &self.vibrancy_prop
    }
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
pub enum WindowMessage {
    /// No operation.
    Noop,
    /// The text has been changed.
    ChangeText,
    /// The visible state has been changed.
    ChangeVisible,
    #[cfg(win32)]
    /// The style has been changed.
    ChangeStyle,
    #[cfg(win32)]
    /// The ex style has been changed.
    ChangeExStyle,
    #[cfg(windows)]
    /// The backdrop has been changed.
    ChangeBackdrop,
    #[cfg(target_os = "macos")]
    /// The vibrancy has been changed.
    ChangeVibrancy,
}

impl Component for Window {
    type Error = Error;
    type Event = WindowEvent;
    type Init<'a> = ();
    type Message = WindowMessage;

    async fn init(_init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::Window::new()?;
        let Ok(text_prop) = Child::<PropSink<String>>::init(String::new()).await;
        let Ok(visible_prop) = Child::<PropSink<bool>>::init(true).await;
        #[cfg(win32)]
        let Ok(style_prop) = Child::<PropSink<u32>>::init(0u32).await;
        #[cfg(win32)]
        let Ok(ex_style_prop) = Child::<PropSink<u32>>::init(0u32).await;
        #[cfg(windows)]
        let Ok(backdrop_prop) = Child::<PropSink<Backdrop>>::init(Backdrop::None).await;
        #[cfg(target_os = "macos")]
        let Ok(vibrancy_prop) = Child::<PropSink<Option<Vibrancy>>>::init(None).await;
        Ok(Self {
            widget,
            text_prop,
            visible_prop,
            #[cfg(win32)]
            style_prop,
            #[cfg(win32)]
            ex_style_prop,
            #[cfg(windows)]
            backdrop_prop,
            #[cfg(target_os = "macos")]
            vibrancy_prop,
        })
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
        let fut_props = async {
            start! {
                sender, default: WindowMessage::Noop,
                self.text_prop => { PropSinkEvent::Changed => WindowMessage::ChangeText },
                self.visible_prop => { PropSinkEvent::Changed => WindowMessage::ChangeVisible },
            }
        };
        #[cfg(win32)]
        let fut_style = async {
            start! {
                sender, default: WindowMessage::Noop,
                self.style_prop => { PropSinkEvent::Changed => WindowMessage::ChangeStyle },
                self.ex_style_prop => { PropSinkEvent::Changed => WindowMessage::ChangeExStyle },
            }
        };
        #[cfg(not(win32))]
        let fut_style = std::future::pending::<()>();
        #[cfg(windows)]
        let fut_backdrop = async {
            start! {
                sender, default: WindowMessage::Noop,
                self.backdrop_prop => { PropSinkEvent::Changed => WindowMessage::ChangeBackdrop },
            }
        };
        #[cfg(not(windows))]
        let fut_backdrop = std::future::pending::<()>();
        #[cfg(target_os = "macos")]
        let fut_vibrancy = async {
            start! {
                sender, default: WindowMessage::Noop,
                self.vibrancy_prop => { PropSinkEvent::Changed => WindowMessage::ChangeVibrancy },
            }
        };
        #[cfg(not(target_os = "macos"))]
        let fut_vibrancy = std::future::pending::<()>();

        futures_util::join!(
            fut_close,
            fut_move,
            fut_resize,
            fut_theme,
            fut_props,
            fut_style,
            fut_backdrop,
            fut_vibrancy,
        )
        .0
    }

    async fn update_children(&mut self) -> Result<bool> {
        let Ok(r0) = self.text_prop.update().await;
        let Ok(r1) = self.visible_prop.update().await;
        #[cfg(win32)]
        let Ok(r2) = self.style_prop.update().await;
        #[cfg(not(win32))]
        let r2 = false;
        #[cfg(win32)]
        let Ok(r3) = self.ex_style_prop.update().await;
        #[cfg(not(win32))]
        let r3 = false;
        #[cfg(windows)]
        let Ok(r4) = self.backdrop_prop.update().await;
        #[cfg(not(windows))]
        let r4 = false;
        #[cfg(target_os = "macos")]
        let Ok(r5) = self.vibrancy_prop.update().await;
        #[cfg(not(target_os = "macos"))]
        let r5 = false;
        Ok(r0 || r1 || r2 || r3 || r4 || r5)
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            WindowMessage::Noop => Ok(false),
            WindowMessage::ChangeText => {
                self.widget.set_text(self.text_prop.get())?;
                Ok(true)
            }
            WindowMessage::ChangeVisible => {
                self.widget.set_visible(**self.visible_prop)?;
                Ok(true)
            }
            #[cfg(win32)]
            WindowMessage::ChangeStyle => {
                self.widget.set_style(**self.style_prop)?;
                Ok(true)
            }
            #[cfg(win32)]
            WindowMessage::ChangeExStyle => {
                self.widget.set_ex_style(**self.ex_style_prop)?;
                Ok(true)
            }
            #[cfg(windows)]
            WindowMessage::ChangeBackdrop => {
                self.widget.set_backdrop(*self.backdrop_prop.get())?;
                Ok(true)
            }
            #[cfg(target_os = "macos")]
            WindowMessage::ChangeVibrancy => {
                self.widget.set_vibrancy(self.vibrancy_prop.get().clone())?;
                Ok(true)
            }
        }
    }
}

winio_handle::impl_as_window!(Window, widget);
winio_handle::impl_as_container!(Window, widget);
