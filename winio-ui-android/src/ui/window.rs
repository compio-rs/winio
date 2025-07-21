//! Android window widget, based on JNI and native FrameLayout

use {
    super::super::RUNTIME,
    jni::objects::JString,
    winio_handle::{AsRawWindow, AsWindow, BorrowedWindow, RawWindow},
    winio_primitive::{Point, Size},
};

#[derive(Debug)]
pub struct Window {
    inner: RawWindow,
}

//noinspection SpellCheckingInspection
impl Window {
    pub fn new<W>(parent: Option<W>) -> Self
    where
        W: AsWindow,
    {
        let inner = RUNTIME.with(|rt| {
            rt.vm_exec(|mut env, act| {
                let class = env.find_class("rs/compio/winio/Window")?;
                let obj = if let Some(ref parent) = parent {
                    env.new_object(
                        class,
                        "(Landroid/content/Context;Lrs/compio/winio/Window;)V",
                        &[(&act).into(), parent.as_window().as_obj().into()],
                    )
                } else {
                    env.new_object(class, "(Landroid/content/Context;)V", &[(&act).into()])
                }?;
                let global = env.new_global_ref(&obj)?;

                Ok(global)
            })
            .unwrap()
        });

        Self { inner }
    }

    pub async fn wait_close(&self) {
        std::future::pending().await
    }

    pub async fn wait_move(&self) {
        std::future::pending().await
    }

    pub async fn wait_size(&self) {
        std::future::pending().await
    }

    pub fn text(&self) -> String {
        // 获取 TextView 的文本
        RUNTIME.with(|rt| {
            let w = self.inner.clone();
            rt.vm_exec(|mut env, _act| {
                let jstr =
                    env.call_method(w.as_obj(), "getText", "()Ljava/lang/CharSequence;", &[])?;
                let obj = jstr.l()?;
                let rust_str: String = env.get_string(&JString::from(obj))?.into();
                Ok(rust_str)
            })
            .unwrap()
        })
    }

    pub fn set_text<S>(&mut self, title: S)
    where
        S: AsRef<str>,
    {
        RUNTIME.with(|rt| {
            let w = self.inner.clone();
            rt.vm_exec(|mut env, _act| {
                let text = env.new_string(title.as_ref())?;
                env.call_method(
                    w.as_obj(),
                    "setText",
                    "(Ljava/lang/CharSequence;)V",
                    &[(&text).into()],
                )?;

                Ok(())
            })
            .unwrap();
        });
    }

    pub fn client_size(&self) -> Size {
        // 获取 FrameLayout 的宽高
        RUNTIME.with(|rt| {
            let frame_layout = self.inner.clone();
            rt.vm_exec(|mut env, _act| {
                let width = env
                    .call_method(frame_layout.as_obj(), "getWidth", "()I", &[])?
                    .i()?;
                let height = env
                    .call_method(frame_layout.as_obj(), "getHeight", "()I", &[])?
                    .i()?;
                Ok(Size::new(width as _, height as _))
            })
            .unwrap()
        })
    }

    pub fn is_visible(&self) -> bool {
        // 查询 FrameLayout 的可见性
        RUNTIME.with(|rt| {
            let frame_layout = self.inner.clone();
            rt.vm_exec(|mut env, _act| {
                let vis = env
                    .call_method(frame_layout.as_obj(), "getVisibility", "()I", &[])?
                    .i()?;
                Ok(vis == 0) // View.VISIBLE = 0
            })
            .unwrap()
        })
    }

    pub fn set_visible(&mut self, visible: bool) {
        RUNTIME.with(|rt| {
            let w = self.inner.clone();
            rt.vm_exec(|mut env, _act| {
                env.call_method(w.as_obj(), "setVisible", "(Z)V", &[visible.into()])?;
                Ok(())
            })
            .unwrap();
        });
    }

    pub fn loc(&self) -> Point {
        todo!()
    }

    pub fn set_loc(&mut self, Point { x, y, .. }: Point) {
        RUNTIME.with(|rt| {
            let w = self.inner.clone();
            rt.vm_exec(|mut env, _act| {
                env.call_method(w.as_obj(), "setLoc", "(FF)V", &[x.into(), y.into()])?;
                Ok(())
            })
            .unwrap();
        });
    }

    pub fn size(&self) -> Size {
        todo!()
    }

    pub fn set_size(&mut self, Size { width, height, .. }: Size) {
        RUNTIME.with(|rt| {
            let w = self.inner.clone();
            rt.vm_exec(|mut env, _act| {
                env.call_method(
                    w.as_obj(),
                    "setSize",
                    "(FF)V",
                    &[width.into(), height.into()],
                )?;
                Ok(())
            })
            .unwrap();
        });
    }
}

impl AsRawWindow for Window {
    fn as_raw_window(&self) -> RawWindow {
        // Return pointer or handle to FrameLayout
        self.inner.clone()
    }
}

impl AsWindow for Window {
    fn as_window(&self) -> BorrowedWindow<'_> {
        unsafe { BorrowedWindow::borrow_raw(self.inner.clone()) }
    }
}
