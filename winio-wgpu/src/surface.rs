//! Canvas-owned swapchain for backends that have a native surface per widget.

use compio_log::warn;
use wgpu::{
    Adapter, CurrentSurfaceTexture, Device, Instance, PresentMode, Queue, Surface,
    SurfaceConfiguration, SurfaceTarget, SurfaceTexture, Texture, TextureFormat, TextureView,
    rwh::{
        DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
        RawWindowHandle, WindowHandle,
    },
};

/// A render target that presents through a native surface owned by the canvas.
///
/// The surface is created lazily on the first [`acquire`](Self::acquire), so
/// the widget only has to be able to provide its native handles by then.
#[derive(Debug)]
pub struct SurfaceCanvas {
    state: Option<State>,
    present_mode: PresentMode,
}

impl Default for SurfaceCanvas {
    fn default() -> Self {
        Self {
            state: None,
            present_mode: PresentMode::AutoVsync,
        }
    }
}

#[derive(Debug)]
struct State {
    instance: Instance,
    adapter: Adapter,
    frame: Option<SurfaceTexture>,
    surface: Option<Surface<'static>>,
    format: TextureFormat,
    configured: Option<ConfigKey>,
}

/// The parameters that require a new call to [`Surface::configure`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ConfigKey {
    size: (u32, u32),
    present_mode: PresentMode,
}

impl SurfaceCanvas {
    /// Create a canvas without a surface.
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether [`bind`](Self::bind) has been called.
    pub fn is_bound(&self) -> bool {
        self.state.is_some()
    }

    /// The presentation mode of the surface.
    ///
    /// It defaults to [`PresentMode::AutoVsync`]. Modes that the adapter does
    /// not support are replaced by `AutoVsync` when the surface is configured.
    pub fn present_mode(&self) -> PresentMode {
        self.present_mode
    }

    /// Set the presentation mode of the surface.
    ///
    /// The new mode is used the next time the surface is configured, which
    /// happens at the first [`acquire`](Self::acquire) after the change. Modes
    /// that the adapter does not support fall back to
    /// [`PresentMode::AutoVsync`].
    pub fn set_present_mode(&mut self, mode: PresentMode) {
        self.present_mode = mode;
    }

    /// Remember the GPU stack of the application.
    ///
    /// It is needed to create and configure the surface, which happens on the
    /// first [`acquire`](Self::acquire).
    pub fn bind(&mut self, instance: &Instance, adapter: &Adapter) {
        self.state = Some(State {
            instance: instance.clone(),
            adapter: adapter.clone(),
            frame: None,
            surface: None,
            format: TextureFormat::Bgra8Unorm,
            configured: None,
        });
    }

    /// Get the underlying surface, if it has been created by
    /// [`acquire`](Self::acquire) or [`acquire_with`](Self::acquire_with).
    pub fn surface(&self) -> Option<&wgpu::Surface<'static>> {
        self.state.as_ref()?.surface.as_ref()
    }

    /// Acquire the next frame of `size` physical pixels.
    ///
    /// `window` is the native handle of the widget, used to create the surface.
    /// Returns `None` when the canvas is not bound yet, or when no frame is
    /// available (the surface is occluded, out of date or lost).
    ///
    /// # Safety
    ///
    /// `window` and the object behind `display` must remain valid until this
    /// canvas, and the surface it creates, is dropped.
    pub unsafe fn acquire(
        &mut self,
        device: &Device,
        window: RawWindowHandle,
        display: RawDisplayHandle,
        size: (u32, u32),
    ) -> Option<(Texture, TextureView, TextureFormat)> {
        unsafe {
            self.acquire_with(device, size, |instance| {
                instance
                    .create_surface(SurfaceTarget::from(WindowHandleWrapper(window, display)))
                    .ok()
            })
        }
    }

    /// Like [`acquire`](Self::acquire), but the surface is created by `create`.
    ///
    /// This is for backends that need a specialized surface target.
    ///
    /// # Safety
    ///
    /// The surface returned by `create` must only reference native handles
    /// that remain valid until this canvas, and the surface, is dropped.
    pub unsafe fn acquire_with(
        &mut self,
        device: &Device,
        size: (u32, u32),
        create: impl FnOnce(&Instance) -> Option<Surface<'static>>,
    ) -> Option<(Texture, TextureView, TextureFormat)> {
        let mut present_mode = self.present_mode;
        let state = self.state.as_mut()?;

        if state.surface.is_none() {
            let surface = create(&state.instance)?;
            let caps = surface.get_capabilities(&state.adapter);
            state.format = caps
                .formats
                .iter()
                .copied()
                .find(TextureFormat::is_srgb)
                .or_else(|| caps.formats.first().copied())?;
            state.configured = None;
            state.surface = Some(surface);
        }

        let format = state.format;
        let mut lost = false;
        let acquired = {
            let surface = state.surface.as_ref()?;
            present_mode = resolve_present_mode(
                &surface.get_capabilities(&state.adapter).present_modes,
                present_mode,
            );

            let key = ConfigKey { size, present_mode };
            if state.configured != Some(key) {
                // `configure` panics if a `SurfaceTexture` of the surface is
                // still alive. This can happen when the size or the present
                // mode changed after a frame was acquired but not presented.
                state.frame = None;
                surface.configure(
                    device,
                    &SurfaceConfiguration {
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                        format,
                        color_space: wgpu::SurfaceColorSpace::Auto,
                        view_formats: vec![],
                        alpha_mode: wgpu::CompositeAlphaMode::Auto,
                        width: size.0,
                        height: size.1,
                        desired_maximum_frame_latency: 2,
                        present_mode,
                    },
                );
                state.configured = Some(key);
            }

            match surface.get_current_texture() {
                CurrentSurfaceTexture::Success(frame) => Some(frame),
                CurrentSurfaceTexture::Lost => {
                    lost = true;
                    None
                }
                CurrentSurfaceTexture::Outdated => {
                    state.configured = None;
                    None
                }
                _ => None,
            }
        };
        self.present_mode = present_mode;
        if lost {
            state.frame = None;
            state.surface = None;
            state.configured = None;
            return None;
        }

        let frame = acquired?;
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(format),
            ..Default::default()
        });
        let texture = frame.texture.clone();
        state.frame = Some(frame);
        Some((texture, view, format))
    }

    /// Present the frame acquired by [`acquire`](Self::acquire).
    ///
    /// Returns whether a frame was presented.
    pub fn present(&mut self, queue: &Queue) -> bool {
        match self.state.as_mut().and_then(|state| state.frame.take()) {
            Some(frame) => {
                queue.present(frame);
                true
            }
            None => false,
        }
    }
}

/// Select a present mode that the surface can be configured with.
fn resolve_present_mode(supported: &[PresentMode], requested: PresentMode) -> PresentMode {
    match requested {
        PresentMode::AutoVsync | PresentMode::AutoNoVsync => requested,
        _ if supported.contains(&requested) => requested,
        _ => {
            warn!(
                "WGPU canvas: present mode {requested:?} is not supported by the surface, falling \
                 back to AutoVsync"
            );
            PresentMode::AutoVsync
        }
    }
}

struct WindowHandleWrapper(RawWindowHandle, RawDisplayHandle);

unsafe impl Send for WindowHandleWrapper {}
unsafe impl Sync for WindowHandleWrapper {}

impl HasWindowHandle for WindowHandleWrapper {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        Ok(unsafe { WindowHandle::borrow_raw(self.0) })
    }
}

impl HasDisplayHandle for WindowHandleWrapper {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        Ok(unsafe { DisplayHandle::borrow_raw(self.1) })
    }
}
