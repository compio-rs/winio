use {
    super::super::RUNTIME,
    jni::objects::GlobalRef,
    winio_handle::AsWindow,
    winio_primitive::{HAlign, Point, Size},
};

#[derive(Debug)]
pub struct Label {
    tv: GlobalRef,
}

impl Label {
    pub fn text(&self) -> String {
        todo!()
    }

    pub fn set_text<S>(&mut self, text: S)
    where
        S: AsRef<str>,
    {
        RUNTIME.with(|rt| {
            let view = self.tv.clone();
            rt.vm_exec(|mut env, _act| {
                let text = env.new_string(text.as_ref())?;
                env.call_method(
                    view.as_obj(),
                    "setText",
                    "(Ljava/lang/CharSequence;)V",
                    &[(&text).into()],
                )?;

                Ok(())
            })
            .unwrap();
        });
    }

    pub fn halign(&self) -> HAlign {
        todo!()
    }

    pub fn set_halign(&mut self, align: HAlign) {
        // 通过JNI设置TextView的文本对齐方式
        RUNTIME.with(|rt| {
            let view = self.tv.clone();
            rt.vm_exec(|mut env, _act| {
                let gravity = match align {
                    HAlign::Left => 3,                     // Gravity.LEFT
                    HAlign::Center | HAlign::Stretch => 1, // Gravity.CENTER_HORIZONTAL
                    HAlign::Right => 5,                    // Gravity.RIGHT
                };
                env.call_method(view.as_obj(), "setGravity", "(I)V", &[gravity.into()])?;
                Ok(())
            })
            .unwrap();
        });
    }

    pub fn is_visible(&self) -> bool {
        todo!()
    }

    pub fn set_visible(&mut self, _v: bool) {
        todo!()
    }

    pub fn is_enabled(&self) -> bool {
        todo!()
    }

    pub fn set_enabled(&mut self, _v: bool) {
        todo!()
    }

    pub fn loc(&self) -> Point {
        todo!()
    }

    pub fn set_loc(&mut self, _p: Point) {
        todo!()
    }

    pub fn size(&self) -> Size {
        todo!()
    }

    pub fn set_size(&mut self, size: Size) {
        let (width, height) = (size.width as i32, size.height as i32);
        // 通过JNI设置TextView的宽高
        RUNTIME.with(|rt| {
            let view = self.tv.clone();
            rt.vm_exec(|mut env, _act| {
                let lp_class = env.find_class("android/widget/FrameLayout$LayoutParams")?;
                let lp_obj =
                    env.new_object(lp_class, "(II)V", &[width.into(), height.into()])?;
                env.call_method(
                    view.as_obj(),
                    "setLayoutParams",
                    "(Landroid/view/ViewGroup$LayoutParams;)V",
                    &[(&lp_obj).into()],
                )?;

                Ok(())
            })
            .unwrap();
        });
    }

    pub fn preferred_size(&self) -> Size {
        todo!()
    }

    pub fn new<W>(parent: W) -> Self
    where
        W: AsWindow,
    {
        let parent = parent.as_window();
        let tv = RUNTIME.with(|rt| {
            rt.vm_exec(|mut env, act| {
                let tv_class = env.find_class("android/widget/TextView")?;
                let tv_obj =
                    env.new_object(tv_class, "(Landroid/content/Context;)V", &[(&act).into()])?;
                env.call_method(
                    &*parent,
                    "addView",
                    "(Landroid/view/View;)V",
                    &[(&tv_obj).into()],
                )?;
                env.new_global_ref(&tv_obj)
            })
            .unwrap()
        });

        Self { tv }
    }
}
