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
use winio_primitive::{Point, Size};

use crate::{
    BaseWidget, Error, Result, current_activity,
    java::{
        android::{
            view::{
                MeasureSpec, View as AView, ViewOnLayoutChangeListener, WindowInsetsType, gravity,
                measure_spec, visibility,
            },
            widget::{FrameLayout, FrameLayoutLayoutParams},
        },
        custom::Activity,
    },
    platform::dpi::{logical_margin, logical_size, physical_point, physical_size},
    runtime::AndroidRuntime,
    vm_exec,
};

pub(crate) const MENU_BAR_VIEW_ID: i32 = 0x7769_6e69;
pub(crate) const TOOL_BAR_VIEW_ID: i32 = 0x7769_6e6a;

#[derive(Debug)]
pub struct Window {
    inner: BaseWidget<FrameLayout<'static>>,
    inner_view: BaseWidget<FrameLayout<'static>>,
    menu_bar_view: Global<FrameLayout<'static>>,
    tool_bar_view: Global<FrameLayout<'static>>,
    activity: Global<Activity<'static>>,
    on_resize: Arc<SyncCallback>,
    #[allow(unused)]
    on_resize_proxy: DynamicProxy,
    size_update: Arc<Mutex<(i32, i32)>>,
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

            let menu_bar_view = FrameLayout::new(env, &act)?;
            menu_bar_view.as_view().set_id(env, MENU_BAR_VIEW_ID)?;
            let params = FrameLayoutLayoutParams::new(env, -1, -2)?;
            params.set_gravity(env, gravity::LEFT | gravity::TOP)?;
            menu_bar_view.as_view().set_layout_params(env, &params)?;
            window
                .as_view_group()
                .add_view(env, menu_bar_view.as_view())?;
            let menu_bar_view = env.new_global_ref(&menu_bar_view)?;

            let tool_bar_view = FrameLayout::new(env, &act)?;
            tool_bar_view.as_view().set_id(env, TOOL_BAR_VIEW_ID)?;
            let params = FrameLayoutLayoutParams::new(env, -1, -2)?;
            params.set_gravity(env, gravity::LEFT | gravity::TOP)?;
            tool_bar_view.as_view().set_layout_params(env, &params)?;
            window
                .as_view_group()
                .add_view(env, tool_bar_view.as_view())?;
            let tool_bar_view = env.new_global_ref(&tool_bar_view)?;

            let on_resize = Arc::new(SyncCallback::new());
            WINDOW_RESIZE_CALLBACK
                .lock()
                .unwrap()
                .replace(on_resize.clone());
            let size_update = Arc::new(Mutex::new((0, 0)));
            let on_resize_proxy = DynamicProxy::build(
                env,
                &LoaderContext::None,
                [ViewOnLayoutChangeListener::class_name()],
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
                                *size_update.lock().unwrap() = (right - left, bottom - top);
                                on_resize.signal::<AndroidRuntime>(());
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
                menu_bar_view,
                tool_bar_view,
                activity: act,
                on_resize,
                on_resize_proxy,
                size_update,
            })
        })
    }

    pub fn client_size(&self) -> Result<Size> {
        let size = self.size()?;
        let (top, right, bottom, left) = vm_exec(|env| {
            let insets = self.inner.as_view().get_root_window_insets(env)?;
            if !insets.is_null() {
                let system_bars = WindowInsetsType::system_bars(env)?;
                let insets = insets.get_insets_ignoring_visibility(env, system_bars)?;
                let left = insets.left(env)?;
                let top = insets.top(env)?;
                let right = insets.right(env)?;
                let bottom = insets.bottom(env)?;
                Result::Ok((top as _, right as _, bottom as _, left as _))
            } else {
                Ok((0, 0, 0, 0))
            }
        })?;
        let margin = logical_margin(top, right, bottom, left)?;
        let size = Size::new(
            size.width - margin.horizontal(),
            size.height - margin.vertical(),
        );

        let menu_bar_size = Size::new(size.width, self.menu_bar_height(size.width)?);
        let view: &AView<'static> = self.menu_bar_view.as_ref();
        set_bounds(view, Point::new(margin.left, margin.top), menu_bar_size)?;

        let tool_bar_size = Size::new(size.width, self.tool_bar_height(size.width)?);
        let view: &AView<'static> = self.tool_bar_view.as_ref();
        set_bounds(
            view,
            Point::new(margin.left, margin.top + menu_bar_size.height),
            tool_bar_size,
        )?;

        let content_top = margin.top + menu_bar_size.height + tool_bar_size.height;
        self.inner_view
            .set_loc(Point::new(margin.left, content_top))?;
        let size = Size::new(
            size.width,
            size.height - menu_bar_size.height - tool_bar_size.height,
        );
        self.inner_view.set_size(size)?;
        Ok(size)
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, visible: bool) -> Result<()>;

    fn size_update(&self) -> Result<Size> {
        let (width, height) = *self.size_update.lock().unwrap();
        logical_size(width as _, height as _)
    }

    pub fn loc(&self) -> Result<Point> {
        Ok(Point::zero())
    }

    pub fn set_loc(&mut self, _p: Point) -> Result<()> {
        Ok(())
    }

    pub fn size(&self) -> Result<Size> {
        let size = self.size_update()?;
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

pub(crate) fn notify_layout_changed() {
    if let Some(callback) = WINDOW_RESIZE_CALLBACK.lock().unwrap().as_ref() {
        callback.signal::<AndroidRuntime>(());
    }
}

fn set_bounds(view: &AView<'static>, loc: Point, size: Size) -> Result<()> {
    let (x, y) = physical_point(loc)?;
    let (width, height) = physical_size(size)?;
    vm_exec(move |env| {
        let params = FrameLayoutLayoutParams::new(env, width as i32, height as i32)?;
        params.set_gravity(env, gravity::LEFT | gravity::TOP)?;
        params.as_margin().set_left_margin(env, x as i32)?;
        params.as_margin().set_top_margin(env, y as i32)?;
        view.set_layout_params(env, params)?;
        Ok(())
    })
}

impl Window {
    fn menu_bar_height(&self, width: f64) -> Result<f64> {
        self.bar_height(self.menu_bar_view.as_ref(), width)
    }

    fn tool_bar_height(&self, width: f64) -> Result<f64> {
        self.bar_height(self.tool_bar_view.as_ref(), width)
    }

    fn bar_height(&self, view: &AView<'static>, width: f64) -> Result<f64> {
        let (width, _) = physical_size(Size::new(width, 0.0))?;
        let height = vm_exec(move |env| {
            if view.get_visibility(env)? != visibility::VISIBLE {
                return Ok::<_, Error>(0);
            }
            let width_spec =
                MeasureSpec::make_measure_spec(env, width as i32, measure_spec::EXACTLY)?;
            let height_spec = MeasureSpec::make_measure_spec(env, 0, measure_spec::UNSPECIFIED)?;
            view.measure(env, width_spec, height_spec)?;
            Ok(view.get_measured_height(env)?)
        })?;
        Ok(logical_size(0.0, height as f32)?.height)
    }
}
