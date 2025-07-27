use {
    super::{super::JObjectExt, vm_exec, vm_exec_on_ui_thread},
    std::ops::Deref,
    winio_handle::{AsRawWindow, BorrowedWindow, RawWidget},
    winio_primitive::{HAlign, Point, Size},
};

#[derive(Clone, Debug)]
pub struct BaseWidget {
    inner: RawWidget,
}

//noinspection SpellCheckingInspection
impl BaseWidget {
    const HALIGN_LEFT: i32 = 0;
    const HALIGN_CENTER: i32 = 1;
    const HALIGN_RIGHT: i32 = 2;
    const HALIGN_STRETCH: i32 = 3;

    pub(crate) fn new(inner: RawWidget) -> Self {
        Self { inner }
    }

    pub(crate) fn create<S, T>(parent: BorrowedWindow, widget_class: S) -> T
    where
        S: AsRef<str>,
        T: From<Self>,
    {
        let parent = parent.as_raw_window();
        let widget_class = widget_class.as_ref().to_owned();
        let inner = vm_exec_on_ui_thread(move |mut env, _| {
            let widget = env.new_object(
                &widget_class,
                "(Lrs/compio/winio/Window;)V",
                &[parent.as_obj().into()],
            )?;
            env.new_global_ref(widget)
        })
        .unwrap()
        .into();

        T::from(inner)
    }

    pub(crate) fn hash_code(&self) -> i32 {
        vm_exec(|mut env, _| {
            env.call_method(self.inner.as_obj(), "hashCode", "()I", &[])?
                .i()
        })
        .unwrap()
    }

    pub(crate) fn loc(&self) -> Point {
        let w = self.inner.clone();
        vm_exec_on_ui_thread(move |mut env, _| {
            env.call_method(w.as_obj(), "getLoc", "()[D", &[])?
                .l()?
                .to(&mut env)
        })
        .unwrap()
    }

    pub(crate) fn set_loc(&self, p: Point) {
        let w = self.inner.clone();
        vm_exec_on_ui_thread(move |mut env, _| {
            env.call_method(w.as_obj(), "setLoc", "(DD)V", &[p.x.into(), p.y.into()])?
                .v()
        })
        .unwrap();
    }

    pub(crate) fn size(&self) -> Size {
        let w = self.inner.clone();
        vm_exec_on_ui_thread(move |mut env, _| {
            env.call_method(w.as_obj(), "getSize", "()[D", &[])?
                .l()?
                .to(&mut env)
        })
        .unwrap()
    }

    pub(crate) fn set_size(&self, size: Size) {
        let w = self.inner.clone();
        vm_exec_on_ui_thread(move |mut env, _| {
            env.call_method(
                w.as_obj(),
                "setSize",
                "(DD)V",
                &[size.width.into(), size.height.into()],
            )?
            .v()
        })
        .unwrap();
    }

    pub(crate) fn preferred_size(&self) -> Size {
        let w = self.inner.clone();
        vm_exec_on_ui_thread(move |mut env, _| {
            env.call_method(w.as_obj(), "getPreferredSize", "()[D", &[])?
                .l()?
                .to(&mut env)
        })
        .unwrap()
    }

    pub(crate) fn is_visible(&self) -> bool {
        let w = self.inner.clone();
        vm_exec_on_ui_thread(move |mut env, _| {
            env.call_method(w.as_obj(), "isVisible", "()Z", &[])?.z()
        })
        .unwrap()
    }

    pub(crate) fn set_visible(&self, visible: bool) {
        let w = self.inner.clone();
        vm_exec_on_ui_thread(move |mut env, _| {
            env.call_method(w.as_obj(), "setVisible", "(Z)V", &[visible.into()])?
                .v()
        })
        .unwrap();
    }

    pub(crate) fn text(&self) -> String {
        let w = self.inner.clone();
        vm_exec_on_ui_thread(move |mut env, _| {
            env.call_method(w.as_obj(), "getText", "()Ljava/lang/CharSequence;", &[])?
                .l()?
                .to(&mut env)
        })
        .unwrap()
    }

    pub(crate) fn set_text<S>(&self, text: S)
    where
        S: AsRef<str>,
    {
        let text = text.as_ref().to_owned();
        let w = self.inner.clone();
        vm_exec_on_ui_thread(move |mut env, _| {
            let text = env.new_string(&text)?;
            env.call_method(
                w.as_obj(),
                "setText",
                "(Ljava/lang/CharSequence;)V",
                &[(&text).into()],
            )?
            .v()
        })
        .unwrap();
    }

    pub fn halign(&self) -> HAlign {
        let w = self.inner.clone();
        let align = vm_exec_on_ui_thread(move |mut env, _| {
            env.call_method(w.as_obj(), "getHAlign", "()I", &[])?.i()
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
        let w = self.inner.clone();
        vm_exec_on_ui_thread(move |mut env, _| {
            env.call_method(w.as_obj(), "setHAlign", "(I)V", &[value.into()])?
                .v()
        })
        .unwrap();
    }

    pub(crate) fn is_checked(&self) -> bool {
        let w = self.inner.clone();
        vm_exec_on_ui_thread(move |mut env, _| {
            env.call_method(w.as_obj(), "isChecked", "()Z", &[])?.z()
        })
        .unwrap()
    }

    pub(crate) fn set_checked(&self, checked: bool) {
        let w = self.inner.clone();
        vm_exec_on_ui_thread(move |mut env, _| {
            env.call_method(w.as_obj(), "setChecked", "(Z)V", &[checked.into()])?
                .v()
        })
        .unwrap();
    }

    pub(crate) fn is_enabled(&self) -> bool {
        let w = self.inner.clone();
        vm_exec_on_ui_thread(move |mut env, _| {
            env.call_method(w.as_obj(), "isEnabled", "()Z", &[])?.z()
        })
        .unwrap()
    }

    pub(crate) fn set_enabled(&self, enabled: bool) {
        let w = self.inner.clone();
        vm_exec_on_ui_thread(move |mut env, _| {
            env.call_method(w.as_obj(), "setEnabled", "(Z)V", &[enabled.into()])?
                .v()
        })
        .unwrap();
    }

    pub(crate) fn range(&self) -> (usize, usize) {
        let w = self.inner.clone();
        vm_exec_on_ui_thread(move |mut env, _| {
            env.call_method(w.as_obj(), "getRange", "()[I", &[])?
                .l()?
                .to(&mut env)
        })
        .unwrap()
    }

    pub(crate) fn set_range(&self, min: usize, max: usize) {
        let w = self.inner.clone();
        vm_exec_on_ui_thread(move |mut env, _| {
            env.call_method(
                w.as_obj(),
                "setRange",
                "(II)V",
                &[(min as i32).into(), (max as i32).into()],
            )?
            .v()
        })
        .unwrap();
    }

    pub(crate) fn pos(&self) -> usize {
        let w = self.inner.clone();
        vm_exec_on_ui_thread(move |mut env, _| {
            env.call_method(w.as_obj(), "getPos", "()I", &[])?.i()
        })
        .unwrap() as _
    }

    pub(crate) fn set_pos(&self, pos: usize) {
        let w = self.inner.clone();
        vm_exec_on_ui_thread(move |mut env, _| {
            env.call_method(w.as_obj(), "setPos", "(I)V", &[(pos as i32).into()])?
                .v()
        })
        .unwrap();
    }

    pub(crate) fn is_indeterminate(&self) -> bool {
        let w = self.inner.clone();
        vm_exec_on_ui_thread(move |mut env, _| {
            env.call_method(w.as_obj(), "isIndeterminate", "()Z", &[])?
                .z()
        })
        .unwrap()
    }

    pub(crate) fn set_indeterminate(&self, indeterminate: bool) {
        let w = self.inner.clone();
        vm_exec_on_ui_thread(move |mut env, _| {
            env.call_method(
                w.as_obj(),
                "setIndeterminate",
                "(Z)V",
                &[indeterminate.into()],
            )?
            .v()
        })
        .unwrap();
    }

    pub(crate) fn minimum(&self) -> usize {
        let w = self.inner.clone();
        vm_exec_on_ui_thread(move |mut env, _| {
            env.call_method(w.as_obj(), "getMinimum", "()I", &[])?.i()
        })
        .unwrap() as _
    }

    pub(crate) fn set_minimum(&self, minimum: usize) {
        let w = self.inner.clone();
        vm_exec_on_ui_thread(move |mut env, _| {
            env.call_method(w.as_obj(), "setMinimum", "(I)V", &[(minimum as i32).into()])?
                .v()
        })
        .unwrap();
    }

    pub(crate) fn maximum(&self) -> usize {
        let w = self.inner.clone();
        vm_exec_on_ui_thread(move |mut env, _| {
            env.call_method(w.as_obj(), "getMaximum", "()I", &[])?.i()
        })
        .unwrap() as _
    }

    pub(crate) fn set_maximum(&self, maximum: usize) {
        let w = self.inner.clone();
        vm_exec_on_ui_thread(move |mut env, _| {
            env.call_method(w.as_obj(), "setMaximum", "(I)V", &[(maximum as i32).into()])?
                .v()
        })
        .unwrap();
    }
}

impl From<RawWidget> for BaseWidget {
    fn from(value: RawWidget) -> Self {
        Self::new(value)
    }
}

impl Deref for BaseWidget {
    type Target = RawWidget;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
