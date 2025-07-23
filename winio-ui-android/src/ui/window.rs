//! Android window widget, based on JNI and FrameLayout

use {
    super::{super::JObjectExt, BaseWidget, vm_exec_on_ui_thread},
    inherit_methods_macro::inherit_methods,
    jni::{JNIEnv, objects::JObject},
    oneshot::{Sender, channel},
    std::{
        collections::HashMap,
        sync::{LazyLock, Mutex},
    },
    winio_handle::{AsRawWindow, AsWindow, BorrowedWindow, RawWindow},
    winio_primitive::{Point, Size},
};

static WAIT_FOR_WINDOW_CLOSING: LazyLock<Mutex<HashMap<i32, Vec<Sender<()>>>>> =
    LazyLock::new(Default::default);

#[derive(Debug)]
pub struct Window {
    inner: BaseWidget,
}

//noinspection SpellCheckingInspection
#[inherit_methods(from = "self.inner")]
impl Window {
    const WINDOW_CLASS: &'static str = "rs/compio/winio/Window";

    pub fn new<W>(parent: Option<W>) -> Self
    where
        W: AsWindow,
    {
        let parent = parent.map(|w| w.as_window().as_raw_window().clone());
        let inner = vm_exec_on_ui_thread(move |mut env, act| {
            let window = if let Some(parent) = parent.as_ref() {
                env.new_object(
                    Self::WINDOW_CLASS,
                    format!("(Landroid/content/Context;L{};)V", Self::WINDOW_CLASS).as_str(),
                    &[act.as_obj().into(), parent.as_obj().into()],
                )
            } else {
                env.new_object(
                    Self::WINDOW_CLASS,
                    "(Landroid/content/Context;)V",
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
        if let Ok(mut lock) = WAIT_FOR_WINDOW_CLOSING.lock() {
            let (tx, rx) = channel();
            let hash_code = self.inner.hash_code();
            if let Some(senders) = lock.get_mut(&hash_code) {
                senders.push(tx);
            } else {
                lock.insert(hash_code, vec![tx]);
            }
            drop(lock);
            let _ = rx.await;
        }
    }

    pub async fn wait_move(&self) {
        std::future::pending().await
    }

    pub async fn wait_size(&self) {
        std::future::pending().await
    }

    pub fn client_size(&self) -> Size {
        let w = self.inner.clone();
        vm_exec_on_ui_thread(move |mut env, _| {
            env.call_method(w.as_obj(), "getClientSize", "()[D", &[])?
                .l()?
                .to(&mut env)
        })
        .unwrap()
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, visible: bool);

    pub fn loc(&self) -> Point {
        todo!()
    }

    pub fn set_loc(&mut self, _p: Point) {
        todo!()
    }

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, size: Size);

    pub fn text(&self) -> String;

    pub fn set_text<S>(&mut self, _text: S)
    where
        S: AsRef<str>;
}

impl AsRawWindow for Window {
    fn as_raw_window(&self) -> RawWindow {
        // Return pointer or handle to FrameLayout
        (&*self.inner).clone()
    }
}

impl AsWindow for Window {
    fn as_window(&self) -> BorrowedWindow<'_> {
        unsafe { BorrowedWindow::borrow_raw(self.as_raw_window()) }
    }
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
extern "C" fn Java_rs_compio_winio_Window_on_1closed(env: JNIEnv, window: JObject) {
    let Ok(w) = env.new_global_ref(window) else {
        return;
    };
    let w: BaseWidget = w.into();

    if let Ok(mut lock) = WAIT_FOR_WINDOW_CLOSING.lock()
        && let Some(mut senders) = lock.remove(&w.hash_code())
    {
        drop(lock);
        while let Some(sender) = senders.pop() {
            let _ = sender.send(());
        }
    }
}
