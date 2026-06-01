use std::sync::Arc;

use inherit_methods_macro::inherit_methods;
use jni::{objects::JObject, refs::LoaderContext};
use jni_min_helper::DynamicProxy;
use winio_callback::SyncCallback;
use winio_handle::{AsContainer, impl_as_widget};
use winio_primitive::{HAlign, Point, Size};

use crate::{ATextView, AView, BaseWidget, Context, Result, gravity, vm_exec};

jni::bind_java_type! {
    AEditText => android.widget.EditText,
    type_map {
        AView => android.view.View,
        ATextView => android.widget.TextView,
        Context => android.content.Context,
        Editable => android.text.Editable,
    },
    constructors {
        fn new(&Context),
    },
    methods {
        fn get_text() -> Editable,
        fn get_input_type() -> jint,
        fn set_input_type(ty: jint),
    },
    is_instance_of = {
        view = AView,
        text_view = ATextView,
    }
}

jni::bind_java_type! {
    Editable => android.text.Editable,
    methods {
        fn to_string() -> JString,
    }
}

#[derive(Debug)]
pub struct Edit {
    inner: BaseWidget<AEditText<'static>>,
    on_change: Arc<SyncCallback>,
    #[allow(dead_code)]
    change_proxy: DynamicProxy,
}

mod input_type {
    pub const TYPE_CLASS_TEXT: i32 = 0x1;
    pub const TYPE_TEXT_VARIATION_PASSWORD: i32 = 0x80;
    pub const TYPE_TEXT_FLAG_MULTI_LINE: i32 = 0x20000;
}

#[inherit_methods(from = "self.inner")]
impl Edit {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let on_change = Arc::new(SyncCallback::new());
        vm_exec(|env| {
            let act = crate::current_activity(env)?;
            let widget = AEditText::new(env, act)?;
            let change_proxy = DynamicProxy::build(
                env,
                &LoaderContext::None,
                [jni::jni_str!("android/text/TextWatcher")],
                {
                    let on_change = on_change.clone();
                    move |env, method, _args| {
                        if method.get_name(env)?.to_string() == "onTextChanged" {
                            on_change.signal(());
                        }
                        Ok(JObject::null())
                    }
                },
            )?;
            let inner = BaseWidget::new_with_env(env, parent.as_container(), widget)?;
            env.call_method(
                inner.as_obj(),
                jni::jni_str!("addTextChangedListener"),
                jni::jni_sig!("(Landroid/text/TextWatcher;)V"),
                &[change_proxy.as_ref().into()],
            )?
            .v()?;
            Ok(Self {
                inner,
                on_change,
                change_proxy,
            })
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

    pub(crate) fn min_size(&self) -> Result<Size>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    fn as_text_view(&self) -> &ATextView<'static> {
        self.inner.as_ref()
    }

    pub fn text(&self) -> Result<String> {
        vm_exec(move |env| {
            Ok(self
                .inner
                .get_text(env)?
                .to_string(env)?
                .try_to_string(env)?)
        })
    }

    pub fn set_text(&mut self, text: impl AsRef<str>) -> Result<()> {
        vm_exec(move |env| {
            let text = env.new_string(&text)?;
            self.as_text_view().set_text(env, text)?;
            Ok(())
        })
    }

    pub fn halign(&self) -> Result<HAlign> {
        let gravity = vm_exec(|env| self.as_text_view().get_gravity(env))?;
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
            self.as_text_view().set_gravity(env, gravity)?;
            Ok(())
        })
    }

    pub(crate) fn input_type(&self) -> Result<i32> {
        vm_exec(move |env| {
            let ty = self.inner.get_input_type(env)?;
            Ok(ty)
        })
    }

    pub(crate) fn set_input_type(&mut self, ty: i32) -> Result<()> {
        vm_exec(move |env| {
            self.inner.set_input_type(env, ty)?;
            Ok(())
        })
    }

    pub fn is_password(&self) -> Result<bool> {
        let ty = self.input_type()?;
        Ok((ty & input_type::TYPE_TEXT_VARIATION_PASSWORD) != 0)
    }

    pub fn set_password(&mut self, password: bool) -> Result<()> {
        let mut ty = self.input_type()?;
        if password {
            ty |= input_type::TYPE_TEXT_VARIATION_PASSWORD;
        } else {
            ty &= !input_type::TYPE_TEXT_VARIATION_PASSWORD;
        }
        self.set_input_type(ty)
    }

    pub fn is_readonly(&self) -> Result<bool> {
        let ty = self.input_type()?;
        Ok((ty & input_type::TYPE_CLASS_TEXT) != 0)
    }

    pub fn set_readonly(&mut self, readonly: bool) -> Result<()> {
        let mut ty = self.input_type()?;
        if readonly {
            ty |= input_type::TYPE_CLASS_TEXT;
        } else {
            ty &= !input_type::TYPE_CLASS_TEXT;
        }
        self.set_input_type(ty)
    }

    pub async fn wait_change(&self) {
        self.on_change.wait().await;
    }
}

impl_as_widget!(Edit, inner);

#[derive(Debug)]
pub struct TextBox {
    inner: Edit,
}

#[inherit_methods(from = "self.inner")]
impl TextBox {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let mut inner = Edit::new(parent)?;
        inner
            .set_input_type(input_type::TYPE_CLASS_TEXT | input_type::TYPE_TEXT_FLAG_MULTI_LINE)?;
        vm_exec(|env| {
            inner
                .as_text_view()
                .set_gravity(env, gravity::LEFT | gravity::TOP)
        })?;
        Ok(Self { inner })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, visible: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn min_size(&self) -> Result<Size>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, text: impl AsRef<str>) -> Result<()>;

    pub fn halign(&self) -> Result<HAlign>;

    pub fn set_halign(&mut self, align: HAlign) -> Result<()>;

    pub fn is_readonly(&self) -> Result<bool>;

    pub fn set_readonly(&mut self, readonly: bool) -> Result<()>;

    pub async fn wait_change(&self) {
        self.inner.wait_change().await;
    }
}

impl_as_widget!(TextBox, inner);
