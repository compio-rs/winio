//! Android window widget, based on JNI and FrameLayout

use std::sync::{Arc, Mutex};

use futures_util::FutureExt;
use inherit_methods_macro::inherit_methods;
use jni::{Env, objects::JObject, refs::LoaderContext, strings::JNIString};
use jni_min_helper::{DynamicProxy, JInteger};
use winio_callback::SyncCallback;
use winio_handle::{AsWindow, BorrowedContainer, BorrowedWindow};
use winio_primitive::{Margin, Point, Size};

use crate::{BaseWidget, DESTROY_SLAB, GlobalRef, Result, current_activity, vm_exec};

#[derive(Debug)]
pub struct Window {
    inner: BaseWidget,
    inner_view: BaseWidget,
    activity: GlobalRef,
    on_resize: Arc<SyncCallback>,
    #[allow(unused)]
    on_resize_proxy: DynamicProxy,
    on_insets: Arc<SyncCallback>,
    #[allow(unused)]
    insets_proxy: DynamicProxy,
    on_destroy: Arc<SyncCallback>,
    destroy_index: usize,
    size_update: Arc<Mutex<Size>>,
    margin_update: Arc<Mutex<Margin>>,
}

#[inherit_methods(from = "self.inner")]
impl Window {
    const WINDOW_CLASS: &'static str = "android/widget/FrameLayout";

    pub fn new() -> Result<Self> {
        vm_exec(move |env| {
            let act = current_activity()?;
            let act = env.new_global_ref(act.as_obj())?;
            let window = env.new_object(
                JNIString::new(Self::WINDOW_CLASS),
                jni::jni_sig!("(Landroid/content/Context;)V"),
                &[act.as_obj().into()],
            )?;
            env.call_method(
                act.as_obj(),
                jni::jni_str!("setContentView"),
                jni::jni_sig!("(Landroid/view/View;)V"),
                &[(&window).into()],
            )?
            .v()?;
            env.call_method(
                &window,
                jni::jni_str!("setFitsSystemWindows"),
                jni::jni_sig!("(Z)V"),
                &[true.into()],
            )?
            .v()?;
            let params = env.new_object(
                jni::jni_str!("android/widget/FrameLayout$LayoutParams"),
                jni::jni_sig!("(II)V"),
                &[(-1i32).into(), (-1i32).into()],
            )?;
            env.call_method(
                &window,
                jni::jni_str!("setLayoutParams"),
                jni::jni_sig!("(Landroid/view/ViewGroup$LayoutParams;)V"),
                &[(&params).into()],
            )?
            .v()?;
            let inner = env.new_global_ref(&window)?;
            let inner_view = BaseWidget::new_with_env(
                env,
                unsafe { BorrowedContainer::android(&inner) },
                Self::WINDOW_CLASS,
            )?;

            let on_resize = Arc::new(SyncCallback::new());
            let size_update = Arc::new(Mutex::new(Size::zero()));
            let on_insets = Arc::new(SyncCallback::new());
            let margin_update = Arc::new(Mutex::new(Margin::zero()));
            let insets_proxy = set_insets_listener(
                env,
                act.as_obj(),
                &window,
                on_insets.clone(),
                margin_update.clone(),
            )?;
            let on_destroy = Arc::new(SyncCallback::new());
            let destroy_index = DESTROY_SLAB.lock().unwrap().insert(on_destroy.clone());
            let on_resize_proxy = {
                let on_resize = on_resize.clone();
                let size_update = size_update.clone();
                let proxy = DynamicProxy::build(
                    env,
                    &LoaderContext::None,
                    [jni::jni_str!("android/view/View$OnLayoutChangeListener")],
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
                    },
                )?;
                env.call_method(
                    inner.as_obj(),
                    jni::jni_str!("addOnLayoutChangeListener"),
                    jni::jni_sig!("(Landroid/view/View$OnLayoutChangeListener;)V"),
                    &[proxy.as_ref().into()],
                )?
                .v()?;
                proxy
            };
            Ok(Self {
                inner: inner.into(),
                inner_view,
                activity: act,
                on_resize,
                on_resize_proxy,
                on_insets,
                insets_proxy,
                on_destroy,
                destroy_index,
                size_update,
                margin_update,
            })
        })
    }

    pub fn client_size(&self) -> Result<Size> {
        let size = self.size()?;
        let margin = self.margin_update();
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

    fn margin_update(&self) -> Margin {
        *self.margin_update.lock().unwrap()
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
        self.on_destroy.wait().await;
    }

    pub async fn wait_move(&self) {
        std::future::pending().await
    }

    pub async fn wait_size(&self) {
        futures_util::select! {
            _ = self.on_resize.wait().fuse() => {},
            _ = self.on_insets.wait().fuse() => {},
        }
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
        DESTROY_SLAB.lock().unwrap().remove(self.destroy_index);
    }
}

jni::bind_java_type! {
    WindowInsetsCompat => androidx.core.view.WindowInsetsCompat,
    type_map {
        Insets => androidx.core.graphics.Insets,
    },
    methods {
        fn get_insets(type_mask: jint) -> Insets,
    }
}

jni::bind_java_type! {
    WindowInsetsCompatType => "androidx.core.view.WindowInsetsCompat$Type",
    methods {
        static fn system_bars() -> jint,
    }
}

jni::bind_java_type! {
    Insets => androidx.core.graphics.Insets,
    fields {
        left: jint,
        top: jint,
        right: jint,
        bottom: jint,
    }
}

fn set_insets_listener(
    env: &mut Env,
    activity: &JObject,
    view: &JObject,
    on_resize: Arc<SyncCallback>,
    margin_update: Arc<Mutex<Margin>>,
) -> Result<DynamicProxy> {
    let proxy = DynamicProxy::build(
        env,
        &LoaderContext::FromObject(activity),
        [jni::jni_str!(
            "androidx/core/view/OnApplyWindowInsetsListener"
        )],
        move |env, _method, args| {
            let insets_compat = args.get_element(env, 1)?;
            let insets_compat =
                unsafe { WindowInsetsCompat::from_raw(env, insets_compat.into_raw()) };
            let insets =
                insets_compat.get_insets(env, WindowInsetsCompatType::system_bars(env)?)?;

            let (left, top, right, bottom) = (
                insets.left(env)?,
                insets.top(env)?,
                insets.right(env)?,
                insets.bottom(env)?,
            );
            let margin = Margin::new(top as _, right as _, bottom as _, left as _);
            *margin_update.lock().unwrap() = margin;
            on_resize.signal(());
            Ok(insets_compat.into())
        },
    )?;
    env.call_static_method(
        jni::jni_str!("androidx/core/view/ViewCompat"),
        jni::jni_str!("setOnApplyWindowInsetsListener"),
        jni::jni_sig!("(Landroid/view/View;Landroidx/core/view/OnApplyWindowInsetsListener;)V"),
        &[view.into(), proxy.as_ref().into()],
    )?;
    Ok(proxy)
}
