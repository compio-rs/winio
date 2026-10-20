use inherit_methods_macro::inherit_methods;
use jni::{Env, refs::Global};
use winio_handle::{AsContainer, BorrowedContainer};
use winio_primitive::{Point, Size};

use crate::{
    BaseWidget, JCharSequenceExt, Result, current_activity,
    java::{
        android::{
            view::{View as AView, gravity},
            widget::{FrameLayout, FrameLayoutLayoutParams, TextView as ATextView},
        },
        material::MaterialCardView,
    },
    vm_exec,
};

const TITLE_TEXT_SIZE: f32 = 18.0;
const PADDING_DP: f32 = 16.0;
const TITLE_GAP_DP: f32 = 8.0;
const TITLE_PADDING_DP: f32 = 4.0;

#[derive(Debug)]
pub struct GroupBox {
    handle: BaseWidget<MaterialCardView<'static>>,
    title: Global<ATextView<'static>>,
    content: Global<FrameLayout<'static>>,
    density: f32,
    padding: i32,
    title_gap: i32,
    title_height: i32,
}

#[inherit_methods(from = "self.handle")]
impl GroupBox {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        vm_exec(|env| {
            let act = current_activity(env)?;
            let resources = act.as_context().get_resources(env)?;
            let metrics = resources.get_display_metrics(env)?;
            let density = metrics.density(env)?;
            let padding = (PADDING_DP * density) as i32;
            let title_gap = (TITLE_GAP_DP * density) as i32;

            let card = MaterialCardView::new(env, &act)?;
            card.set_content_padding(env, 0, 0, 0, 0)?;
            card.set_card_elevation(env, 0.0)?;
            let handle = BaseWidget::new_with_env(env, parent.as_container(), card)?;

            let title = ATextView::new(env, &act)?;
            title.set_text_size(env, TITLE_TEXT_SIZE)?;
            title.set_single_line(env, true)?;
            title.set_include_font_padding(env, false)?;
            title.set_gravity(env, gravity::CENTER_VERTICAL | gravity::LEFT)?;
            let title_height = title.get_line_height(env)? + (TITLE_PADDING_DP * density) as i32;

            let content = FrameLayout::new(env, &act)?;
            content.as_view_group().set_clip_children(env, true)?;

            handle.as_view_group().add_view(env, content.as_view())?;
            handle.as_view_group().add_view(env, title.as_view())?;

            let title = env.new_global_ref(title)?;
            let content = env.new_global_ref(content)?;

            let this = Self {
                handle,
                title,
                content,
                density,
                padding,
                title_gap,
                title_height,
            };
            this.update_layout(env, 0, 0)?;
            Ok(this)
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, visible: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, size: Size) -> Result<()> {
        self.handle.set_size(size)?;
        let width = (size.width * self.density as f64) as i32;
        let height = (size.height * self.density as f64) as i32;
        vm_exec(move |env| self.update_layout(env, width, height))
    }

    pub fn preferred_size(&self) -> Result<Size> {
        let width = vm_exec(|env| -> Result<i32> {
            self.title.as_view().measure(env, 0, 0)?;
            Ok(self.title.as_view().get_measured_width(env)?)
        })?;
        Ok(Size::new(
            (width + self.padding * 2) as f64 / self.density as f64,
            (self.title_height + self.padding * 2 + self.title_gap) as f64 / self.density as f64,
        ))
    }

    pub fn client_size(&self) -> Result<Size> {
        let (width, height) = vm_exec(|env| -> Result<(i32, i32)> {
            let params = self.content.as_view().get_layout_params(env)?;
            let params = unsafe { FrameLayoutLayoutParams::from_raw(env, params.into_raw()) };
            let width = params.as_base().width(env)?;
            let height = params.as_base().height(env)?;
            Ok((width, height))
        })?;
        Ok(Size::new(
            width as f64 / self.density as f64,
            height as f64 / self.density as f64,
        ))
    }

    pub fn text(&self) -> Result<String> {
        vm_exec(|env| Ok(self.title.get_text(env)?.try_to_string(env)?))
    }

    pub fn set_text(&mut self, text: impl AsRef<str>) -> Result<()> {
        vm_exec(move |env| {
            let text = env.new_string(&text)?;
            self.title.set_text(env, text)?;
            Ok(())
        })
    }

    fn update_layout(&self, env: &mut Env<'_>, width: i32, height: i32) -> Result<()> {
        let inner_width = (width - self.padding * 2).max(0);
        set_child_layout(
            env,
            &self.title.as_view(),
            self.padding,
            self.padding,
            inner_width,
            self.title_height,
        )?;
        let content_top = self.padding + self.title_height + self.title_gap;
        let content_height = (height - content_top - self.padding).max(0);
        set_child_layout(
            env,
            &self.content.as_view(),
            self.padding,
            content_top,
            inner_width,
            content_height,
        )
    }
}

fn set_child_layout(
    env: &mut Env<'_>,
    view: &AView<'_>,
    left: i32,
    top: i32,
    width: i32,
    height: i32,
) -> Result<()> {
    let params = FrameLayoutLayoutParams::new(env, width, height)?;
    params.as_margin().set_left_margin(env, left)?;
    params.as_margin().set_top_margin(env, top)?;
    params.set_gravity(env, gravity::LEFT | gravity::TOP)?;
    view.set_layout_params(env, params)?;
    Ok(())
}

winio_handle::impl_as_widget!(GroupBox, handle);

impl AsContainer for GroupBox {
    fn as_container(&self) -> BorrowedContainer<'_> {
        unsafe { BorrowedContainer::android(self.content.as_obj()) }
    }
}
