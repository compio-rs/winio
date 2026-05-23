//! Android window widget, based on JNI and FrameLayout

use inherit_methods_macro::inherit_methods;
use winio_handle::{AsWindow, BorrowedContainer, BorrowedWindow};
use winio_primitive::{Point, Size};

use super::{
    super::{JObjectExt, define_event, recv_event},
    BaseWidget, vm_exec_on_ui_thread,
};

define_event!(
    WAIT_FOR_WINDOW_CLOSING,
    Java_rs_compio_winio_Window_on_1closed
);
define_event!(
    WAIT_FOR_WINDOW_MOVING,
    Java_rs_compio_winio_Window_on_1moved
);
define_event!(
    WAIT_FOR_WINDOW_SIZING,
    Java_rs_compio_winio_Window_on_1sized
);

#[derive(Debug)]
pub struct Window {
    inner: BaseWidget,
}

// noinspection SpellCheckingInspection
#[inherit_methods(from = "self.inner")]
impl Window {
    const WINDOW_CLASS: &'static str = "rs/compio/winio/Window";

    pub fn new<W>(parent: Option<W>) -> Self
    where
        W: AsWindow,
    {
        let parent = parent.map(|w| w.as_window().to_android().clone());
        let inner = vm_exec_on_ui_thread(move |mut env, act| {
            let window = if let Some(parent) = parent.as_ref() {
                env.new_object(
                    jni::strings::JNIString::from(Self::WINDOW_CLASS),
                    jni::strings::JNIString::from(format!(
                        "(Landroid/content/Context;L{};)V",
                        Self::WINDOW_CLASS
                    )),
                    &[act.as_obj().into(), parent.as_obj().into()],
                )
            } else {
                env.new_object(
                    jni::strings::JNIString::from(Self::WINDOW_CLASS),
                    jni::strings::JNIString::from("(Landroid/content/Context;)V"),
                    &[act.as_obj().into()],
                )
            }?;
            env.new_global_ref(window)
        })
        .unwrap()
        .into();
        Self { inner }
    }

    pub async fn wait_close(&self) {
        recv_event!(self, WAIT_FOR_WINDOW_CLOSING)
    }

    pub async fn wait_move(&self) {
        recv_event!(self, WAIT_FOR_WINDOW_MOVING)
    }

    pub async fn wait_size(&self) {
        recv_event!(self, WAIT_FOR_WINDOW_SIZING)
    }

    pub fn client_size(&self) -> Size {
        let w = self.inner.duplicate();
        vm_exec_on_ui_thread(move |mut env, _| {
            env.call_method(
                w.as_obj(),
                jni::jni_str!("getClientSize"),
                jni::jni_sig!("()[D"),
                &[],
            )?
            .l()?
            .to(&mut env)
        })
        .unwrap()
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&self, visible: bool);

    pub fn loc(&self) -> Point;

    pub fn set_loc(&self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&self, size: Size);

    pub fn text(&self) -> String;

    pub fn set_text<S>(&self, _text: S)
    where
        S: AsRef<str>;
}

impl AsWindow for Window {
    fn as_window(&self) -> BorrowedWindow<'_> {
        unsafe { BorrowedWindow::android(&&self.inner) }
    }
}

impl Window {
    pub fn as_container(&self) -> BorrowedContainer<'_> {
        unsafe { BorrowedContainer::android(&&self.inner) }
    }
}
