use inherit_methods_macro::inherit_methods;
use wgpu::{Adapter, Device, Instance, PresentMode, Queue, Texture, TextureFormat, TextureView};
use winio_elm::{Component, ComponentSender};
use winio_handle::{AsContainer, BorrowedContainer};
use winio_primitive::{
    Enable, Failable, KeyCode, Layoutable, MouseButton, Point, Rect, Size, ToolTip, Vector, Visible,
};

use crate::{
    sys,
    sys::{Error, Result},
};

#[derive(Debug)]
pub(crate) struct WgpuCanvasBase {
    widget: sys::WgpuCanvas,
}

#[inherit_methods(from = "self.widget")]
impl WgpuCanvasBase {
    pub(crate) fn new(parent: impl AsContainer) -> Result<Self> {
        Ok(Self {
            widget: sys::WgpuCanvas::new(parent)?,
        })
    }

    pub(crate) fn bind(&mut self, instance: &Instance, adapter: &Adapter);

    pub(crate) fn present_mode(&self) -> Option<PresentMode>;

    pub(crate) fn set_present_mode(&mut self, mode: PresentMode);

    pub(crate) fn acquire(&mut self, device: &Device) -> Option<WgpuCanvasFrame> {
        let (texture, view, format) = self.widget.acquire(device)?;
        Some(WgpuCanvasFrame {
            texture,
            view,
            format,
        })
    }

    pub(crate) fn present(&mut self, queue: &Queue) -> Result<()>;

    pub(crate) async fn start<T>(&self, sender: &ComponentSender<T>) -> !
    where
        T: Component<Event = WgpuCanvasEvent>,
    {
        let fut_move = async {
            loop {
                let p = self.widget.wait_mouse_move().await;
                sender.output(WgpuCanvasEvent::MouseMove(p));
            }
        };
        let fut_down = async {
            loop {
                let b = self.widget.wait_mouse_down().await;
                sender.output(WgpuCanvasEvent::MouseDown(b));
            }
        };
        let fut_up = async {
            loop {
                let b = self.widget.wait_mouse_up().await;
                sender.output(WgpuCanvasEvent::MouseUp(b));
            }
        };
        let fut_wheel = async {
            loop {
                let w = self.widget.wait_mouse_wheel().await;
                sender.output(WgpuCanvasEvent::MouseWheel(w));
            }
        };
        let fut_key_down = async {
            loop {
                let event = self.widget.wait_key_down().await;
                sender.output(WgpuCanvasEvent::KeyDown(event));
            }
        };
        let fut_key_up = async {
            loop {
                let event = self.widget.wait_key_up().await;
                sender.output(WgpuCanvasEvent::KeyUp(event));
            }
        };
        let fut_key_char = async {
            loop {
                let c = self.widget.wait_key_char().await;
                sender.output(WgpuCanvasEvent::KeyChar(c));
            }
        };
        futures_util::join!(
            fut_move,
            fut_down,
            fut_up,
            fut_wheel,
            fut_key_down,
            fut_key_up,
            fut_key_char
        )
        .0
    }
}

#[inherit_methods(from = "self.widget")]
impl ToolTip for WgpuCanvasBase {
    fn tooltip(&self) -> Result<String>;

    fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Visible for WgpuCanvasBase {
    fn is_visible(&self) -> Result<bool>;

    fn set_visible(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Enable for WgpuCanvasBase {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for WgpuCanvasBase {
    fn loc(&self) -> Result<Point>;

    fn set_loc(&mut self, p: Point) -> Result<()>;

    fn size(&self) -> Result<Size>;

    fn set_size(&mut self, s: Size) -> Result<()>;
}

impl Failable for WgpuCanvasBase {
    type Error = Error;
}

winio_handle::impl_as_widget!(WgpuCanvasBase, widget);

/// A frame acquired from a [`WgpuCanvas`].
#[derive(Debug)]
pub struct WgpuCanvasFrame {
    texture: Texture,
    view: TextureView,
    format: TextureFormat,
}

impl WgpuCanvasFrame {
    /// The texture to render into.
    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    /// A view of [`texture`][Self::texture], to be used as the render target.
    pub fn view(&self) -> &TextureView {
        &self.view
    }

    /// The format of [`view`][Self::view].
    pub fn format(&self) -> TextureFormat {
        self.format
    }
}

// SAFETY: it is actually `Send` + `Sync`, but we need to make rustc happy.
unsafe impl Send for WgpuCanvasFrame {}
unsafe impl Sync for WgpuCanvasFrame {}

/// Renders GPU graphics with [`wgpu`], accelerated by the native graphics API
/// of the platform.
///
/// The frames are acquired and presented by the UI thread, through
/// [`acquire`](Self::acquire) and [`present`](Self::present). To render from
/// another thread, use [`WgpuCanvasSync`](crate::widgets::WgpuCanvasSync)
/// instead.
///
/// ## Recommended backends
/// * Windows: DirectX 12
/// * macOS & iOS: Metal
/// * Android & Qt & GTK: Vulkan
///
/// ## Platform specific
/// * iOS: Simulator does not support [`wgpu`].
#[derive(Debug)]
pub struct WgpuCanvas {
    base: WgpuCanvasBase,
}

#[inherit_methods(from = "self.base")]
impl WgpuCanvas {
    /// Hand the GPU stack over to the canvas.
    ///
    /// Call it once, after the `device` has been created and before the first
    /// [`WgpuCanvas::acquire`].
    pub fn bind(&mut self, instance: &Instance, adapter: &Adapter);

    /// The presentation mode of the canvas.
    ///
    /// Returns `Some` when the canvas owns the native surface, and `None` when
    /// the windowing toolkit composites the frames itself, so the presentation
    /// timing is outside the control of winio.
    pub fn present_mode(&self) -> Option<PresentMode>;

    /// Set the presentation mode of the canvas.
    ///
    /// The new mode is applied at the next [`acquire`](Self::acquire). A frame
    /// that has already been acquired and is presented before then still uses
    /// the old mode. Unsupported modes fall back to
    /// [`PresentMode::AutoVsync`].
    pub fn set_present_mode(&mut self, mode: PresentMode);

    /// Acquire a frame to render into.
    ///
    /// The texture is owned by the canvas: it is recreated when the canvas is
    /// resized, and [`WgpuCanvas::present`] hands it over to the widget.
    ///
    /// Returns `None` if the backend cannot provide a canvas-owned texture.
    pub fn acquire(&mut self, device: &Device) -> Option<WgpuCanvasFrame>;

    /// Present the frame acquired by [`WgpuCanvas::acquire`].
    ///
    /// It must be called after the queue submission that renders into the
    /// frame is recorded.
    pub fn present(&mut self, queue: &Queue) -> Result<()>;
}

#[inherit_methods(from = "self.base")]
impl ToolTip for WgpuCanvas {
    fn tooltip(&self) -> Result<String>;

    fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;
}

#[inherit_methods(from = "self.base")]
impl Visible for WgpuCanvas {
    fn is_visible(&self) -> Result<bool>;

    fn set_visible(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.base")]
impl Enable for WgpuCanvas {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.base")]
impl Layoutable for WgpuCanvas {
    fn loc(&self) -> Result<Point>;

    fn set_loc(&mut self, p: Point) -> Result<()>;

    fn size(&self) -> Result<Size>;

    fn set_size(&mut self, s: Size) -> Result<()>;
}

impl Failable for WgpuCanvas {
    type Error = Error;
}

/// Events of [`WgpuCanvas`] and
/// [`WgpuCanvasSync`](crate::widgets::WgpuCanvasSync).
#[derive(Debug)]
#[non_exhaustive]
pub enum WgpuCanvasEvent {
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
    /// A keyboard key was pressed down.
    KeyDown(KeyCode),
    /// A keyboard key was released.
    KeyUp(KeyCode),
    /// A Unicode character was produced directly by a key press. Text
    /// containing multiple characters produces one event per character.
    KeyChar(char),
}

/// Messages of [`WgpuCanvas`].
#[derive(Debug)]
#[non_exhaustive]
pub enum WgpuCanvasMessage {
    /// Set the rect.
    SetRect(Rect),
    /// Set the enabled state.
    SetEnabled(bool),
    /// Set the visible state.
    SetVisible(bool),
    /// Set the tooltip.
    SetTooltip(String),
}

impl Component for WgpuCanvas {
    type Error = Error;
    type Event = WgpuCanvasEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = WgpuCanvasMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        Ok(Self {
            base: WgpuCanvasBase::new(init)?,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        self.base.start(sender).await
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            WgpuCanvasMessage::SetRect(rect) => {
                self.set_rect(rect)?;
                Ok(true)
            }
            WgpuCanvasMessage::SetEnabled(enabled) => {
                self.set_enabled(enabled)?;
                Ok(false)
            }
            WgpuCanvasMessage::SetVisible(visible) => {
                self.set_visible(visible)?;
                Ok(true)
            }
            WgpuCanvasMessage::SetTooltip(tooltip) => {
                self.set_tooltip(tooltip)?;
                Ok(false)
            }
        }
    }
}

winio_handle::impl_as_widget!(WgpuCanvas, base);
