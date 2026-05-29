use std::sync::Arc;

use inherit_methods_macro::inherit_methods;
use jni::{
    Env,
    objects::{JList, JObject},
    refs::Global,
};
use jni_min_helper::DynamicProxy;
use winio_callback::SyncCallback;
use winio_handle::{AsContainer, impl_as_widget};
use winio_primitive::{Point, Size};

use crate::{BaseWidget, GlobalRef, JObjectExt, Result, vm_exec};

jni::bind_java_type! {
    Layout => "android.R$layout",
    fields {
        static simple_spinner_item {
            sig = jint,
            name = "simple_spinner_item",
        },
        static simple_spinner_dropdown_item {
            sig = jint,
            name = "simple_spinner_dropdown_item",
        },
    }
}

#[derive(Debug)]
pub struct ComboBox {
    inner: BaseWidget,
    list: Global<JList<'static>>,
    adapter: GlobalRef,
    on_select: Arc<SyncCallback>,
    #[allow(dead_code)]
    select_proxy: DynamicProxy,
}

#[inherit_methods(from = "self.inner")]
impl ComboBox {
    const WIDGET_CLASS: &'static str = "android/widget/Spinner";

    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let on_select = Arc::new(SyncCallback::new());
        vm_exec(|env| {
            let select_proxy = DynamicProxy::build(
                env,
                &jni::refs::LoaderContext::None,
                [jni::jni_str!(
                    "android/widget/AdapterView$OnItemSelectedListener"
                )],
                {
                    let on_select = on_select.clone();
                    move |env, method, _args| {
                        if method.get_name(env)?.to_string() == "onItemSelected" {
                            on_select.signal(());
                        }
                        Ok(JObject::null())
                    }
                },
            )?;
            let inner = BaseWidget::new_with_env(env, parent.as_container(), Self::WIDGET_CLASS)?;
            env.call_method(
                inner.as_obj(),
                jni::jni_str!("setOnItemSelectedListener"),
                jni::jni_sig!("(Landroid/widget/AdapterView$OnItemSelectedListener;)V"),
                &[select_proxy.as_ref().into()],
            )?
            .v()?;
            let list = env.new_object(
                jni::jni_str!("java/util/ArrayList"),
                jni::jni_sig!("()V"),
                &[],
            )?;
            let list = unsafe { JList::from_raw(env, list.into_raw()) };
            let list = env.new_global_ref(list)?;
            let context = crate::current_activity()?;
            let adapter = env.new_object(
                jni::jni_str!("android/widget/ArrayAdapter"),
                jni::jni_sig!("(Landroid/content/Context;ILjava/util/List;)V"),
                &[
                    context.as_obj().into(),
                    Layout::simple_spinner_item(env)?.into(),
                    list.as_obj().into(),
                ],
            )?;
            env.call_method(
                &adapter,
                jni::jni_str!("setDropDownViewResource"),
                jni::jni_sig!("(I)V"),
                &[Layout::simple_spinner_dropdown_item(env)?.into()],
            )?;
            env.call_method(
                inner.as_obj(),
                jni::jni_str!("setAdapter"),
                jni::jni_sig!("(Landroid/widget/SpinnerAdapter;)V"),
                &[(&adapter).into()],
            )?
            .v()?;
            let adapter = env.new_global_ref(adapter)?;
            Ok(Self {
                inner,
                list,
                adapter,
                on_select,
                select_proxy,
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

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn text(&self) -> Result<String> {
        match self.selection()? {
            Some(i) => self.get(i),
            None => Ok(String::new()),
        }
    }

    pub fn set_text(&mut self, _s: impl AsRef<str>) -> Result<()> {
        Ok(())
    }

    fn invalidate(&self, env: &mut Env) -> Result<()> {
        env.call_method(
            self.adapter.as_obj(),
            jni::jni_str!("notifyDataSetChanged"),
            jni::jni_sig!("()V"),
            &[],
        )?
        .v()?;
        Ok(())
    }

    pub fn selection(&self) -> Result<Option<usize>> {
        vm_exec(|env| {
            let pos = env
                .call_method(
                    self.inner.as_obj(),
                    jni::jni_str!("getSelectedItemPosition"),
                    jni::jni_sig!("()I"),
                    &[],
                )?
                .i()?;
            if pos >= 0 {
                Ok(Some(pos as _))
            } else {
                Ok(None)
            }
        })
    }

    pub fn set_selection(&mut self, i: usize) -> Result<()> {
        vm_exec(|env| {
            env.call_method(
                self.inner.as_obj(),
                jni::jni_str!("setSelection"),
                jni::jni_sig!("(I)V"),
                &[(i as i32).into()],
            )?
            .v()?;
            Ok(())
        })
    }

    pub fn is_editable(&self) -> Result<bool> {
        Ok(false)
    }

    pub fn set_editable(&mut self, _v: bool) -> Result<()> {
        Ok(())
    }

    pub fn len(&self) -> Result<usize> {
        vm_exec(|env| {
            let size = self.list.size(env)?;
            Ok(size as _)
        })
    }

    pub fn is_empty(&self) -> Result<bool> {
        vm_exec(|env| {
            let empty = self.list.is_empty(env)?;
            Ok(empty)
        })
    }

    pub fn clear(&mut self) -> Result<()> {
        vm_exec(|env| {
            self.list.clear(env)?;
            self.invalidate(env)?;
            Ok(())
        })
    }

    pub fn get(&self, i: usize) -> Result<String> {
        vm_exec(|env| self.list.get(env, i as _)?.to(env))
    }

    pub fn set(&mut self, i: usize, s: impl AsRef<str>) -> Result<()> {
        vm_exec(|env| {
            self.list.remove(env, i as _)?;
            let str = env.new_string(s.as_ref())?;
            self.list.insert(env, i as _, str)?;
            self.invalidate(env)?;
            Ok(())
        })
    }

    pub fn insert(&mut self, i: usize, s: impl AsRef<str>) -> Result<()> {
        vm_exec(|env| {
            let str = env.new_string(s.as_ref())?;
            self.list.insert(env, i as _, str)?;
            self.invalidate(env)?;
            Ok(())
        })
    }

    pub fn remove(&mut self, i: usize) -> Result<()> {
        vm_exec(|env| {
            self.list.remove(env, i as _)?;
            self.invalidate(env)?;
            Ok(())
        })
    }

    pub async fn wait_change(&self) {
        std::future::pending().await
    }

    pub async fn wait_select(&self) {
        self.on_select.wait().await
    }
}

impl_as_widget!(ComboBox, inner);
