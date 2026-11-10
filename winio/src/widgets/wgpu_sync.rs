//! A wgpu canvas that is rendered from another thread.

use std::{
    fmt,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread::{self, ThreadId},
};

use inherit_methods_macro::inherit_methods;
use wgpu::{Adapter, Device, Instance, PresentMode, Queue, Texture, TextureFormat, TextureView};
use winio_elm::{Component, ComponentSender};
use winio_handle::BorrowedContainer;
use winio_primitive::{Enable, Failable, Layoutable, Point, Size, ToolTip, Visible};

use super::wgpu::{WgpuCanvasBase, WgpuCanvasEvent, WgpuCanvasFrame};
use crate::sys::{Error, Result};

/// A [`WgpuCanvas`](super::WgpuCanvas) whose frames are rendered from another
/// thread.
///
/// The widget is still created and owned by the UI thread, but the frames are
/// acquired and presented through a [`WgpuCanvasSyncProxy`], which can be moved
/// to the rendering thread. See [`WgpuCanvasSync::proxy`].
#[derive(Debug)]
pub struct WgpuCanvasSync {
    base: WgpuCanvasBase,
    sender: ComponentSender<Self>,
    /// Whether a frame is in flight. Shared with the [`WgpuCanvasSyncFrame`]s.
    in_flight: Arc<AtomicBool>,
}

#[inherit_methods(from = "self.base")]
impl WgpuCanvasSync {
    /// Bind the canvas to the GPU stack, and create a proxy that renders it.
    ///
    /// The proxy is cheap to clone, and every clone renders the same canvas.
    /// It can be moved to another thread; the calls that need the native widget
    /// are forwarded to the UI thread, while the GPU work stays on the calling
    /// thread.
    pub fn proxy(&mut self, instance: &Instance, adapter: &Adapter) -> WgpuCanvasSyncProxy {
        self.base.bind(instance, adapter);
        WgpuCanvasSyncProxy {
            sender: self.sender.clone(),
            in_flight: self.in_flight.clone(),
            ui: thread::current().id(),
        }
    }

    /// The presentation mode of the canvas.
    ///
    /// Returns `Some` when the canvas owns the native surface, and `None` when
    /// the windowing toolkit composites the frames itself, so the presentation
    /// timing is outside the control of winio.
    pub fn present_mode(&self) -> Option<PresentMode>;

    /// Set the presentation mode of the canvas.
    ///
    /// The new mode is applied at the next
    /// [`acquire`](WgpuCanvasSyncProxy::acquire). Unsupported modes fall back
    /// to [`PresentMode::AutoVsync`].
    pub fn set_present_mode(&mut self, mode: PresentMode);
}

impl Failable for WgpuCanvasSync {
    type Error = Error;
}

/// A thread-safe handle that renders a [`WgpuCanvasSync`].
///
/// Cloning is cheap, and every clone refers to the same canvas. The proxy is
/// created with [`WgpuCanvasSync::proxy`].
#[derive(Clone)]
pub struct WgpuCanvasSyncProxy {
    sender: ComponentSender<WgpuCanvasSync>,
    in_flight: Arc<AtomicBool>,
    /// The thread that owns the canvas, where waiting would deadlock.
    ui: ThreadId,
}

impl WgpuCanvasSyncProxy {
    /// Acquire a frame to render into.
    ///
    /// The frame is acquired on the UI thread and handed over to the calling
    /// thread, which can then record and submit the commands itself. It blocks
    /// until the UI thread has serviced the request.
    ///
    /// # Errors
    ///
    /// * [`WgpuCanvasSyncError::Closed`]: the canvas has been dropped.
    /// * [`WgpuCanvasSyncError::WrongThread`]: called from the UI thread.
    /// * [`WgpuCanvasSyncError::Unavailable`]: the backend cannot provide a
    ///   frame, for example because the previous one is still in flight.
    pub fn acquire(
        &self,
        device: &Device,
    ) -> std::result::Result<WgpuCanvasSyncFrame, WgpuCanvasSyncError> {
        struct SendWrapper(Device);

        // SAFETY: `Device` is `Send`, but we need to make rustc happy.
        unsafe impl Send for SendWrapper {}

        let device = SendWrapper(device.clone());
        let in_flight = self.in_flight.clone();
        let probe = in_flight.clone();
        let frame = self.call(move |base| {
            if probe.load(Ordering::Acquire) {
                return None;
            }
            let device = device;
            let frame = base.acquire(&device.0)?;
            probe.store(true, Ordering::Release);
            Some(frame)
        })?;
        let frame = frame.ok_or(WgpuCanvasSyncError::Unavailable)?;
        Ok(WgpuCanvasSyncFrame {
            proxy: self.clone(),
            frame,
            in_flight,
        })
    }

    /// Run `f` on the UI thread with exclusive access to the canvas.
    fn call<T: Send + 'static>(
        &self,
        f: impl FnOnce(&mut WgpuCanvasBase) -> T + Send + 'static,
    ) -> std::result::Result<T, WgpuCanvasSyncError> {
        if thread::current().id() == self.ui {
            return Err(WgpuCanvasSyncError::WrongThread);
        }
        let (tx, rx) = mpsc::sync_channel(1);
        self.sender
            .post(WgpuCanvasSyncMessage::Call(Call(Box::new(move |base| {
                let _ = tx.send(f(base));
            }))));
        rx.recv().map_err(|_| WgpuCanvasSyncError::Closed)
    }
}

impl fmt::Debug for WgpuCanvasSyncProxy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WgpuCanvasSyncProxy")
            .finish_non_exhaustive()
    }
}

/// A frame acquired by [`WgpuCanvasSyncProxy::acquire`].
///
/// The frame is presented with [`WgpuCanvasSyncFrame::present`]. Dropping it
/// instead skips the frame, so that the canvas can hand out the next one.
#[derive(Debug)]
#[must_use = "the frame is presented with `present`, or dropped to skip it"]
pub struct WgpuCanvasSyncFrame {
    proxy: WgpuCanvasSyncProxy,
    frame: WgpuCanvasFrame,
    in_flight: Arc<AtomicBool>,
}

#[inherit_methods(from = "self.frame")]
impl WgpuCanvasSyncFrame {
    /// The texture to render into.
    pub fn texture(&self) -> &Texture;

    /// A view of [`texture`][Self::texture], to be used as the render target.
    pub fn view(&self) -> &TextureView;

    /// The format of [`view`][Self::view].
    pub fn format(&self) -> TextureFormat;

    /// Present the frame, after the queue submission that renders into it is
    /// recorded.
    ///
    /// # Errors
    ///
    /// Returns [`WgpuCanvasSyncError::Closed`] when the canvas has been
    /// dropped, and [`WgpuCanvasSyncError::Backend`] when the backend fails to
    /// present.
    pub fn present(self, queue: &Queue) -> std::result::Result<(), WgpuCanvasSyncError> {
        struct SendWrapper(Queue);

        // SAFETY: `Queue` is `Send`, but we need to make rustc happy.
        unsafe impl Send for SendWrapper {}

        let queue = SendWrapper(queue.clone());
        self.proxy
            .call(move |base| {
                let queue = queue;
                base.present(&queue.0)
            })?
            .map_err(WgpuCanvasSyncError::Backend)
    }
}

impl Drop for WgpuCanvasSyncFrame {
    fn drop(&mut self) {
        self.in_flight.store(false, Ordering::Release);
    }
}

/// The errors of a [`WgpuCanvasSyncProxy`].
#[derive(Debug)]
#[non_exhaustive]
pub enum WgpuCanvasSyncError {
    /// The canvas has been dropped.
    Closed,
    /// The proxy was used from the UI thread, where waiting would deadlock.
    WrongThread,
    /// The backend cannot provide a frame at the moment.
    Unavailable,
    /// The backend failed to present the frame.
    Backend(Error),
}

impl fmt::Display for WgpuCanvasSyncError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Closed => f.write_str("the canvas has been closed"),
            Self::WrongThread => f.write_str("the proxy cannot be used from the UI thread"),
            Self::Unavailable => f.write_str("no frame is available"),
            Self::Backend(e) => write!(f, "the backend failed: {e}"),
        }
    }
}

impl std::error::Error for WgpuCanvasSyncError {}

/// A call forwarded to the UI thread by a [`WgpuCanvasSyncProxy`].
#[doc(hidden)]
pub struct Call(Box<dyn FnOnce(&mut WgpuCanvasBase) + Send>);

impl fmt::Debug for Call {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Call")
    }
}

/// Messages of [`WgpuCanvasSync`].
#[derive(Debug)]
#[non_exhaustive]
pub enum WgpuCanvasSyncMessage {
    /// Run a call forwarded by a [`WgpuCanvasSyncProxy`].
    #[doc(hidden)]
    Call(Call),
}

#[inherit_methods(from = "self.base")]
impl ToolTip for WgpuCanvasSync {
    fn tooltip(&self) -> Result<String>;

    fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;
}

#[inherit_methods(from = "self.base")]
impl Visible for WgpuCanvasSync {
    fn is_visible(&self) -> Result<bool>;

    fn set_visible(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.base")]
impl Enable for WgpuCanvasSync {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.base")]
impl Layoutable for WgpuCanvasSync {
    fn loc(&self) -> Result<Point>;

    fn set_loc(&mut self, p: Point) -> Result<()>;

    fn size(&self) -> Result<Size>;

    fn set_size(&mut self, s: Size) -> Result<()>;
}

impl Component for WgpuCanvasSync {
    type Error = Error;
    type Event = WgpuCanvasEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = WgpuCanvasSyncMessage;

    async fn init(init: Self::Init<'_>, sender: &ComponentSender<Self>) -> Result<Self> {
        Ok(Self {
            base: WgpuCanvasBase::new(init)?,
            sender: sender.clone(),
            in_flight: Arc::new(AtomicBool::new(false)),
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
            WgpuCanvasSyncMessage::Call(Call(f)) => {
                f(&mut self.base);
                Ok(false)
            }
        }
    }
}

winio_handle::impl_as_widget!(WgpuCanvasSync, base);

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn proxy_and_frame_are_send_sync() {
        assert_send_sync::<WgpuCanvasSyncProxy>();
        assert_send_sync::<WgpuCanvasSyncFrame>();
    }
}
