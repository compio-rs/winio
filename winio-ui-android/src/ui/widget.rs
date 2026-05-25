use std::ops::Deref;

use jni::{jni_sig, jni_str, objects::JObject, refs::Global, strings::JNIString};
use winio_handle::{AsWindow, BorrowedWidget, BorrowedWindow};
use winio_primitive::{HAlign, Point, Size};

use super::{super::JObjectExt, vm_exec, vm_exec_on_ui_thread};
use crate::GlobalRef;

#[derive(Debug)]
pub struct BaseWidget {
    inner: Global<JObject<'static>>,
}

// noinspection SpellCheckingInspection
impl BaseWidget {
    const HALIGN_CENTER: i32 = 1;
    const HALIGN_LEFT: i32 = 0;
    const HALIGN_RIGHT: i32 = 2;
    const HALIGN_STRETCH: i32 = 3;

    pub(crate) fn create<S, T>(parent: BorrowedWindow, widget_class: S) -> T
    where
        S: AsRef<str>,
        T: From<Self>,
    {
        let parent =
            vm_exec(|env, _| env.new_global_ref(parent.as_window().to_android().as_obj())).unwrap();
        let widget_class = widget_class.as_ref().to_owned();
        let inner = vm_exec_on_ui_thread(move |env, _| {
            let widget = env.new_object(
                JNIString::new(widget_class.as_str()),
                jni_sig!("(Lrs/compio/winio/Window;)V"),
                &[parent.as_obj().into()],
            )?;
            env.new_global_ref(widget)
        })
        .unwrap()
        .into();

        T::from(inner)
    }

    pub(crate) fn as_obj(&self) -> &JObject<'static> {
        self.inner.as_obj()
    }

    pub(crate) fn duplicate(&self) -> GlobalRef {
        vm_exec(|env, _| env.new_global_ref(self.inner.as_obj())).unwrap()
    }

    pub(crate) fn hash_code(&self) -> i32 {
        vm_exec(|env, _| {
            env.call_method(
                self.inner.as_obj(),
                jni::jni_str!("hashCode"),
                jni::jni_sig!("()I"),
                &[],
            )?
            .i()
        })
        .unwrap()
    }

    pub(crate) fn loc(&self) -> Point {
        let w = self.duplicate();
        vm_exec_on_ui_thread(move |env, _| {
            env.call_method(
                w.as_obj(),
                jni::jni_str!("getLoc"),
                jni::jni_sig!("()[D"),
                &[],
            )?
            .l()?
            .to(env)
        })
        .unwrap()
    }

    pub(crate) fn set_loc(&self, p: Point) {
        let w = self.duplicate();
        vm_exec_on_ui_thread(move |env, _| {
            env.call_method(
                w.as_obj(),
                jni::jni_str!("setLoc"),
                jni::jni_sig!("(DD)V"),
                &[p.x.into(), p.y.into()],
            )?
            .v()
        })
        .unwrap();
    }

    pub(crate) fn size(&self) -> Size {
        let w = self.duplicate();
        vm_exec_on_ui_thread(move |env, _| {
            env.call_method(
                w.as_obj(),
                jni::jni_str!("getSize"),
                jni::jni_sig!("()[D"),
                &[],
            )?
            .l()?
            .to(env)
        })
        .unwrap()
    }

    pub(crate) fn set_size(&self, size: Size) {
        let w = self.duplicate();
        vm_exec_on_ui_thread(move |env, _| {
            env.call_method(
                w.as_obj(),
                jni::jni_str!("setSize"),
                jni::jni_sig!("(DD)V"),
                &[size.width.into(), size.height.into()],
            )?
            .v()
        })
        .unwrap();
    }

    pub(crate) fn preferred_size(&self) -> Size {
        let w = self.duplicate();
        vm_exec_on_ui_thread(move |env, _| {
            env.call_method(
                w.as_obj(),
                jni::jni_str!("getPreferredSize"),
                jni::jni_sig!("()[D"),
                &[],
            )?
            .l()?
            .to(env)
        })
        .unwrap()
    }

    pub(crate) fn is_visible(&self) -> bool {
        let w = self.duplicate();
        vm_exec_on_ui_thread(move |env, _| {
            env.call_method(
                w.as_obj(),
                jni::jni_str!("isVisible"),
                jni::jni_sig!("()Z"),
                &[],
            )?
            .z()
        })
        .unwrap()
    }

    pub(crate) fn set_visible(&self, visible: bool) {
        let w = self.duplicate();
        vm_exec_on_ui_thread(move |env, _| {
            env.call_method(
                w.as_obj(),
                jni::jni_str!("setVisible"),
                jni::jni_sig!("(Z)V"),
                &[visible.into()],
            )?
            .v()
        })
        .unwrap();
    }

    pub(crate) fn text(&self) -> String {
        let w = self.duplicate();
        vm_exec_on_ui_thread(move |env, _| {
            env.call_method(
                w.as_obj(),
                jni::jni_str!("getText"),
                jni::jni_sig!("()Ljava/lang/CharSequence;"),
                &[],
            )?
            .l()?
            .to(env)
        })
        .unwrap()
    }

    pub(crate) fn set_text<S>(&self, text: S)
    where
        S: AsRef<str>,
    {
        let text = text.as_ref().to_owned();
        let w = self.duplicate();
        vm_exec_on_ui_thread(move |env, _| {
            let text = env.new_string(&text)?;
            env.call_method(
                w.as_obj(),
                jni::jni_str!("setText"),
                jni::jni_sig!("(Ljava/lang/CharSequence;)V"),
                &[(&text).into()],
            )?
            .v()
        })
        .unwrap();
    }

    pub fn halign(&self) -> HAlign {
        let w = self.duplicate();
        let align = vm_exec_on_ui_thread(move |env, _| {
            env.call_method(
                w.as_obj(),
                jni::jni_str!("getHAlign"),
                jni::jni_sig!("()I"),
                &[],
            )?
            .i()
        })
        .unwrap();

        match align {
            Self::HALIGN_LEFT => HAlign::Left,
            Self::HALIGN_CENTER => HAlign::Center,
            Self::HALIGN_RIGHT => HAlign::Right,
            Self::HALIGN_STRETCH => HAlign::Stretch,
            _ => HAlign::Left,
        }
    }

    pub fn set_halign(&self, align: HAlign) {
        let value = match align {
            HAlign::Left => Self::HALIGN_LEFT,
            HAlign::Center => Self::HALIGN_CENTER,
            HAlign::Right => Self::HALIGN_RIGHT,
            HAlign::Stretch => Self::HALIGN_STRETCH,
        };
        let w = self.duplicate();
        vm_exec_on_ui_thread(move |env, _| {
            env.call_method(
                w.as_obj(),
                jni::jni_str!("setHAlign"),
                jni::jni_sig!("(I)V"),
                &[value.into()],
            )?
            .v()
        })
        .unwrap();
    }

    pub(crate) fn is_checked(&self) -> bool {
        let w = self.duplicate();
        vm_exec_on_ui_thread(move |env, _| {
            env.call_method(
                w.as_obj(),
                jni::jni_str!("isChecked"),
                jni::jni_sig!("()Z"),
                &[],
            )?
            .z()
        })
        .unwrap()
    }

    pub(crate) fn set_checked(&self, checked: bool) {
        let w = self.duplicate();
        vm_exec_on_ui_thread(move |env, _| {
            env.call_method(
                w.as_obj(),
                jni::jni_str!("setChecked"),
                jni::jni_sig!("(Z)V"),
                &[checked.into()],
            )?
            .v()
        })
        .unwrap();
    }

    pub(crate) fn is_enabled(&self) -> bool {
        let w = self.duplicate();
        vm_exec_on_ui_thread(move |env, _| {
            env.call_method(
                w.as_obj(),
                jni::jni_str!("isEnabled"),
                jni::jni_sig!("()Z"),
                &[],
            )?
            .z()
        })
        .unwrap()
    }

    pub(crate) fn set_enabled(&self, enabled: bool) {
        let w = self.duplicate();
        vm_exec_on_ui_thread(move |env, _| {
            env.call_method(
                w.as_obj(),
                jni::jni_str!("setEnabled"),
                jni::jni_sig!("(Z)V"),
                &[enabled.into()],
            )?
            .v()
        })
        .unwrap();
    }

    pub(crate) fn range(&self) -> (usize, usize) {
        let w = self.duplicate();
        vm_exec_on_ui_thread(move |env, _| {
            env.call_method(
                w.as_obj(),
                jni::jni_str!("getRange"),
                jni::jni_sig!("()[I"),
                &[],
            )?
            .l()?
            .to(env)
        })
        .unwrap()
    }

    pub(crate) fn set_range(&self, min: usize, max: usize) {
        let w = self.duplicate();
        vm_exec_on_ui_thread(move |env, _| {
            env.call_method(
                w.as_obj(),
                jni::jni_str!("setRange"),
                jni::jni_sig!("(II)V"),
                &[(min as i32).into(), (max as i32).into()],
            )?
            .v()
        })
        .unwrap();
    }

    pub(crate) fn pos(&self) -> usize {
        let w = self.duplicate();
        vm_exec_on_ui_thread(move |env, _| {
            env.call_method(
                w.as_obj(),
                jni::jni_str!("getPos"),
                jni::jni_sig!("()I"),
                &[],
            )?
            .i()
        })
        .unwrap() as _
    }

    pub(crate) fn set_pos(&self, pos: usize) {
        let w = self.duplicate();
        vm_exec_on_ui_thread(move |env, _| {
            env.call_method(
                w.as_obj(),
                jni::jni_str!("setPos"),
                jni::jni_sig!("(I)V"),
                &[(pos as i32).into()],
            )?
            .v()
        })
        .unwrap();
    }

    pub(crate) fn is_indeterminate(&self) -> bool {
        let w = self.duplicate();
        vm_exec_on_ui_thread(move |env, _| {
            env.call_method(
                w.as_obj(),
                jni::jni_str!("isIndeterminate"),
                jni::jni_sig!("()Z"),
                &[],
            )?
            .z()
        })
        .unwrap()
    }

    pub(crate) fn set_indeterminate(&self, indeterminate: bool) {
        let w = self.duplicate();
        vm_exec_on_ui_thread(move |env, _| {
            env.call_method(
                w.as_obj(),
                jni::jni_str!("setIndeterminate"),
                jni::jni_sig!("(Z)V"),
                &[indeterminate.into()],
            )?
            .v()
        })
        .unwrap();
    }

    pub(crate) fn minimum(&self) -> usize {
        let w = self.duplicate();
        vm_exec_on_ui_thread(move |env, _| {
            env.call_method(
                w.as_obj(),
                jni::jni_str!("getMinimum"),
                jni::jni_sig!("()I"),
                &[],
            )?
            .i()
        })
        .unwrap() as _
    }

    pub(crate) fn set_minimum(&self, minimum: usize) {
        let w = self.duplicate();
        vm_exec_on_ui_thread(move |env, _| {
            env.call_method(
                w.as_obj(),
                jni::jni_str!("setMinimum"),
                jni::jni_sig!("(I)V"),
                &[(minimum as i32).into()],
            )?
            .v()
        })
        .unwrap();
    }

    pub(crate) fn maximum(&self) -> usize {
        let w = self.duplicate();
        vm_exec_on_ui_thread(move |env, _| {
            env.call_method(
                w.as_obj(),
                jni::jni_str!("getMaximum"),
                jni::jni_sig!("()I"),
                &[],
            )?
            .i()
        })
        .unwrap() as _
    }

    pub(crate) fn set_maximum(&self, maximum: usize) {
        let w = self.duplicate();
        vm_exec_on_ui_thread(move |env, _| {
            env.call_method(
                w.as_obj(),
                jni::jni_str!("setMaximum"),
                jni::jni_sig!("(I)V"),
                &[(maximum as i32).into()],
            )?
            .v()
        })
        .unwrap();
    }

    pub(crate) fn clear(&self) {
        let w = self.duplicate();
        vm_exec_on_ui_thread(move |env, _| {
            env.call_method(
                w.as_obj(),
                jni::jni_str!("clear"),
                jni::jni_sig!("()V"),
                &[],
            )?
            .v()
        })
        .unwrap()
    }

    pub(crate) fn get(&self, i: usize) -> String {
        let w = self.duplicate();
        vm_exec_on_ui_thread(move |env, _| {
            env.call_method(
                w.as_obj(),
                jni::jni_str!("get"),
                jni::jni_sig!("(I)Ljava/lang/CharSequence;"),
                &[(i as i32).into()],
            )?
            .l()?
            .to(env)
        })
        .unwrap()
    }

    pub(crate) fn set<S>(&self, i: usize, item: S)
    where
        S: AsRef<str>,
    {
        let item = item.as_ref().to_owned();
        let w = self.duplicate();
        vm_exec_on_ui_thread(move |env, _| {
            let text = env.new_string(&item)?;
            env.call_method(
                w.as_obj(),
                jni::jni_str!("set"),
                jni::jni_sig!("(ILjava/lang/CharSequence;)V"),
                &[(i as i32).into(), (&text).into()],
            )?
            .v()
        })
        .unwrap();
    }

    pub(crate) fn insert<S>(&self, i: usize, item: S)
    where
        S: AsRef<str>,
    {
        let item = item.as_ref().to_owned();
        let w = self.duplicate();
        vm_exec_on_ui_thread(move |env, _| {
            let text = env.new_string(&item)?;
            env.call_method(
                w.as_obj(),
                jni::jni_str!("insert"),
                jni::jni_sig!("(ILjava/lang/CharSequence;)V"),
                &[(i as i32).into(), (&text).into()],
            )?
            .v()
        })
        .unwrap();
    }

    pub(crate) fn remove(&self, i: usize) {
        let w = self.duplicate();
        vm_exec_on_ui_thread(move |env, _| {
            env.call_method(
                w.as_obj(),
                jni::jni_str!("remove"),
                jni::jni_sig!("(I)V"),
                &[(i as i32).into()],
            )?
            .v()
        })
        .unwrap();
    }

    pub(crate) fn selection(&self) -> Option<usize> {
        let w = self.duplicate();
        vm_exec_on_ui_thread(move |env, _| {
            env.call_method(
                w.as_obj(),
                jni::jni_str!("getSelection"),
                jni::jni_sig!("()Ljava/lang/Integer;"),
                &[],
            )?
            .l()?
            .to(env)
        })
        .unwrap()
    }

    pub(crate) fn set_selection(&self, i: Option<usize>) {
        let w = self.duplicate();
        vm_exec_on_ui_thread(move |env, _| {
            let i = if let Some(i) = i {
                env.call_static_method(
                    jni_str!("java/lang/Integer"),
                    jni_str!("valueOf"),
                    jni_sig!("(I)Ljava/lang/Integer;"),
                    &[(i as i32).into()],
                )?
                .l()?
            } else {
                JObject::null()
            };
            env.call_method(
                w.as_obj(),
                jni_str!("setSelection"),
                jni_sig!("(Ljava/lang/Integer;)V"),
                &[(&i).into()],
            )?
            .v()
        })
        .unwrap();
    }

    pub(crate) fn len(&self) -> usize {
        let w = self.duplicate();
        vm_exec_on_ui_thread(move |env, _| {
            env.call_method(
                w.as_obj(),
                jni::jni_str!("getLength"),
                jni::jni_sig!("()I"),
                &[],
            )?
            .i()
        })
        .unwrap() as _
    }

    pub(crate) fn is_editable(&self) -> bool {
        let w = self.duplicate();
        vm_exec_on_ui_thread(move |env, _| {
            env.call_method(
                w.as_obj(),
                jni::jni_str!("isEditable"),
                jni::jni_sig!("()Z"),
                &[],
            )?
            .z()
        })
        .unwrap()
    }

    pub(crate) fn set_editable(&self, editable: bool) {
        let w = self.duplicate();
        vm_exec_on_ui_thread(move |env, _| {
            env.call_method(
                w.as_obj(),
                jni::jni_str!("setEditable"),
                jni::jni_sig!("(Z)V"),
                &[editable.into()],
            )?
            .v()
        })
        .unwrap();
    }

    pub(crate) fn is_empty(&self) -> bool {
        let w = self.duplicate();
        vm_exec_on_ui_thread(move |env, _| {
            env.call_method(
                w.as_obj(),
                jni::jni_str!("isEmpty"),
                jni::jni_sig!("()Z"),
                &[],
            )?
            .z()
        })
        .unwrap()
    }

    pub fn as_widget(&self) -> BorrowedWidget<'_> {
        unsafe { BorrowedWidget::android(&&self.inner) }
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
