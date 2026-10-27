use std::sync::Arc;

use android_activity::ndk::native_window::NativeWindow;
use inherit_methods_macro::inherit_methods;
use jni_min_helper::DynamicProxy;
use wgpu::{
    Adapter, Instance,
    rwh::{AndroidDisplayHandle, AndroidNdkWindowHandle, RawDisplayHandle, RawWindowHandle},
};
use winio_callback::SyncCallback;
use winio_handle::AsContainer;
use winio_primitive::{KeyCode, MouseButton, Point, Size, Vector};
use winio_wgpu::SurfaceCanvas;

use crate::{
    BaseWidget, Result, current_activity, java::android::view::SurfaceView, platform::Keyboard,
    view_touch_proxy, vm_exec,
};

#[derive(Debug)]
pub struct WgpuCanvas {
    surface: SurfaceCanvas,
    inner: BaseWidget<SurfaceView<'static>>,
    on_down: Arc<SyncCallback<MouseButton>>,
    on_up: Arc<SyncCallback<MouseButton>>,
    on_move: Arc<SyncCallback<Point>>,
    on_scroll: Arc<SyncCallback<Vector>>,
    keyboard: Keyboard,
    #[allow(dead_code)]
    touch_proxy: DynamicProxy,
}

#[inherit_methods(from = "self.inner")]
impl WgpuCanvas {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        vm_exec(|env| {
            let act = current_activity(env)?;
            let widget = SurfaceView::new(env, &act)?;
            let inner = BaseWidget::new_with_env(env, parent.as_container(), widget)?;
            let keyboard = Keyboard::new(env, inner.as_view())?;
            let on_down = Arc::new(SyncCallback::new());
            let on_up = Arc::new(SyncCallback::new());
            let on_move = Arc::new(SyncCallback::new());
            let on_scroll = Arc::new(SyncCallback::new());
            let touch_proxy = view_touch_proxy(
                env,
                inner.as_view(),
                on_down.clone(),
                on_up.clone(),
                on_move.clone(),
                on_scroll.clone(),
            )?;
            Ok(Self {
                surface: SurfaceCanvas::new(),
                inner,
                on_down,
                on_up,
                on_move,
                on_scroll,
                keyboard,
                touch_proxy,
            })
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, visible: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub async fn wait_mouse_move(&self) -> Point {
        self.on_move.wait().await
    }

    pub async fn wait_mouse_down(&self) -> MouseButton {
        self.on_down.wait().await
    }

    pub async fn wait_mouse_up(&self) -> MouseButton {
        self.on_up.wait().await
    }

    pub async fn wait_mouse_wheel(&self) -> Vector {
        self.on_scroll.wait().await
    }

    pub async fn wait_key_down(&self) -> KeyCode {
        self.keyboard.wait_key_down().await
    }

    pub async fn wait_key_up(&self) -> KeyCode {
        self.keyboard.wait_key_up().await
    }

    pub async fn wait_key_char(&self) -> char {
        self.keyboard.wait_key_char().await
    }

    fn native_window(&self) -> Option<NativeWindow> {
        vm_exec(|env| {
            let holder = self.inner.get_holder(env)?;
            if holder.is_null() {
                return Ok(None);
            }
            let surface = holder.get_surface(env)?;
            if surface.is_null() {
                return Ok(None);
            }
            let native_window =
                unsafe { NativeWindow::from_surface(env.get_raw().cast(), surface.as_raw()) };
            Result::Ok(native_window)
        })
        .ok()
        .flatten()
    }

    pub fn acquire(
        &mut self,
        device: &wgpu::Device,
    ) -> Option<(wgpu::Texture, wgpu::TextureView, wgpu::TextureFormat)> {
        let size = self.inner.size().ok()?;
        let (width, height) = crate::platform::dpi::physical_size(size).ok()?;
        let width = (width as u32).max(1);
        let height = (height as u32).max(1);
        let native_window = self.native_window()?;
        let window =
            RawWindowHandle::AndroidNdk(AndroidNdkWindowHandle::new(native_window.ptr().cast()));
        let display = RawDisplayHandle::Android(AndroidDisplayHandle::new());
        // SAFETY: `surface` is declared before `inner`, so it is dropped before
        // the view that owns the native window.
        unsafe {
            self.surface
                .acquire(device, window, display, (width, height))
        }
    }

    pub fn bind(&mut self, instance: &Instance, adapter: &Adapter) {
        self.surface.bind(instance, adapter);
    }

    pub fn present(&mut self, queue: &wgpu::Queue) -> Result<()> {
        self.surface.present(queue);
        Ok(())
    }
}

winio_handle::impl_as_widget!(WgpuCanvas, inner);
