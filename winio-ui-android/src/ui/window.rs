//! Android window widget, based on JNI and FrameLayout

use inherit_methods_macro::inherit_methods;
use jni::{jni_sig, strings::JNIString};
use winio_handle::{AsWindow, BorrowedContainer, BorrowedWindow};
use winio_primitive::{Point, Size};

use crate::{BaseWidget, Result, current_activity, vm_exec};

#[derive(Debug)]
pub struct Window {
    inner: BaseWidget,
}

#[inherit_methods(from = "self.inner")]
impl Window {
    const WINDOW_CLASS: &'static str = "android/widget/FrameLayout";

    pub fn new() -> Result<Self> {
        let inner = vm_exec(move |env| {
            let act = current_activity()?;
            let window = env.new_object(
                JNIString::new(Self::WINDOW_CLASS),
                jni_sig!("(Landroid/content/Context;)V"),
                &[act.as_obj().into()],
            )?;
            env.call_method(
                act,
                jni::jni_str!("setContentView"),
                jni_sig!("(Landroid/view/View;)V"),
                &[(&window).into()],
            )?
            .v()?;
            Ok(env.new_global_ref(window)?)
        })?;
        Ok(Self {
            inner: inner.into(),
        })
    }

    pub fn client_size(&self) -> Result<Size> {
        self.size()
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&self, visible: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&self, size: Size) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&self, text: impl AsRef<str>) -> Result<()>;

    pub async fn wait_close(&self) {
        std::future::pending().await
    }

    pub async fn wait_move(&self) {
        todo!()
    }

    pub async fn wait_size(&self) {
        todo!()
    }
}

impl AsWindow for Window {
    fn as_window(&self) -> BorrowedWindow<'_> {
        unsafe { BorrowedWindow::android(&self.inner) }
    }
}

impl Window {
    pub fn as_container(&self) -> BorrowedContainer<'_> {
        unsafe { BorrowedContainer::android(&self.inner) }
    }
}
