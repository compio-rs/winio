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
            widget::{
                AdapterViewOnItemClickListener, ArrayAdapter, ListView as AListView,
                abs_list_view::{CHOICE_MODE_MULTIPLE, CHOICE_MODE_SINGLE},
            },
        },
        util::ArrayList,
    },
    vm_exec,
};

#[derive(Debug)]
pub struct ListBox {
    inner: BaseWidget<AListView<'static>>,
    list: Global<JList<'static>>,
    adapter: Global<ArrayAdapter<'static>>,
    on_select: Arc<SyncCallback>,
    #[allow(dead_code)]
    select_proxy: DynamicProxy,
}

#[inherit_methods(from = "self.inner")]
impl ListBox {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        vm_exec(|env| {
            let act = current_activity(env)?;
            let widget = AListView::new(env, &act)?;
            let on_select = Arc::new(SyncCallback::new());
            let select_proxy = DynamicProxy::build(
                env,
                &jni::refs::LoaderContext::None,
                [AdapterViewOnItemClickListener::class_name()],
                {
                    let on_select = on_select.clone();
                    move |_env, _method, _args| {
                        on_select.signal(());
                        Ok(JObject::null())
                    }
                },
            )?;
            widget.set_on_item_click_listener(env, &select_proxy)?;
            let inner = BaseWidget::new_with_env(env, parent.as_container(), widget)?;
            let list = JList::from(ArrayList::new(env)?);
            let list = env.new_global_ref(list)?;
            let adapter =
                ArrayAdapter::new(env, &act, Layout::simple_list_item_activated_1(env)?, &list)?;
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

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn min_size(&self) -> Result<Size>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn is_multiple(&self) -> Result<bool> {
        vm_exec(|env| {
            let mode = self.inner.get_choice_mode(env)?;
            Ok(mode == CHOICE_MODE_MULTIPLE)
        })
    }

    pub fn set_multiple(&mut self, v: bool) -> Result<()> {
        vm_exec(|env| {
            let mode = if v {
                CHOICE_MODE_MULTIPLE
            } else {
                CHOICE_MODE_SINGLE
            };
            self.inner.set_choice_mode(env, mode)?;
            Ok(())
        })
    }

    fn invalidate(&self, env: &mut Env) -> Result<()> {
        self.adapter.notify_data_set_changed(env)?;
        Ok(())
    }

    fn selection(&self, env: &mut Env) -> Result<Vec<usize>> {
        let sel = self.inner.get_checked_item_positions(env)?;
        let mut result = Vec::new();
        for i in 0..sel.size(env)? {
            if sel.value_at(env, i)? {
                result.push(sel.key_at(env, i)? as usize);
            }
        }
        Ok(result)
    }

    pub fn is_selected(&self, i: usize) -> Result<bool> {
        vm_exec(|env| {
            let sel = self.selection(env)?;
            Ok(sel.contains(&i))
        })
    }

    pub fn set_selected(&mut self, i: usize, v: bool) -> Result<()> {
        vm_exec(|env| {
            self.inner.set_item_checked(env, i as _, v)?;
            Ok(())
        })
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

    pub async fn wait_select(&self) {
        self.on_select.wait().await;
    }
}

impl_as_widget!(ListBox, inner);
