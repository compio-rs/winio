use std::sync::{Arc, Mutex};

use inherit_methods_macro::inherit_methods;
use jni::{
    objects::{JObject, JString},
    refs::{Global, LoaderContext, Reference},
};
use jni_min_helper::{DynamicProxy, JInteger};
use winio_callback::SyncCallback;
use winio_handle::{AsContainer, AsWidget};
use winio_primitive::{Point, Size};

use crate::{
    BaseWidget, Result, View, WINDOW_RESIZE_CALLBACK, current_activity,
    java::{
        android::{
            view::{View as AView, ViewOnLayoutChangeListener},
            widget::{LinearLayout, LinearLayoutLayoutParams},
        },
        androidx::viewpager2::ViewPager2,
        custom::WinioTabViewAdapter,
        material::{
            TabLayout, TabLayoutMediator, TabLayoutMediatorTabConfigurationStrategy,
            TabLayoutOnTabSelectedListener, TabLayoutTab,
        },
    },
    vm_exec,
};

const VERTICAL: i32 = 1;

#[derive(Debug)]
#[allow(dead_code)]
pub struct TabView {
    handle: BaseWidget<LinearLayout<'static>>,
    tab: Global<TabLayout<'static>>,
    pager: Global<ViewPager2<'static>>,
    mediator: Global<TabLayoutMediator<'static>>,
    tab_titles: Arc<Mutex<Vec<Global<JString<'static>>>>>,
    tab_proxy: DynamicProxy,
    on_select: Arc<SyncCallback>,
    select_proxy: DynamicProxy,
    adapter: Global<WinioTabViewAdapter<'static>>,
}

#[inherit_methods(from = "self.handle")]
impl TabView {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        vm_exec(|env| {
            let act = current_activity(env)?;
            let layout = LinearLayout::new(env, &act)?;
            layout.set_orientation(env, VERTICAL)?;
            let layout = BaseWidget::new_with_env(env, parent.as_container(), layout)?;

            let tab = TabLayout::new(env, &act)?;
            let params = LinearLayoutLayoutParams::new(env, -1, -2)?;
            tab.as_view().set_layout_params(env, params)?;
            layout.as_view_group().add_view(env, &tab)?;
            let tab = env.new_global_ref(tab)?;

            let on_select = Arc::new(SyncCallback::new());
            let select_proxy = DynamicProxy::build(
                env,
                &LoaderContext::FromObject(&act),
                [TabLayoutOnTabSelectedListener::class_name()],
                {
                    let on_select = on_select.clone();
                    move |_env, _method, _args| {
                        on_select.signal(());
                        Ok(JObject::null())
                    }
                },
            )?;
            tab.add_on_tab_selected_listener(env, &select_proxy)?;

            let pager = ViewPager2::new(env, &act)?;
            let params = LinearLayoutLayoutParams::with_weight(env, -1, -1, 1.0)?;
            pager.as_view().set_layout_params(env, params)?;
            layout.as_view_group().add_view(env, &pager)?;
            let pager = env.new_global_ref(pager)?;

            let adapter = WinioTabViewAdapter::new(env)?;
            pager.set_adapter(env, &adapter)?;
            let adapter = env.new_global_ref(adapter)?;

            let tab_titles = Arc::new(Mutex::new(Vec::<Global<JString<'static>>>::new()));

            let tab_proxy = DynamicProxy::build(
                env,
                &LoaderContext::FromObject(&act),
                [TabLayoutMediatorTabConfigurationStrategy::class_name()],
                {
                    let titles = tab_titles.clone();
                    move |env, _method, args| {
                        let tab = args.get_element(env, 0)?;
                        let position = args.get_element(env, 1)?;
                        let position =
                            unsafe { JInteger::from_raw(env, position.into_raw()) }.value(env)?;
                        let titles = titles.lock().unwrap();
                        if let Some(title) = titles.get(position as usize) {
                            let tab = unsafe { TabLayoutTab::from_raw(env, tab.into_raw()) };
                            tab.set_text(env, title)?;
                        }
                        Ok(JObject::null())
                    }
                },
            )?;

            let mediator = TabLayoutMediator::new(env, &tab, &pager, &tab_proxy)?;
            mediator.attach(env)?;
            let mediator = env.new_global_ref(mediator)?;

            Ok(Self {
                handle: layout,
                tab,
                pager,
                mediator,
                tab_titles,
                tab_proxy,
                on_select,
                select_proxy,
                adapter,
            })
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn selection(&self) -> Result<Option<usize>> {
        vm_exec(|env| {
            let pos = self.tab.get_selected_tab_position(env)?;
            if pos >= 0 {
                Ok(Some(pos as usize))
            } else {
                Ok(None)
            }
        })
    }

    pub fn set_selection(&mut self, i: usize) -> Result<()> {
        vm_exec(|env| {
            let tab = self.tab.get_tab_at(env, i as _)?;
            self.tab.select_tab(env, &tab)?;
            Ok(())
        })
    }

    pub fn insert(&mut self, i: usize, item: &TabViewItem) -> Result<()> {
        vm_exec(|env| {
            let pages = self.adapter.get_pages(env)?;
            pages.add(env, item.handle.as_widget().to_android())?;
            let title = env.new_string(item.text()?)?;
            self.tab_titles
                .lock()
                .unwrap()
                .insert(i, env.new_global_ref(title)?);
            self.adapter.notify_item_inserted(env, i as _)?;
            Ok(())
        })
    }

    pub fn remove(&mut self, i: usize) -> Result<()> {
        vm_exec(|env| {
            let pages = self.adapter.get_pages(env)?;
            pages.remove(env, i as _)?;
            self.adapter.notify_item_removed(env, i as _)?;
            Ok(())
        })
    }

    pub fn len(&self) -> Result<usize> {
        vm_exec(|env| {
            let pages = self.adapter.get_pages(env)?;
            Ok(pages.size(env)? as usize)
        })
    }

    pub fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }

    pub fn clear(&mut self) -> Result<()> {
        vm_exec(|env| {
            let pages = self.adapter.get_pages(env)?;
            let len = pages.size(env)?;
            pages.clear(env)?;
            self.adapter.notify_item_range_removed(env, 0, len)?;
            Ok(())
        })
    }

    pub async fn wait_select(&self) {
        self.on_select.wait().await;
    }
}

winio_handle::impl_as_widget!(TabView, handle);

#[derive(Debug)]
pub struct TabViewItem {
    handle: View,
    text: String,
    #[allow(unused)]
    on_resize_proxy: DynamicProxy,
}

impl TabViewItem {
    pub fn new() -> Result<Self> {
        let handle = View::new_standalone()?;
        vm_exec(|env| {
            let on_resize_proxy = DynamicProxy::build(
                env,
                &LoaderContext::None,
                [ViewOnLayoutChangeListener::class_name()],
                {
                    move |env, method, args| {
                        let name = method.get_name(env)?;
                        if name.try_to_string(env)? == "onLayoutChange" {
                            let mut get_element = |i: usize| -> jni::errors::Result<i32> {
                                let obj = args.get_element(env, i)?;
                                let int = unsafe { JInteger::from_raw(env, obj.into_raw()) };
                                int.value(env)
                            };

                            let left = get_element(1)?;
                            let top = get_element(2)?;
                            let right = get_element(3)?;
                            let bottom = get_element(4)?;
                            let old_left = get_element(5)?;
                            let old_top = get_element(6)?;
                            let old_right = get_element(7)?;
                            let old_bottom = get_element(8)?;

                            if left != old_left
                                || top != old_top
                                || right != old_right
                                || bottom != old_bottom
                            {
                                let callback = WINDOW_RESIZE_CALLBACK.lock().unwrap();
                                if let Some(callback) = callback.as_ref() {
                                    callback.signal(());
                                }
                            }
                        }
                        Ok(JObject::null())
                    }
                },
            )?;
            let view = handle.as_widget().to_android();
            let view = env.new_local_ref(view)?;
            let view = unsafe { AView::from_raw(env, view.into_raw()) };
            view.add_on_layout_change_listener(env, &on_resize_proxy)?;
            Ok(Self {
                handle,
                text: String::new(),
                on_resize_proxy,
            })
        })
    }

    pub fn text(&self) -> Result<String> {
        Ok(self.text.clone())
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.text = s.as_ref().into();
        Ok(())
    }

    pub fn size(&self) -> Result<Size> {
        self.handle.size()
    }
}

winio_handle::impl_as_container!(TabViewItem, handle);
