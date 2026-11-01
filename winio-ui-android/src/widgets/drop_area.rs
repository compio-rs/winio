use std::sync::Arc;

use inherit_methods_macro::inherit_methods;
use jni::refs::{LoaderContext, Reference};
use jni_min_helper::{DynamicProxy, JBoolean};
use winio_callback::SyncCallback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{
    BaseWidget, Result, current_activity,
    java::android::{
        view::{DragEvent, View as AView, ViewOnDragListener, drag_action},
        widget::FrameLayout,
    },
    platform::Data,
    runtime::AndroidRuntime,
    vm_exec,
};

/// A container that receives dragged data.
#[derive(Debug)]
pub struct DropArea {
    inner: BaseWidget<FrameLayout<'static>>,
    on_drop: Arc<SyncCallback<Data>>,
    #[allow(dead_code)]
    drag_proxy: DynamicProxy,
}

#[inherit_methods(from = "self.inner")]
impl DropArea {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let on_drop = Arc::new(SyncCallback::new());
        vm_exec(|env| {
            let act = current_activity(env)?;
            let widget = FrameLayout::new(env, &act)?;

            let drag_proxy = DynamicProxy::build(
                env,
                &LoaderContext::None,
                [ViewOnDragListener::class_name()],
                {
                    let on_drop = on_drop.clone();
                    move |env, _method, args| {
                        let event = args.get_element(env, 1)?;
                        let event = unsafe { DragEvent::from_raw(env, event.into_raw()) };
                        if event.get_action(env)? == drag_action::ACTION_DROP {
                            let clip = event.get_clip_data(env)?;
                            if let Ok(Some(data)) = Data::from_clip_data(env, &clip) {
                                on_drop.signal::<AndroidRuntime>(data);
                            }
                        }
                        Ok(JBoolean::new(env, true)?.into())
                    }
                },
            )?;
            AsRef::<AView>::as_ref(&widget).set_on_drag_listener(env, &drag_proxy)?;

            let inner = BaseWidget::new_with_env(env, parent.as_container(), widget)?;
            Ok(Self {
                inner,
                on_drop,
                drag_proxy,
            })
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, visible: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, size: Size) -> Result<()>;

    pub async fn wait_drop(&self) -> Data {
        self.on_drop.wait().await
    }
}

winio_handle::impl_as_widget!(DropArea, inner);
winio_handle::impl_as_container!(DropArea, inner);
