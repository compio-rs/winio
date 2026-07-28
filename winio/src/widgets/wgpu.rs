use inherit_methods_macro::inherit_methods;
use wgpu::{CreateSurfaceError, Instance, Surface};
use winio_elm::{Child, Component, ComponentSender, PropSink, PropSinkEvent, start};
use winio_handle::BorrowedContainer;
use winio_primitive::{
    Enable, Failable, Layoutable, MouseButton, Point, Size, ToolTip, Vector, Visible,
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
    enabled_prop: Child<PropSink<bool>>,
    visible_prop: Child<PropSink<bool>>,
    tooltip_prop: Child<PropSink<String>>,
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

    /// Property for [`Enable::set_enabled`].
    pub fn enabled_prop(&self) -> &PropSink<bool> {
        &self.enabled_prop
    }

    /// Property for [`Visible::set_visible`].
    pub fn visible_prop(&self) -> &PropSink<bool> {
        &self.visible_prop
    }

    /// Property for [`ToolTip::set_tooltip`].
    pub fn tooltip_prop(&self) -> &PropSink<String> {
        &self.tooltip_prop
    }
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

    fn set_size(&mut self, v: Size) -> Result<()>;
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
    /// No operation.
    Noop,
    /// The enabled state has been changed.
    ChangeEnabled,
    /// The visible state has been changed.
    ChangeVisible,
    /// The tooltip has been changed.
    ChangeTooltip,
}

impl Component for WgpuCanvas {
    type Error = Error;
    type Event = WgpuCanvasEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = WgpuCanvasMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::WgpuCanvas::new(init)?;
        let Ok(enabled_prop) = Child::<PropSink<bool>>::init(true).await;
        let Ok(visible_prop) = Child::<PropSink<bool>>::init(true).await;
        let Ok(tooltip_prop) = Child::<PropSink<String>>::init(String::new()).await;
        Ok(Self {
            widget,
            enabled_prop,
            visible_prop,
            tooltip_prop,
        })
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
        let fut_props = async {
            start! {
                sender, default: WgpuCanvasMessage::Noop,
                self.enabled_prop => { PropSinkEvent::Changed => WgpuCanvasMessage::ChangeEnabled },
                self.visible_prop => { PropSinkEvent::Changed => WgpuCanvasMessage::ChangeVisible },
                self.tooltip_prop => { PropSinkEvent::Changed => WgpuCanvasMessage::ChangeTooltip },
            }
        };
        futures_util::future::join5(fut_move, fut_down, fut_up, fut_wheel, fut_props)
            .await
            .0
    }

    async fn update_children(&mut self) -> Result<bool> {
        let Ok(r0) = self.enabled_prop.update().await;
        let Ok(r1) = self.visible_prop.update().await;
        let Ok(r2) = self.tooltip_prop.update().await;
        Ok(r0 || r1 || r2)
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            WgpuCanvasMessage::Noop => Ok(false),
            WgpuCanvasMessage::ChangeEnabled => {
                self.widget.set_enabled(**self.enabled_prop)?;
                Ok(true)
            }
            WgpuCanvasMessage::ChangeVisible => {
                self.widget.set_visible(**self.visible_prop)?;
                Ok(true)
            }
            WgpuCanvasMessage::ChangeTooltip => {
                self.widget.set_tooltip(self.tooltip_prop.get())?;
                Ok(true)
            }
        }
    }
}

winio_handle::impl_as_widget!(WgpuCanvas, widget);
