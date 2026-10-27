//! Canvas-owned swapchain for backends that have a native surface per widget.

use wgpu::{
    Adapter, CurrentSurfaceTexture, Device, Instance, Queue, Surface, SurfaceConfiguration,
    SurfaceTarget, SurfaceTexture, Texture, TextureFormat, TextureView,
    rwh::{
        DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
        RawWindowHandle, WindowHandle,
    },
};

/// A render target that presents through a native surface owned by the canvas.
///
/// The surface is created lazily on the first [`acquire`](Self::acquire), so
/// the widget only has to be able to provide its native handles by then.
#[derive(Debug, Default)]
pub struct SurfaceCanvas {
    state: Option<State>,
}

#[derive(Debug)]
struct State {
    instance: Instance,
    adapter: Adapter,
    surface: Option<Surface<'static>>,
    format: TextureFormat,
    configured: (u32, u32),
    frame: Option<SurfaceTexture>,
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

    /// Remember the GPU stack of the application.
    ///
    /// It is needed to create and configure the surface, which happens on the
    /// first [`acquire`](Self::acquire).
    pub fn bind(&mut self, instance: &Instance, adapter: &Adapter) {
        self.state = Some(State {
            instance: instance.clone(),
            adapter: adapter.clone(),
            surface: None,
            format: TextureFormat::Bgra8Unorm,
            configured: (0, 0),
            frame: None,
        });
    }

    /// Acquire the next frame of `size` physical pixels.
    ///
    /// `window` is the native handle of the widget, used to create the surface.
    /// Returns `None` when the canvas is not bound yet, or when no frame is
    /// available (the surface is occluded, out of date or lost).
    pub fn acquire(
        &mut self,
        device: &Device,
        window: RawWindowHandle,
        display: RawDisplayHandle,
        size: (u32, u32),
    ) -> Option<(Texture, TextureView, TextureFormat, (u32, u32))> {
        self.acquire_with(device, size, |instance| {
            instance
                .create_surface(SurfaceTarget::from(WindowHandleWrapper(window, display)))
                .ok()
        })
    }

    /// Like [`acquire`](Self::acquire), but the surface is created by `create`.
    ///
    /// This is for backends that need a specialized surface target.
    pub fn acquire_with(
        &mut self,
        device: &Device,
        size: (u32, u32),
        create: impl FnOnce(&Instance) -> Option<Surface<'static>>,
    ) -> Option<(Texture, TextureView, TextureFormat, (u32, u32))> {
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
            state.configured = (0, 0);
            state.surface = Some(surface);
        }

        let format = state.format;
        let mut lost = false;
        let acquired = {
            let surface = state.surface.as_ref()?;
            if state.configured != size {
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
                        present_mode: wgpu::PresentMode::AutoVsync,
                    },
                );
            }
            match surface.get_current_texture() {
                CurrentSurfaceTexture::Success(frame) => Some(frame),
                CurrentSurfaceTexture::Lost => {
                    lost = true;
                    None
                }
                _ => None,
            }
        };
        state.configured = size;
        if lost {
            state.surface = None;
            return None;
        }

        let frame = acquired?;
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(format),
            ..Default::default()
        });
        let texture = frame.texture.clone();
        state.frame = Some(frame);
        Some((texture, view, format, size))
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
