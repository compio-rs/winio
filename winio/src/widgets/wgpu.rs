use inherit_methods_macro::inherit_methods;
use wgpu::{CreateSurfaceError, Instance, Surface};
use winio_elm::{Component, ComponentSender};
use winio_handle::BorrowedContainer;
use winio_primitive::{
    Enable, Failable, Layoutable, MouseButton, Point, Rect, Size, ToolTip, Vector, Visible,
};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A canvas for [`wgpu`].
///
/// ## Recommended backends
/// * Windows: DirectX 12
/// * macOS & iOS: Metal
/// * Android: Vulkan
///
/// ## Platform specific
/// * iOS: Simulator do not support [`wgpu`].
/// * Android: Simulator might not work correctly; real devices work fine.
/// * Qt & GTK: Might not work correctly.
#[derive(Debug)]
pub struct WgpuCanvas {
    widget: sys::WgpuCanvas,
}

#[inherit_methods(from = "self.widget")]
impl WgpuCanvas {
    /// Create [`Surface`] to render on this canvas.
    ///
    /// This method returns an error if the canvas is not yet ready to create a
    /// surface, e.g. it is not yet visible.
    pub fn create_surface(
        &self,
        instance: &Instance,
    ) -> std::result::Result<Surface<'static>, CreateSurfaceError>;
}

impl Failable for WgpuCanvas {
    type Error = Error;
}

#[inherit_methods(from = "self.widget")]
impl ToolTip for WgpuCanvas {
    fn tooltip(&self) -> Result<String>;

    fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Visible for WgpuCanvas {
    fn is_visible(&self) -> Result<bool>;

    fn set_visible(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Enable for WgpuCanvas {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for WgpuCanvas {
    fn loc(&self) -> Result<Point>;

    fn set_loc(&mut self, p: Point) -> Result<()>;

    fn size(&self) -> Result<Size>;

    fn set_size(&mut self, s: Size) -> Result<()>;
}

/// Events of [`WgpuCanvas`].
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
        let widget = sys::WgpuCanvas::new(init)?;
        Ok(Self { widget })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
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
        futures_util::future::join4(fut_move, fut_down, fut_up, fut_wheel)
            .await
            .0
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

winio_handle::impl_as_widget!(WgpuCanvas, widget);
