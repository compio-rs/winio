use inherit_methods_macro::inherit_methods;
use winio_handle::{AsContainer, impl_as_widget};
use winio_primitive::{HAlign, Point, Size};

use crate::{
    AView, BaseWidget, Context, JCharSequenceExt, MovementMethod, Result, current_activity,
    gravity, vm_exec,
};

jni::bind_java_type! {
    pub(crate) ATextView => android.widget.TextView,
    type_map {
        AView => android.view.View,
        Context => android.content.Context,
        MovementMethod => android.text.method.MovementMethod,
    },
    constructors {
        fn new(&Context),
    },
    methods {
        fn get_text() -> JCharSequence,
        fn set_text(text: &JCharSequence),
        fn get_gravity() -> jint,
        fn set_gravity(gravity: jint),
        fn set_movement_method(method: &MovementMethod),
    },
    is_instance_of = {
        view = AView,
    }
}

#[derive(Debug)]
pub struct Label {
    inner: BaseWidget<ATextView<'static>>,
}

#[inherit_methods(from = "self.inner")]
impl Label {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        vm_exec(|env| {
            let act = current_activity(env)?;
            let widget = ATextView::new(env, act)?;
            let inner = BaseWidget::new_with_env(env, parent.as_container(), widget)?;
            inner.set_gravity(env, gravity::CENTER_VERTICAL | gravity::LEFT)?;
            Ok(Self { inner })
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, visible: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn text(&self) -> Result<String> {
        vm_exec(move |env| Ok(self.inner.get_text(env)?.try_to_string(env)?))
    }

    pub fn set_text(&mut self, text: impl AsRef<str>) -> Result<()> {
        vm_exec(move |env| {
            let text = env.new_string(&text)?;
            self.inner.set_text(env, text)?;
            Ok(())
        })
    }

    pub fn halign(&self) -> Result<HAlign> {
        let gravity = vm_exec(|env| self.inner.get_gravity(env))?;
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
            self.inner.set_gravity(env, gravity)?;
            Ok(())
        })
    }
}

impl_as_widget!(Label, inner);
