//! Android window widget, based on JNI and native FrameLayout

use {
    super::super::RUNTIME,
    jni::objects::{GlobalRef, JString},
    winio_handle::{AsRawWindow, AsWindow, BorrowedWindow, RawWindow},
    winio_primitive::{Point, Size},
};

/// Represents a window rendered by an Android FrameLayout.
#[derive(Debug)]
pub struct Window {
    /// Global reference to the FrameLayout Java object
    frame_layout: GlobalRef,
    /// Global reference to the TextView Java object
    title_view: GlobalRef,
    /// Location of the window
    loc: Point,
    /// Size of the window
    size: Size,
}

impl Window {
    pub fn new<W>(_parent: Option<W>) -> Self
    where
        W: AsWindow,
    {
        // Create FrameLayout via JNI
        let (frame_layout, title_view) = RUNTIME.with(|rt| {
            rt.vm_exec(|mut env, act| {
                // 创建 FrameLayout
                let frame_class = env.find_class("android/widget/FrameLayout")?;
                let frame_obj = env.new_object(
                    frame_class,
                    "(Landroid/content/Context;)V",
                    &[(&act).into()],
                )?;
                let frame_global = env.new_global_ref(&frame_obj)?;
                // 创建 TextView
                let text_class = env.find_class("android/widget/TextView")?;
                let text_obj =
                    env.new_object(text_class, "(Landroid/content/Context;)V", &[(&act).into()])?;
                let text_global = env.new_global_ref(&text_obj)?;
                // 设置 TextView 布局参数（宽度MATCH_PARENT，高度WRAP_CONTENT）
                let lp_class = env.find_class("android/widget/FrameLayout$LayoutParams")?;
                let match_parent = env.get_static_field(&lp_class, "MATCH_PARENT", "I")?.i()?;
                let wrap_content = env.get_static_field(&lp_class, "WRAP_CONTENT", "I")?.i()?;
                let lp_obj = env.new_object(
                    lp_class,
                    "(II)V",
                    &[match_parent.into(), wrap_content.into()],
                )?;
                env.call_method(
                    &text_obj,
                    "setLayoutParams",
                    "(Landroid/view/ViewGroup$LayoutParams;)V",
                    &[(&lp_obj).into()],
                )?;
                // 添加 TextView 到 FrameLayout
                env.call_method(
                    frame_obj,
                    "addView",
                    "(Landroid/view/View;)V",
                    &[(&text_obj).into()],
                )?;
                Ok((frame_global, text_global))
            })
            .unwrap()
        });

        Self {
            frame_layout,
            title_view,
            loc: Default::default(),
            size: Default::default(),
        }
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
            let title_view = self.title_view.clone();
            rt.vm_exec(|mut env, _act| {
                let jstr = env.call_method(
                    title_view.as_obj(),
                    "getText",
                    "()Ljava/lang/CharSequence;",
                    &[],
                )?;
                let obj = jstr.l()?;
                let str_obj = env.call_method(obj, "toString", "()Ljava/lang/String;", &[])?;
                let jstr2 = str_obj.l()?;
                let rust_str: String = env.get_string(&JString::from(jstr2))?.into();
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
            let title_view = self.title_view.clone();
            rt.vm_exec(|mut env, _act| {
                let text = env.new_string(title.as_ref())?;
                env.call_method(
                    title_view.as_obj(),
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
            let frame_layout = self.frame_layout.clone();
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
            let frame_layout = self.frame_layout.clone();
            rt.vm_exec(|mut env, _act| {
                let vis = env
                    .call_method(frame_layout.as_obj(), "getVisibility", "()I", &[])?
                    .i()?;
                Ok(vis == 0) // View.VISIBLE = 0
            })
            .unwrap()
        })
    }

    pub fn set_visible(&mut self, v: bool) {
        // 设置 FrameLayout 的可见性
        RUNTIME.with(|rt| {
            let frame_layout = self.frame_layout.clone();
            rt.vm_exec(|mut env, _act| {
                let vis = if v { 0 } else { 4 }; // View.VISIBLE = 0, View.INVISIBLE = 4
                env.call_method(
                    frame_layout.as_obj(),
                    "setVisibility",
                    "(I)V",
                    &[vis.into()],
                )?;
                Ok(())
            })
            .unwrap();
        });
    }

    pub fn loc(&self) -> Point {
        self.loc
    }

    pub fn set_loc(&mut self, p: Point) {
        self.loc = p;
        // Optionally update Java side position
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn set_size(&mut self, v: Size) {
        self.size = v;
        // Optionally update Java side size
    }
}

impl AsRawWindow for Window {
    fn as_raw_window(&self) -> RawWindow {
        // Return pointer or handle to FrameLayout
        self.frame_layout.clone()
    }
}

impl AsWindow for Window {
    fn as_window(&self) -> BorrowedWindow<'_> {
        unsafe { BorrowedWindow::borrow_raw(self.frame_layout.clone()) }
    }
}
