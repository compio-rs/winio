use std::sync::Arc;

use android_activity::ndk::native_window::NativeWindow;
use inherit_methods_macro::inherit_methods;
use jni_min_helper::DynamicProxy;
use wgpu::{
    CreateSurfaceError, Instance, Surface, SurfaceTarget,
    rwh::{
        AndroidNdkWindowHandle, DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle,
        RawWindowHandle, WindowHandle,
    },
};
use winio_callback::SyncCallback;
use winio_handle::AsContainer;
use winio_primitive::{MouseButton, Point, Size, Vector};

use crate::{
    BaseWidget, Result, current_activity, java::android::view::SurfaceView, view_touch_proxy,
    vm_exec,
};

#[derive(Debug)]
pub struct WgpuCanvas {
    inner: BaseWidget<SurfaceView<'static>>,
    on_down: Arc<SyncCallback<MouseButton>>,
    on_up: Arc<SyncCallback<MouseButton>>,
    on_move: Arc<SyncCallback<Point>>,
    on_scroll: Arc<SyncCallback<Vector>>,
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
                inner,
                on_down,
                on_up,
                on_move,
                on_scroll,
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

    pub fn create_surface(
        &self,
        instance: &Instance,
    ) -> std::result::Result<Surface<'static>, CreateSurfaceError> {
        let native_window = self
            .native_window()
            .ok_or_else(|| wgpu::wgc::instance::CreateSurfaceError::MissingDisplayHandle)?;

        let handle =
            RawWindowHandle::AndroidNdk(AndroidNdkWindowHandle::new(native_window.ptr().cast()));

        struct WindowHandleWrapper(RawWindowHandle);

        unsafe impl Send for WindowHandleWrapper {}
        unsafe impl Sync for WindowHandleWrapper {}

        impl HasWindowHandle for WindowHandleWrapper {
            fn window_handle(&self) -> std::result::Result<WindowHandle<'_>, HandleError> {
                Ok(unsafe { WindowHandle::borrow_raw(self.0) })
            }
        }

        impl HasDisplayHandle for WindowHandleWrapper {
            fn display_handle(&self) -> std::result::Result<DisplayHandle<'_>, HandleError> {
                Ok(DisplayHandle::android())
            }
        }

        let target = SurfaceTarget::from(WindowHandleWrapper(handle));

        instance.create_surface(target)
    }
}

winio_handle::impl_as_widget!(WgpuCanvas, inner);
