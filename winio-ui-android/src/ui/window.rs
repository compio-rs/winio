//! Android window widget, based on JNI and FrameLayout

use std::rc::Rc;

use inherit_methods_macro::inherit_methods;
use jni::{jni_sig, strings::JNIString};
use winio_callback::Callback;
use winio_handle::{AsWindow, BorrowedWindow};
use winio_primitive::{Point, Size};

use crate::{BaseWidget, GlobalRef, RESIZE_SLAB, Result, current_activity, vm_exec};

#[derive(Debug)]
pub struct Window {
    inner: BaseWidget,
    activity: GlobalRef,
    on_resize: Rc<Callback>,
    resize_index: usize,
}

#[inherit_methods(from = "self.inner")]
impl Window {
    const WINDOW_CLASS: &'static str = "android/widget/FrameLayout";

    pub fn new() -> Result<Self> {
        vm_exec(move |env| {
            let act = current_activity()?;
            let window = env.new_object(
                JNIString::new(Self::WINDOW_CLASS),
                jni_sig!("(Landroid/content/Context;)V"),
                &[act.as_obj().into()],
            )?;
            env.call_method(
                act.as_obj(),
                jni::jni_str!("setContentView"),
                jni_sig!("(Landroid/view/View;)V"),
                &[(&window).into()],
            )?
            .v()?;
            let inner = env.new_global_ref(window)?;
            let on_resize = Rc::new(Callback::new());
            let resize_index = RESIZE_SLAB.with_borrow_mut(|s| s.insert(on_resize.clone()));
            Ok(Self {
                inner: inner.into(),
                activity: act,
                on_resize,
                resize_index,
            })
        })
    }

    pub fn client_size(&self) -> Result<Size> {
        self.size()
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, visible: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, size: Size) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, text: impl AsRef<str>) -> Result<()>;

    pub async fn wait_close(&self) {
        std::future::pending().await
    }

    pub async fn wait_move(&self) {
        std::future::pending().await
    }

    pub async fn wait_size(&self) {
        self.on_resize.wait().await;
    }

    pub async fn wait_theme_changed(&self) {
        std::future::pending().await
    }
}

impl AsWindow for Window {
    fn as_window(&self) -> BorrowedWindow<'_> {
        unsafe { BorrowedWindow::android(&self.activity) }
    }
}

winio_handle::impl_as_container!(Window, inner);

impl Drop for Window {
    fn drop(&mut self) {
        RESIZE_SLAB.with_borrow_mut(|s| s.remove(self.resize_index));
    }
}
