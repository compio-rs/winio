use std::sync::Arc;

use inherit_methods_macro::inherit_methods;
use jni::{
    Env,
    objects::{JList, JObject, JString},
    refs::{Global, Reference},
};
use jni_min_helper::DynamicProxy;
use winio_callback::SyncCallback;
use winio_handle::{AsContainer, impl_as_widget};
use winio_primitive::{Point, Size};

use crate::{
    BaseWidget, Result, current_activity,
    java::{
        android::{
            r::Layout,
            widget::{AdapterViewOnItemSelectedListener, ArrayAdapter, Spinner as ASpinner},
        },
        util::ArrayList,
    },
    vm_exec,
};

#[derive(Debug)]
pub struct ComboBox {
    inner: BaseWidget<ASpinner<'static>>,
    list: Global<JList<'static>>,
    adapter: Global<ArrayAdapter<'static>>,
    on_select: Arc<SyncCallback>,
    #[allow(dead_code)]
    select_proxy: DynamicProxy,
}

#[inherit_methods(from = "self.inner")]
impl ComboBox {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let on_select = Arc::new(SyncCallback::new());
        vm_exec(|env| {
            let act = current_activity(env)?;
            let widget = ASpinner::new(env, &act)?;
            let select_proxy = DynamicProxy::build(
                env,
                &jni::refs::LoaderContext::None,
                [AdapterViewOnItemSelectedListener::class_name()],
                {
                    let on_select = on_select.clone();
                    move |env, method, _args| {
                        if method.get_name(env)?.try_to_string(env)? == "onItemSelected" {
                            on_select.signal(());
                        }
                        Ok(JObject::null())
                    }
                },
            )?;
            widget.set_on_item_selected_listener(env, &select_proxy)?;
            let inner = BaseWidget::new_with_env(env, parent.as_container(), widget)?;
            let list = JList::from(ArrayList::new(env)?);
            let list = env.new_global_ref(list)?;
            let adapter = ArrayAdapter::new(env, &act, Layout::simple_spinner_item(env)?, &list)?;
            adapter.set_drop_down_view_resource(env, Layout::simple_spinner_dropdown_item(env)?)?;
            inner.set_adapter(env, &adapter)?;
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
        self.adapter.notify_data_set_changed(env)?;
        Ok(())
    }

    pub fn selection(&self) -> Result<Option<usize>> {
        vm_exec(|env| {
            let pos = self.inner.get_selected_item_position(env)?;
            if pos >= 0 {
                Ok(Some(pos as _))
            } else {
                Ok(None)
            }
        })
    }

    pub fn set_selection(&mut self, i: usize) -> Result<()> {
        vm_exec(|env| {
            self.inner.set_selection(env, i as _)?;
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
        vm_exec(|env| {
            let item = self.list.get(env, i as _)?;
            let item = unsafe { JString::from_raw(env, item.into_raw()) };
            Ok(item.try_to_string(env)?)
        })
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
