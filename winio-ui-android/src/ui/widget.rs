use std::ops::Deref;

use jni::{Env, jni_sig, objects::JObject, strings::JNIString};
use winio_handle::{AsContainer, AsWidget, BorrowedContainer, BorrowedWidget};
use winio_primitive::{HAlign, Point, Size};

use crate::{GlobalRef, JObjectExt, Result, current_activity, vm_exec};

#[derive(Debug)]
pub(crate) struct BaseWidget {
    inner: GlobalRef,
}

pub(crate) mod gravity {
    pub const CENTER_HORIZONTAL: i32 = 0x1;
    pub const CENTER_VERTICAL: i32 = 0x10;
    pub const FILL_HORIZONTAL: i32 = 0x7;
    pub const LEFT: i32 = 0x3;
    pub const RIGHT: i32 = 0x5;
}

// noinspection SpellCheckingInspection
impl BaseWidget {
    pub(crate) fn new(parent: BorrowedContainer, widget_class: &str) -> Result<Self> {
        vm_exec(|env| Self::new_with_env(env, parent, widget_class))
    }

    pub(crate) fn new_with_env(
        env: &mut Env,
        parent: BorrowedContainer,
        widget_class: &str,
    ) -> Result<Self> {
        let parent = env.new_global_ref(parent.as_container().to_android())?;
        let context = current_activity()?;
        let widget = env.new_object(
            JNIString::new(widget_class),
            jni_sig!("(Landroid/content/Context;)V"),
            &[context.as_obj().into()],
        )?;
        env.call_method(
            parent.as_obj(),
            jni::jni_str!("addView"),
            jni::jni_sig!("(Landroid/view/View;)V"),
            &[(&widget).into()],
        )?
        .v()?;
        let inner = env.new_global_ref(widget)?;
        Ok(Self { inner })
    }

    pub fn as_obj(&self) -> &JObject<'static> {
        self.inner.as_obj()
    }

    pub fn hash_code(&self) -> Result<i32> {
        vm_exec(|env| {
            Ok(env
                .call_method(
                    self.inner.as_obj(),
                    jni::jni_str!("hashCode"),
                    jni::jni_sig!("()I"),
                    &[],
                )?
                .i()?)
        })
    }

    pub fn loc(&self) -> Result<Point> {
        vm_exec(move |env| {
            let x = env
                .call_method(
                    self.as_obj(),
                    jni::jni_str!("getX"),
                    jni::jni_sig!("()D"),
                    &[],
                )?
                .d()?;
            let y = env
                .call_method(
                    self.as_obj(),
                    jni::jni_str!("getY"),
                    jni::jni_sig!("()D"),
                    &[],
                )?
                .d()?;
            Ok(Point::new(x, y))
        })
    }

    pub fn set_loc(&mut self, p: Point) -> Result<()> {
        vm_exec(move |env| {
            env.call_method(
                self.inner.as_obj(),
                jni::jni_str!("setX"),
                jni::jni_sig!("(D)V"),
                &[p.x.into()],
            )?;
            env.call_method(
                self.inner.as_obj(),
                jni::jni_str!("setY"),
                jni::jni_sig!("(D)V"),
                &[p.y.into()],
            )?;
            Ok(())
        })
    }

    pub fn size(&self) -> Result<Size> {
        vm_exec(|env| {
            let width = env
                .call_method(
                    self.inner.as_obj(),
                    jni::jni_str!("getWidth"),
                    jni::jni_sig!("()I"),
                    &[],
                )?
                .i()?;
            let height = env
                .call_method(
                    self.inner.as_obj(),
                    jni::jni_str!("getHeight"),
                    jni::jni_sig!("()I"),
                    &[],
                )?
                .i()?;
            Ok(Size::new(width as _, height as _))
        })
    }

    pub fn set_size(&mut self, size: Size) -> Result<()> {
        vm_exec(move |env| {
            let params = env.new_object(
                jni::jni_str!("android/view/ViewGroup$LayoutParams"),
                jni::jni_sig!("(II)V"),
                &[(size.width as i32).into(), (size.height as i32).into()],
            )?;
            env.call_method(
                self.inner.as_obj(),
                jni::jni_str!("setLayoutParams"),
                jni::jni_sig!("(Landroid/view/ViewGroup$LayoutParams;)V"),
                &[(&params).into()],
            )?;
            Ok(())
        })
    }

    pub fn preferred_size(&self) -> Result<Size> {
        vm_exec(move |env| {
            env.call_method(
                self.inner.as_obj(),
                jni::jni_str!("measure"),
                jni::jni_sig!("(II)V"),
                &[0i32.into(), 0i32.into()],
            )?;
            let width = env
                .call_method(
                    self.inner.as_obj(),
                    jni::jni_str!("getMeasuredWidth"),
                    jni::jni_sig!("()I"),
                    &[],
                )?
                .i()?;
            let height = env
                .call_method(
                    self.inner.as_obj(),
                    jni::jni_str!("getMeasuredHeight"),
                    jni::jni_sig!("()I"),
                    &[],
                )?
                .i()?;
            Ok(Size::new(width as _, height as _))
        })
    }

    pub fn min_size(&self) -> Result<Size> {
        vm_exec(move |env| {
            let width = env
                .call_method(
                    self.inner.as_obj(),
                    jni::jni_str!("getMinimumWidth"),
                    jni::jni_sig!("()I"),
                    &[],
                )?
                .i()?;
            let height = env
                .call_method(
                    self.inner.as_obj(),
                    jni::jni_str!("getMinimumHeight"),
                    jni::jni_sig!("()I"),
                    &[],
                )?
                .i()?;
            Ok(Size::new(width as _, height as _))
        })
    }

    pub fn is_visible(&self) -> Result<bool> {
        vm_exec(move |env| {
            let vis = env
                .call_method(
                    self.inner.as_obj(),
                    jni::jni_str!("getVisibility"),
                    jni::jni_sig!("()I"),
                    &[],
                )?
                .i()?;
            Ok(vis == 0)
        })
    }

    pub fn set_visible(&mut self, visible: bool) -> Result<()> {
        vm_exec(move |env| {
            env.call_method(
                self.inner.as_obj(),
                jni::jni_str!("setVisibility"),
                jni::jni_sig!("(I)V"),
                &[if visible { 0 } else { 4 }.into()],
            )?
            .v()?;
            Ok(())
        })
    }

    pub fn tooltip(&self) -> Result<String> {
        Ok(String::new())
    }

    pub fn set_tooltip(&mut self, _s: impl AsRef<str>) -> Result<()> {
        Ok(())
    }

    pub fn text(&self) -> Result<String> {
        vm_exec(move |env| {
            env.call_method(
                self.inner.as_obj(),
                jni::jni_str!("getText"),
                jni::jni_sig!("()Ljava/lang/CharSequence;"),
                &[],
            )?
            .l()?
            .to(env)
        })
    }

    pub fn set_text(&mut self, text: impl AsRef<str>) -> Result<()> {
        vm_exec(move |env| {
            let text = env.new_string(&text)?;
            env.call_method(
                self.inner.as_obj(),
                jni::jni_str!("setText"),
                jni::jni_sig!("(Ljava/lang/CharSequence;)V"),
                &[(&text).into()],
            )?
            .v()?;
            Ok(())
        })
    }

    pub fn is_enabled(&self) -> Result<bool> {
        vm_exec(move |env| {
            Ok(env
                .call_method(
                    self.inner.as_obj(),
                    jni::jni_str!("isEnabled"),
                    jni::jni_sig!("()Z"),
                    &[],
                )?
                .z()?)
        })
    }

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()> {
        vm_exec(move |env| {
            env.call_method(
                self.inner.as_obj(),
                jni::jni_str!("setEnabled"),
                jni::jni_sig!("(Z)V"),
                &[enabled.into()],
            )?
            .v()?;
            Ok(())
        })
    }

    pub fn halign(&self) -> Result<HAlign> {
        let gravity = vm_exec(|env| {
            Ok(env
                .call_method(
                    self.inner.as_obj(),
                    jni::jni_str!("getGravity"),
                    jni::jni_sig!("()I"),
                    &[],
                )?
                .i()?)
        })?;
        if gravity & gravity::CENTER_HORIZONTAL != 0 {
            Ok(HAlign::Center)
        } else if gravity & gravity::FILL_HORIZONTAL == gravity::FILL_HORIZONTAL {
            Ok(HAlign::Stretch)
        } else if gravity & gravity::RIGHT != 0 {
            Ok(HAlign::Right)
        } else {
            Ok(HAlign::Left)
        }
    }

    pub fn set_halign(&mut self, align: HAlign) -> Result<()> {
        let gravity = match align {
            HAlign::Left => gravity::LEFT,
            HAlign::Center => gravity::CENTER_HORIZONTAL,
            HAlign::Right => gravity::RIGHT,
            HAlign::Stretch => gravity::FILL_HORIZONTAL,
        } | gravity::CENTER_VERTICAL;
        vm_exec(|env| {
            env.call_method(
                self.inner.as_obj(),
                jni::jni_str!("setGravity"),
                jni::jni_sig!("(I)V"),
                &[jni::JValue::Int(gravity)],
            )?
            .v()?;
            Ok(())
        })
    }
}

impl From<GlobalRef> for BaseWidget {
    fn from(value: GlobalRef) -> Self {
        Self { inner: value }
    }
}

impl Deref for BaseWidget {
    type Target = GlobalRef;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AsWidget for BaseWidget {
    fn as_widget(&self) -> BorrowedWidget<'_> {
        unsafe { BorrowedWidget::android(&self.inner) }
    }
}
