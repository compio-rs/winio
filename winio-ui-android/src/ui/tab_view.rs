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
    AView, AViewGroup, BaseWidget, Context, Result, View, ViewGroupLayoutParams, current_activity,
    vm_exec,
};

jni::bind_java_type! {
    LinearLayout => android.widget.LinearLayout,
    type_map {
        AView => android.view.View,
        AViewGroup => android.view.ViewGroup,
        Context => android.content.Context,
    },
    constructors {
        fn new(&Context),
    },
    methods {
        fn set_orientation(orient: jint),
    },
    is_instance_of = {
        view = AView,
        view_group = AViewGroup,
    }
}

const VERTICAL: i32 = 1;

jni::bind_java_type! {
    LinearLayoutLayoutParams => "android.widget.LinearLayout$LayoutParams",
    type_map {
        ViewGroupLayoutParams => "android.view.ViewGroup$LayoutParams",
    },
    constructors {
        fn new(width: jint, height: jint),
        fn with_weight(width: jint, height: jint, weight: jfloat),
    },
    is_instance_of = {
        base = ViewGroupLayoutParams,
    }
}

jni::bind_java_type! {
    TabLayout => com.google.android.material.tabs.TabLayout,
    type_map {
        AView => android.view.View,
        Context => android.content.Context,
        TabLayoutTab => "com.google.android.material.tabs.TabLayout$Tab",
    },
    constructors {
        fn new(&Context),
    },
    methods {
        fn get_selected_tab_position() -> jint,
        fn get_tab_at(index: jint) -> TabLayoutTab,
        fn select_tab(tab: &TabLayoutTab),
    },
    is_instance_of = {
        view = AView,
    }
}

jni::bind_java_type! {
    TabLayoutTab => "com.google.android.material.tabs.TabLayout$Tab",
    methods {
        fn set_text(text: &JCharSequence) -> TabLayoutTab,
    },
}

jni::bind_java_type! {
    TabLayoutMediator => com.google.android.material.tabs.TabLayoutMediator,
    type_map {
        TabLayout => com.google.android.material.tabs.TabLayout,
        ViewPager2 => androidx.viewpager2.widget.ViewPager2,
        TabLayoutMediatorTabConfigurationStrategy => "com.google.android.material.tabs.TabLayoutMediator$TabConfigurationStrategy",
    },
    constructors {
        fn new(&TabLayout, &ViewPager2, &TabLayoutMediatorTabConfigurationStrategy),
    },
    methods {
        fn attach(),
    }
}

jni::bind_java_type! {
    TabLayoutMediatorTabConfigurationStrategy => "com.google.android.material.tabs.TabLayoutMediator$TabConfigurationStrategy",
}

jni::bind_java_type! {
    ViewPager2 => androidx.viewpager2.widget.ViewPager2,
    type_map {
        AView => android.view.View,
        Context => android.content.Context,
        RecyclerViewAdapter => "androidx.recyclerview.widget.RecyclerView$Adapter",
    },
    constructors {
        fn new(&Context),
    },
    methods {
        fn set_adapter(adapter: &RecyclerViewAdapter),
    },
    is_instance_of = {
        view = AView,
    }
}

jni::bind_java_type! {
    RecyclerViewAdapter => "androidx.recyclerview.widget.RecyclerView$Adapter",
}

jni::bind_java_type! {
    WinioTabViewAdapter => rs.compio.winio.TabViewAdapter,
    type_map {
        RecyclerViewAdapter => "androidx.recyclerview.widget.RecyclerView$Adapter",
    },
    constructors {
        fn new(),
    },
    methods {
        fn get_pages() -> JList,

        fn notify_item_inserted(position: jint),
        fn notify_item_removed(position: jint),
        fn notify_item_range_removed(start: jint, count: jint),
    },
    is_instance_of = {
        base = RecyclerViewAdapter,
    }
}

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
                [jni::jni_str!(
                    "com.google.android.material.tabs.TabLayout$OnTabSelectedListener"
                )],
                {
                    let on_select = on_select.clone();
                    move |_env, _method, _args| {
                        on_select.signal(());
                        Ok(JObject::null())
                    }
                },
            )?;
            env.call_method(
                &tab,
                jni::jni_str!("addOnTabSelectedListener"),
                jni::jni_sig!(
                    "(Lcom/google/android/material/tabs/TabLayout$OnTabSelectedListener;)V"
                ),
                &[select_proxy.as_ref().into()],
            )?
            .v()?;

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

            let listener = env.new_local_ref(tab_proxy.as_ref())?;
            let listener = unsafe {
                TabLayoutMediatorTabConfigurationStrategy::from_raw(env, listener.into_raw())
            };
            let mediator = TabLayoutMediator::new(env, &tab, &pager, listener)?;
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
}

#[inherit_methods(from = "self.handle")]
impl TabViewItem {
    pub fn new() -> Result<Self> {
        let handle = View::new_standalone()?;
        Ok(Self {
            handle,
            text: String::new(),
        })
    }

    pub fn text(&self) -> Result<String> {
        Ok(self.text.clone())
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.text = s.as_ref().into();
        Ok(())
    }

    pub fn size(&self) -> Result<Size>;
}

winio_handle::impl_as_container!(TabViewItem, handle);
