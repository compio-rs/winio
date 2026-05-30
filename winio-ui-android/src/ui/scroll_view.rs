use inherit_methods_macro::inherit_methods;
use jni::refs::Global;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{AView, BaseWidget, Context, FrameLayout, Result, current_activity, vm_exec};

jni::bind_java_type! {
    AScrollView => android.widget.ScrollView,
    type_map {
        AView => android.view.View,
        Context => android.content.Context,
        FrameLayout => android.widget.FrameLayout,
    },
    constructors {
        fn new(&Context),
    },
    is_instance_of = {
        view = AView,
        frame_layout = FrameLayout,
    }
}

jni::bind_java_type! {
    AHorizontalScrollView => android.widget.HorizontalScrollView,
    type_map {
        AView => android.view.View,
        Context => android.content.Context,
        FrameLayout => android.widget.FrameLayout,
    },
    constructors {
        fn new(&Context),
    },
    is_instance_of = {
        view = AView,
        frame_layout = FrameLayout,
    }
}

#[derive(Debug)]
pub struct ScrollView {
    parent: Global<FrameLayout<'static>>,
    vertical: BaseWidget<AScrollView<'static>>,
    horizontal: BaseWidget<AHorizontalScrollView<'static>>,
    inner_view: BaseWidget<FrameLayout<'static>>,
    enable_vertical: bool,
    enable_horizontal: bool,
}

#[inherit_methods(from = "self.vertical")]
impl ScrollView {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        vm_exec(|env| {
            let context = current_activity(env)?;
            let vertical = AScrollView::new(env, &context)?;
            let horizontal = AHorizontalScrollView::new(env, &context)?;
            let inner_view = FrameLayout::new(env, &context)?;
            vertical.as_frame_layout().add_view(env, &horizontal)?;
            horizontal.as_frame_layout().add_view(env, &inner_view)?;
            let p = parent.as_container();
            let parent = env.new_local_ref(p.to_android())?;
            let parent =
                env.new_global_ref(unsafe { FrameLayout::from_raw(env, parent.into_raw()) })?;
            let vertical = BaseWidget::new_with_env(env, p, vertical)?;
            let horizontal = env.new_global_ref(horizontal)?.into();
            let inner_view = env.new_global_ref(inner_view)?.into();
            Ok(Self {
                parent,
                vertical,
                horizontal,
                inner_view,
                enable_vertical: true,
                enable_horizontal: true,
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

    fn detach(&mut self) -> Result<()> {
        vm_exec(|env| {
            if self.enable_vertical {
                self.parent.remove_view(env, self.vertical.as_view())?;
                if self.enable_horizontal {
                    // V > H > I
                    self.vertical
                        .as_frame_layout()
                        .remove_view(env, self.horizontal.as_view())?;
                    self.horizontal
                        .as_frame_layout()
                        .remove_view(env, self.inner_view.as_view())?;
                } else {
                    // V > I
                    self.vertical
                        .as_frame_layout()
                        .remove_view(env, self.inner_view.as_view())?;
                }
            } else {
                if self.enable_horizontal {
                    // H > I
                    self.parent.remove_view(env, self.horizontal.as_view())?;
                    self.horizontal
                        .as_frame_layout()
                        .remove_view(env, self.inner_view.as_view())?;
                } else {
                    // I
                    self.parent.remove_view(env, self.inner_view.as_view())?;
                }
            }
            Ok(())
        })
    }

    fn attach(&mut self) -> Result<()> {
        vm_exec(|env| {
            if self.enable_vertical {
                self.parent.add_view(env, self.vertical.as_view())?;
                if self.enable_horizontal {
                    // V > H > I
                    self.vertical
                        .as_frame_layout()
                        .add_view(env, self.horizontal.as_view())?;
                    self.horizontal
                        .as_frame_layout()
                        .add_view(env, self.inner_view.as_view())?;
                } else {
                    // V > I
                    self.vertical
                        .as_frame_layout()
                        .add_view(env, self.inner_view.as_view())?;
                }
            } else {
                if self.enable_horizontal {
                    // H > I
                    self.parent.add_view(env, self.horizontal.as_view())?;
                    self.horizontal
                        .as_frame_layout()
                        .add_view(env, self.inner_view.as_view())?;
                } else {
                    // I
                    self.parent.add_view(env, self.inner_view.as_view())?;
                }
            }
            Ok(())
        })
    }

    pub fn hscroll(&self) -> Result<bool> {
        Ok(self.enable_horizontal)
    }

    pub fn set_hscroll(&mut self, v: bool) -> Result<()> {
        if self.enable_horizontal != v {
            self.detach()?;
            self.enable_horizontal = v;
            self.attach()?;
        }
        Ok(())
    }

    pub fn vscroll(&self) -> Result<bool> {
        Ok(self.enable_vertical)
    }

    pub fn set_vscroll(&mut self, v: bool) -> Result<()> {
        if self.enable_vertical != v {
            self.detach()?;
            self.enable_vertical = v;
            self.attach()?;
        }
        Ok(())
    }

    pub async fn start(&self) -> ! {
        std::future::pending().await
    }
}

winio_handle::impl_as_widget!(ScrollView, vertical);
winio_handle::impl_as_container!(ScrollView, inner_view);
