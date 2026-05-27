use std::marker::PhantomData;

use image::DynamicImage;
use inherit_methods_macro::inherit_methods;
use jni::{Env, errors::Result as JniResult};
use winio_handle::{AsContainer, impl_as_widget};
use winio_primitive::{
    BrushPen, DrawingFont, LinearGradientBrush, MouseButton, Point, RadialGradientBrush, Rect,
    Size, SolidColorBrush, Transform, Vector,
};

use crate::{BaseWidget, GlobalRef, Result};

/// Drawing brush.
pub trait Brush {
    fn get_raw(&self, env: &mut Env<'_>) -> JniResult<GlobalRef>;
}

impl<B: Brush> Brush for &'_ B {
    fn get_raw(&self, env: &mut Env<'_>) -> JniResult<GlobalRef> {
        B::get_raw(self, env)
    }
}

impl Brush for LinearGradientBrush {
    fn get_raw(&self, _env: &mut Env<'_>) -> JniResult<GlobalRef> {
        todo!()
    }
}

impl Brush for RadialGradientBrush {
    fn get_raw(&self, _env: &mut Env<'_>) -> JniResult<GlobalRef> {
        todo!()
    }
}

impl Brush for SolidColorBrush {
    fn get_raw(&self, _env: &mut Env<'_>) -> JniResult<GlobalRef> {
        todo!()
    }
}

/// Drawing pen.
pub trait Pen {}

impl<P: Pen> Pen for &'_ P {}

impl<B: Brush> Pen for BrushPen<B> {}

#[derive(Debug, Clone)]
pub struct DrawingImage;

impl DrawingImage {
    pub fn size(&self) -> Result<Size> {
        todo!()
    }
}

pub struct DrawingContext<'a> {
    _a: PhantomData<&'a ()>,
}

impl<'a> DrawingContext<'a> {
    pub fn close(self) -> Result<()> {
        todo!()
    }

    pub fn set_transform(&mut self, _transform: Transform) -> Result<()> {
        todo!()
    }

    pub fn transform(&self) -> Result<Transform> {
        todo!()
    }

    pub fn draw_path(&mut self, _pen: impl Pen, _path: &DrawingPath) -> Result<()> {
        todo!()
    }

    pub fn fill_path(&mut self, _brush: impl Brush, _path: &DrawingPath) -> Result<()> {
        todo!()
    }

    pub fn draw_arc(&mut self, _pen: impl Pen, _rect: Rect, _start: f64, _end: f64) -> Result<()> {
        todo!()
    }

    pub fn draw_pie(&mut self, _pen: impl Pen, _rect: Rect, _start: f64, _end: f64) -> Result<()> {
        todo!()
    }

    pub fn fill_pie(
        &mut self,
        _brush: impl Brush,
        _rect: Rect,
        _start: f64,
        _end: f64,
    ) -> Result<()> {
        todo!()
    }

    pub fn draw_ellipse(&mut self, _pen: impl Pen, _rect: Rect) -> Result<()> {
        todo!()
    }

    pub fn fill_ellipse(&mut self, _brush: impl Brush, _rect: Rect) -> Result<()> {
        todo!()
    }

    pub fn draw_line(&mut self, _pen: impl Pen, _start: Point, _end: Point) -> Result<()> {
        todo!()
    }

    pub fn draw_rect(&mut self, _pen: impl Pen, _rect: Rect) -> Result<()> {
        todo!()
    }

    pub fn fill_rect(&mut self, _brush: impl Brush, _rect: Rect) -> Result<()> {
        todo!()
    }

    pub fn draw_round_rect(&mut self, _pen: impl Pen, _rect: Rect, _round: Size) -> Result<()> {
        todo!()
    }

    pub fn fill_round_rect(&mut self, _brush: impl Brush, _rect: Rect, _round: Size) -> Result<()> {
        todo!()
    }

    pub fn draw_str(
        &mut self,
        _brush: impl Brush,
        _font: DrawingFont,
        _pos: Point,
        _text: &str,
    ) -> Result<()> {
        todo!()
    }

    pub fn measure_str(&self, _font: DrawingFont, _text: &str) -> Result<Size> {
        todo!()
    }

    pub fn create_image(&self, _image: DynamicImage) -> Result<DrawingImage> {
        todo!()
    }

    pub fn draw_image(
        &mut self,
        _image: &DrawingImage,
        _rect: Rect,
        _clip: Option<Rect>,
    ) -> Result<()> {
        todo!()
    }

    pub fn create_path_builder(&self, _start: Point) -> Result<DrawingPathBuilder> {
        todo!()
    }
}

pub struct DrawingPath;

pub struct DrawingPathBuilder;

impl DrawingPathBuilder {
    pub fn add_line(&mut self, _p: Point) -> Result<()> {
        todo!()
    }

    pub fn add_arc(
        &mut self,
        _center: Point,
        _radius: Size,
        _start: f64,
        _end: f64,
        _clockwise: bool,
    ) -> Result<()> {
        todo!()
    }

    pub fn add_bezier(&mut self, _p1: Point, _p2: Point, _p3: Point) -> Result<()> {
        todo!()
    }

    pub fn build(self, _close: bool) -> Result<DrawingPath> {
        todo!()
    }
}

#[derive(Debug)]
pub struct Canvas {
    inner: BaseWidget,
}

// noinspection SpellCheckingInspection
#[inherit_methods(from = "self.inner")]
impl Canvas {
    const WIDGET_CLASS: &'static str = "rs/compio/winio/Canvas";

    pub fn new(parent: impl AsContainer) -> Result<Self> {
        Ok(Self {
            inner: BaseWidget::new(parent.as_container(), Self::WIDGET_CLASS)?,
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

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn context(&self) -> Result<DrawingContext<'_>> {
        todo!()
    }

    pub async fn wait_mouse_move(&self) -> Point {
        todo!()
    }

    pub async fn wait_mouse_down(&self) -> MouseButton {
        todo!()
    }

    pub async fn wait_mouse_up(&self) -> MouseButton {
        todo!()
    }

    pub async fn wait_mouse_wheel(&self) -> Vector {
        todo!()
    }
}

impl_as_widget!(Canvas, inner);
