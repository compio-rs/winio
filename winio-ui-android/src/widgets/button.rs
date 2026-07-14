use std::sync::Arc;

use inherit_methods_macro::inherit_methods;
use jni::{
    Env,
    objects::JObject,
    refs::{LoaderContext, Reference},
};
use jni_min_helper::DynamicProxy;
use winio_callback::SyncCallback;
use winio_handle::{AsContainer, AsWidget, BorrowedWidget, impl_as_widget};
use winio_primitive::{Point, Size};

use crate::{
    ATextView, AView, BaseWidget, Context, JCharSequenceExt, Result, current_activity,
    impl_listener, vm_exec,
};

jni::bind_java_type! {
    AButton => android.widget.Button,
    type_map {
        AView => android.view.View,
        ATextView => android.widget.TextView,
        Context => android.content.Context,
        OnClickListener => "android.view.View$OnClickListener"
    },
    constructors {
        fn new(context: &Context),
    },
    methods {
        fn set_on_click_listener(listener: &OnClickListener),
    },
    is_instance_of = {
        view = AView,
        text_view = ATextView,
    }
}

jni::bind_java_type! {
    MaterialButton => com.google.android.material.button.MaterialButton,
    type_map {
        AButton => android.widget.Button,
        AView => android.view.View,
        ATextView => android.widget.TextView,
        Context => android.content.Context,
    },
    constructors {
        fn new(context: &Context),
    },
    is_instance_of = {
        button = AButton,
        view = AView,
        text_view = ATextView,
    }
}

jni::bind_java_type! {
    ACompoundButton => android.widget.CompoundButton,
    type_map {
        AButton => android.widget.Button,
    },
    methods {
        fn is_checked() -> jboolean,
        fn set_checked(checked: jboolean),
    },
    is_instance_of = {
        button = AButton,
    }
}

jni::bind_java_type! {
    ACheckBox => com.google.android.material.checkbox.MaterialCheckBox,
    type_map {
        AButton => android.widget.Button,
        ACompoundButton => android.widget.CompoundButton,
        AView => android.view.View,
        ATextView => android.widget.TextView,
        Context => android.content.Context,
    },
    constructors {
        fn new(context: &Context),
    },
    is_instance_of = {
        button = AButton,
        compound_button = ACompoundButton,
        view = AView,
        text_view = ATextView,
    }
}

jni::bind_java_type! {
    ARadioButton => com.google.android.material.radiobutton.MaterialRadioButton,
    type_map {
        AButton => android.widget.Button,
        ACompoundButton => android.widget.CompoundButton,
        AView => android.view.View,
        ATextView => android.widget.TextView,
        Context => android.content.Context,
    },
    constructors {
        fn new(context: &Context),
    },
    is_instance_of = {
        button = AButton,
        compound_button = ACompoundButton,
        view = AView,
        text_view = ATextView,
    }
}

jni::bind_java_type! {
    OnClickListener => "android.view.View$OnClickListener",
}

impl_listener!(OnClickListener);

#[derive(Debug)]
struct ButtonImpl<T>
where
    T: Into<JObject<'static>>
        + AsRef<JObject<'static>>
        + AsRef<AView<'static>>
        + Default
        + Reference
        + Send
        + Sync
        + 'static,
{
    inner: BaseWidget<T>,
    on_click: Arc<SyncCallback>,
    #[allow(dead_code)]
    click_proxy: DynamicProxy,
}

impl<T> ButtonImpl<T>
where
    T: Into<JObject<'static>>
        + AsRef<JObject<'static>>
        + AsRef<AView<'static>>
        + AsRef<AButton<'static>>
        + Default
        + Reference
        + Send
        + Sync
        + 'static,
{
    pub fn new<'any_local, O>(env: &mut Env, parent: impl AsContainer, widget: O) -> Result<Self>
    where
        O: Reference<GlobalKind = T> + AsRef<JObject<'any_local>> + AsRef<AButton<'any_local>>,
    {
        let on_click = Arc::new(SyncCallback::new());
        let click_proxy = DynamicProxy::build(
            env,
            &LoaderContext::None,
            [OnClickListener::class_name()],
            {
                let on_click = on_click.clone();
                move |_env, _method, _args| {
                    on_click.signal(());
                    Ok(JObject::null())
                }
            },
        )?;
        AsRef::<AButton>::as_ref(&widget).set_on_click_listener(env, &click_proxy)?;
        let inner = BaseWidget::new_with_env(env, parent.as_container(), widget)?;
        Ok(Self {
            inner,
            on_click,
            click_proxy,
        })
    }
}

#[inherit_methods(from = "self.inner")]
impl<T> ButtonImpl<T>
where
    T: Into<JObject<'static>>
        + AsRef<JObject<'static>>
        + AsRef<AView<'static>>
        + Default
        + Reference
        + Send
        + Sync
        + 'static,
{
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

    pub async fn wait_click(&self) {
        self.on_click.wait().await;
    }
}

impl<T> ButtonImpl<T>
where
    T: Into<JObject<'static>>
        + AsRef<JObject<'static>>
        + AsRef<AView<'static>>
        + AsRef<ATextView<'static>>
        + Default
        + Reference
        + Send
        + Sync
        + 'static,
{
    fn as_text_view(&self) -> &ATextView<'static> {
        self.inner.as_ref()
    }

    pub fn text(&self) -> Result<String> {
        vm_exec(move |env| Ok(self.as_text_view().get_text(env)?.try_to_string(env)?))
    }

    pub fn set_text(&mut self, text: impl AsRef<str>) -> Result<()> {
        vm_exec(move |env| {
            let text = env.new_string(&text)?;
            self.as_text_view().set_text(env, text)?;
            Ok(())
        })
    }
}

impl<T> ButtonImpl<T>
where
    T: Into<JObject<'static>>
        + AsRef<JObject<'static>>
        + AsRef<AView<'static>>
        + AsRef<ACompoundButton<'static>>
        + Default
        + Reference
        + Send
        + Sync
        + 'static,
{
    fn as_compound_button(&self) -> &ACompoundButton<'static> {
        self.inner.as_ref()
    }

    pub fn is_checked(&self) -> Result<bool> {
        vm_exec(|env| Ok(self.as_compound_button().is_checked(env)?))
    }

    pub fn set_checked(&mut self, checked: bool) -> Result<()> {
        vm_exec(|env| {
            self.as_compound_button().set_checked(env, checked)?;
            Ok(())
        })
    }
}

impl<T> AsWidget for ButtonImpl<T>
where
    T: Into<JObject<'static>>
        + AsRef<JObject<'static>>
        + AsRef<AView<'static>>
        + Default
        + Reference
        + Send
        + Sync
        + 'static,
{
    fn as_widget(&self) -> BorrowedWidget<'_> {
        self.inner.as_widget()
    }
}

#[derive(Debug)]
pub struct Button {
    inner: ButtonImpl<MaterialButton<'static>>,
}

#[inherit_methods(from = "self.inner")]
impl Button {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        vm_exec(|env| {
            let act = current_activity(env)?;
            let widget = MaterialButton::new(env, act)?;
            let inner = ButtonImpl::new(env, parent, widget)?;
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

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, text: impl AsRef<str>) -> Result<()>;

    pub async fn wait_click(&self) {
        self.inner.wait_click().await;
    }
}

impl_as_widget!(Button, inner);

#[derive(Debug)]
pub struct CheckBox {
    inner: ButtonImpl<ACheckBox<'static>>,
}

#[inherit_methods(from = "self.inner")]
impl CheckBox {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        vm_exec(|env| {
            let act = current_activity(env)?;
            let widget = ACheckBox::new(env, act)?;
            let inner = ButtonImpl::new(env, parent, widget)?;
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

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, text: impl AsRef<str>) -> Result<()>;

    pub fn is_checked(&self) -> Result<bool>;

    pub fn set_checked(&mut self, checked: bool) -> Result<()>;

    pub async fn wait_click(&self) {
        self.inner.wait_click().await;
    }
}

impl_as_widget!(CheckBox, inner);

#[derive(Debug)]
pub struct RadioButton {
    inner: ButtonImpl<ARadioButton<'static>>,
}

#[inherit_methods(from = "self.inner")]
impl RadioButton {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        vm_exec(|env| {
            let act = current_activity(env)?;
            let widget = ARadioButton::new(env, act)?;
            let inner = ButtonImpl::new(env, parent, widget)?;
            Ok(Self { inner })
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, visible: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn is_checked(&self) -> Result<bool>;

    pub fn set_checked(&mut self, _v: bool) -> Result<()>;

    pub async fn wait_click(&self) {
        self.inner.wait_click().await
    }
}

impl_as_widget!(RadioButton, inner);
