//! Android window widget, based on JNI and FrameLayout

use std::sync::{Arc, Mutex};

use inherit_methods_macro::inherit_methods;
use jni::{
    objects::JObject,
    refs::{Global, LoaderContext, Reference},
};
use jni_min_helper::{DynamicProxy, JInteger};
use winio_callback::SyncCallback;
use winio_handle::{AsWindow, BorrowedContainer, BorrowedWindow};
use winio_primitive::{Margin, Point, Size};

use crate::{
    AView, Activity, BaseWidget, Context, FrameLayoutLayoutParams, OnLayoutChangeListener, Result,
    current_activity, vm_exec,
};

jni::bind_java_type! {
    pub(crate) AViewGroup => android.view.ViewGroup,
    type_map {
        AView => android.view.View,
    },
    methods {
        fn add_view(view: &AView),
        fn remove_view(view: &AView),
    },
    is_instance_of = {
        view = AView,
    }
}

jni::bind_java_type! {
    pub(crate) FrameLayout => android.widget.FrameLayout,
    type_map {
        AView => android.view.View,
        AViewGroup => android.view.ViewGroup,
        Context => android.content.Context,
    },
    constructors {
        fn new(context: &Context),
    },
    is_instance_of = {
        view = AView,
        view_group = AViewGroup,
    }
}

jni::bind_java_type! {
    pub(crate) WindowInsets => android.view.WindowInsets,
    type_map {
        Insets => android.graphics.Insets,
    },
    methods {
        fn get_insets_ignoring_visibility(type_mask: jint) -> Insets,
    }
}

jni::bind_java_type! {
    WindowInsetsType => "android.view.WindowInsets$Type",
    methods {
        static fn system_bars() -> jint,
    }
}

jni::bind_java_type! {
    Insets => android.graphics.Insets,
    fields {
        left: jint,
        top: jint,
        right: jint,
        bottom: jint,
    }
}

#[derive(Debug)]
pub struct Window {
    inner: BaseWidget<FrameLayout<'static>>,
    inner_view: BaseWidget<FrameLayout<'static>>,
    activity: Global<Activity<'static>>,
    on_resize: Arc<SyncCallback>,
    #[allow(unused)]
    on_resize_proxy: DynamicProxy,
    size_update: Arc<Mutex<Size>>,
}

#[inherit_methods(from = "self.inner")]
impl Window {
    pub fn new() -> Result<Self> {
        vm_exec(move |env| {
            let act = current_activity(env)?;
            let act = env.new_global_ref(act)?;
            let window = FrameLayout::new(env, &act)?;
            act.set_content_view(env, &window)?;
            let params = FrameLayoutLayoutParams::new(env, -1, -1)?;
            window.as_view().set_layout_params(env, &params)?;
            let inner = env.new_global_ref(&window)?;
            let inner_view = FrameLayout::new(env, &act)?;
            let inner_view = BaseWidget::new_with_env(
                env,
                unsafe { BorrowedContainer::android(&inner) },
                inner_view,
            )?;

            let on_resize = Arc::new(SyncCallback::new());
            WINDOW_RESIZE_CALLBACK
                .lock()
                .unwrap()
                .replace(on_resize.clone());
            let size_update = Arc::new(Mutex::new(Size::zero()));
            let on_resize_proxy = DynamicProxy::build(
                env,
                &LoaderContext::None,
                [OnLayoutChangeListener::class_name()],
                {
                    let on_resize = on_resize.clone();
                    let size_update = size_update.clone();
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
                                let size = Size::new((right - left) as _, (bottom - top) as _);
                                *size_update.lock().unwrap() = size;
                                on_resize.signal(());
                            }
                        }
                        Ok(JObject::null())
                    }
                },
            )?;
            inner
                .as_view()
                .add_on_layout_change_listener(env, &on_resize_proxy)?;
            Ok(Self {
                inner: inner.into(),
                inner_view,
                activity: act,
                on_resize,
                on_resize_proxy,
                size_update,
            })
        })
    }

    pub fn client_size(&self) -> Result<Size> {
        let size = self.size()?;
        let margin = vm_exec(|env| {
            let insets = self.inner.as_view().get_root_window_insets(env)?;
            if !insets.is_null() {
                let system_bars = WindowInsetsType::system_bars(env)?;
                let insets = insets.get_insets_ignoring_visibility(env, system_bars)?;
                let left = insets.left(env)?;
                let top = insets.top(env)?;
                let right = insets.right(env)?;
                let bottom = insets.bottom(env)?;
                Result::Ok(Margin::new(top as _, right as _, bottom as _, left as _))
            } else {
                Ok(Margin::zero())
            }
        })?;
        let size = Size::new(
            size.width - margin.horizontal(),
            size.height - margin.vertical(),
        );
        self.inner_view
            .set_loc(Point::new(margin.left, margin.top))?;
        self.inner_view.set_size(size)?;
        Ok(size)
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, visible: bool) -> Result<()>;

    fn size_update(&self) -> Size {
        *self.size_update.lock().unwrap()
    }

    pub fn loc(&self) -> Result<Point> {
        Ok(Point::zero())
    }

    pub fn set_loc(&mut self, _p: Point) -> Result<()> {
        Ok(())
    }

    pub fn size(&self) -> Result<Size> {
        let size = self.size_update();
        if size == Size::zero() {
            self.inner.preferred_size()
        } else {
            Ok(size)
        }
    }

    pub fn set_size(&mut self, _size: Size) -> Result<()> {
        Ok(())
    }

    pub fn text(&self) -> Result<String> {
        Ok(String::new())
    }

    pub fn set_text(&mut self, _text: impl AsRef<str>) -> Result<()> {
        Ok(())
    }

    pub async fn wait_close(&self) {
        std::future::pending().await
    }

    pub async fn wait_move(&self) {
        std::future::pending().await
    }

    pub async fn wait_size(&self) {
        self.on_resize.wait().await
    }

    pub async fn wait_theme_changed(&self) {
        std::future::pending().await
    }
}

impl AsWindow for Window {
    fn as_window(&self) -> BorrowedWindow<'_> {
        unsafe { BorrowedWindow::android(&self.activity) }
    }
}

winio_handle::impl_as_container!(Window, inner_view);

impl Drop for Window {
    fn drop(&mut self) {
        WINDOW_RESIZE_CALLBACK.lock().unwrap().take();
    }
}

pub(crate) static WINDOW_RESIZE_CALLBACK: Mutex<Option<Arc<SyncCallback>>> = Mutex::new(None);
