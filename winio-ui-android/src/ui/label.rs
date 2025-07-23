use {
    super::{BaseWidget, vm_exec_on_ui_thread},
    inherit_methods_macro::inherit_methods,
    winio_handle::{AsRawWindow, AsWindow, impl_as_widget},
    winio_primitive::{HAlign, Point, Size},
};

#[derive(Debug)]
pub struct Label {
    inner: BaseWidget,
}

//noinspection SpellCheckingInspection
#[inherit_methods(from = "self.inner")]
impl Label {
    const WIDGET_CLASS: &'static str = "rs/compio/winio/Label";
    const HALIGN_LEFT: i32 = 0;
    const HALIGN_CENTER: i32 = 1;
    const HALIGN_RIGHT: i32 = 2;
    const HALIGN_STRETCH: i32 = 3;

    pub fn text(&self) -> String;

    pub fn set_text<S>(&mut self, _text: S)
    where
        S: AsRef<str>;

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

    pub fn set_halign(&mut self, align: HAlign) {
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

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, visible: bool);

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

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, size: Size);

    pub fn preferred_size(&self) -> Size {
        todo!()
    }

    pub fn new<W>(parent: W) -> Self
    where
        W: AsWindow,
    {
        let parent = parent.as_window().as_raw_window();
        let inner = vm_exec_on_ui_thread(move |mut env, _| {
            let widget = env.new_object(
                Self::WIDGET_CLASS,
                "(Lrs/compio/winio/Window;)V",
                &[parent.as_obj().into()],
            )?;
            env.new_global_ref(widget)
        })
        .unwrap()
        .into();

        Self { inner }
    }
}

impl_as_widget!(Label, inner);
