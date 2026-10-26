use std::cell::{Cell, RefCell};

use compio_log::error;
use gtk4::{
    ContentFit, GraphicsOffload, GraphicsOffloadEnabled, Overlay, Picture, glib::object::Cast,
    prelude::*,
};
use inherit_methods_macro::inherit_methods;
use winio_handle::{AsContainer, AsWidget};
use winio_primitive::{KeyCode, MouseButton, Point, Size, Vector};

use crate::{CanvasImpl, Result};

mod target;

/// A GPU canvas composited by GTK.
#[derive(Debug)]
pub struct WgpuCanvas {
    handle: CanvasImpl,
    picture: Picture,
    target: RefCell<Option<target::Target>>,
    /// Size of the last failed attempt, to avoid retrying every frame.
    failed: Cell<Option<(u32, u32)>>,
    /// Whether a presentation error has already been reported.
    present_failed: Cell<bool>,
}

#[inherit_methods(from = "self.handle")]
impl WgpuCanvas {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let area = gtk4::DrawingArea::new();
        area.set_hexpand(true);
        area.set_vexpand(true);

        let picture = Picture::new();
        picture.set_content_fit(ContentFit::Fill);

        let offload = GraphicsOffload::new(Some(&picture));
        offload.set_enabled(GraphicsOffloadEnabled::Enabled);
        offload.set_can_target(false);
        offload.set_hexpand(true);
        offload.set_vexpand(true);
        offload.set_halign(gtk4::Align::Fill);
        offload.set_valign(gtk4::Align::Fill);

        let overlay = Overlay::new();
        overlay.set_child(Some(&area));
        overlay.add_overlay(&offload);

        let handle = CanvasImpl::new(parent, area, overlay.upcast())?;

        Ok(Self {
            handle,
            picture,
            target: RefCell::new(None),
            failed: Cell::new(None),
            present_failed: Cell::new(false),
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, s: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub async fn wait_mouse_down(&self) -> MouseButton {
        self.handle.wait_mouse_down().await
    }

    pub async fn wait_mouse_up(&self) -> MouseButton {
        self.handle.wait_mouse_up().await
    }

    pub async fn wait_mouse_move(&self) -> Point {
        self.handle.wait_mouse_move().await
    }

    pub async fn wait_mouse_wheel(&self) -> Vector {
        self.handle.wait_mouse_wheel().await
    }

    pub async fn wait_key_down(&self) -> KeyCode {
        self.handle.wait_key_down().await
    }

    pub async fn wait_key_up(&self) -> KeyCode {
        self.handle.wait_key_up().await
    }

    pub async fn wait_key_char(&self) -> char {
        self.handle.wait_key_char().await
    }

    pub fn acquire(
        &self,
        device: &wgpu::Device,
    ) -> Option<(
        wgpu::Texture,
        wgpu::TextureView,
        wgpu::TextureFormat,
        (u32, u32),
    )> {
        let size = self.handle.size().ok()?;
        let scale = self.handle.as_widget().to_gtk().scale_factor() as f64;
        let width = ((size.width * scale).round() as u32).max(1);
        let height = ((size.height * scale).round() as u32).max(1);
        let wanted = (width, height);

        if self.failed.get() == Some(wanted) {
            return None;
        }

        let mut target = self.target.borrow_mut();
        if target.as_ref().map(|t| t.size()) != Some(wanted) {
            *target = None;
            match target::Target::new(device, wanted) {
                Ok(t) => *target = Some(t),
                Err(e) => {
                    error!("WGPU canvas: cannot create a render target: {e}");
                    self.failed.set(Some(wanted));
                    return None;
                }
            }
        }
        let target = target.as_ref()?;
        Some((
            target.texture().clone(),
            target.view().clone(),
            target.format(),
            target.size(),
        ))
    }

    pub fn present(&self, queue: &wgpu::Queue) -> Result<()> {
        let target = self.target.borrow();
        if let Some(target) = target.as_ref() {
            if let Err(e) = target.present(&self.picture, queue) {
                if !self.present_failed.replace(true) {
                    error!("WGPU canvas: cannot present a frame: {e}");
                }
            }
        }
        Ok(())
    }
}

winio_handle::impl_as_widget!(WgpuCanvas, handle);
