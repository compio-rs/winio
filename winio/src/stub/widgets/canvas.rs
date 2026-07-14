use image::DynamicImage;
use inherit_methods_macro::inherit_methods;
use winio_handle::AsContainer;
use winio_primitive::{
    BrushPen, DrawingFont, LinearGradientBrush, MouseButton, Point, RadialGradientBrush, Rect,
    Size, SolidColorBrush, Transform, Vector,
};

use crate::stub::{Result, Widget, not_impl};

#[derive(Debug)]
pub struct Canvas {
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl Canvas {
    pub fn new(_parent: impl AsContainer) -> Result<Self> {
        not_impl()
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

    pub fn context(&mut self) -> Result<DrawingContext<'_>> {
        not_impl()
    }

    pub async fn wait_mouse_move(&self) -> Point {
        not_impl()
    }

    pub async fn wait_mouse_down(&self) -> MouseButton {
        not_impl()
    }

    pub async fn wait_mouse_up(&self) -> MouseButton {
        not_impl()
    }

    pub async fn wait_mouse_wheel(&self) -> Vector {
        not_impl()
    }
}

winio_handle::impl_as_widget!(Canvas, handle);

pub trait Brush {}

impl<B: Brush> Brush for &B {}

impl Brush for SolidColorBrush {}

impl Brush for LinearGradientBrush {}

impl Brush for RadialGradientBrush {}

pub trait Pen {}

impl<P: Pen> Pen for &P {}

impl<B: Brush> Pen for BrushPen<B> {}

pub struct DrawingContext<'a>(std::marker::PhantomData<&'a ()>);

impl DrawingContext<'_> {
    pub fn close(self) -> Result<()> {
        not_impl()
    }

    pub fn set_transform(&mut self, _transform: Transform) -> Result<()> {
        not_impl()
    }

    pub fn transform(&self) -> Result<Transform> {
        not_impl()
    }

    pub fn draw_path(&mut self, _pen: impl Pen, _path: &DrawingPath) -> Result<()> {
        not_impl()
    }

    pub fn fill_path(&mut self, _brush: impl Brush, _path: &DrawingPath) -> Result<()> {
        not_impl()
    }

    pub fn draw_arc(&mut self, _pen: impl Pen, _rect: Rect, _start: f64, _end: f64) -> Result<()> {
        not_impl()
    }

    pub fn draw_pie(&mut self, _pen: impl Pen, _rect: Rect, _start: f64, _end: f64) -> Result<()> {
        not_impl()
    }

    pub fn fill_pie(
        &mut self,
        _brush: impl Brush,
        _rect: Rect,
        _start: f64,
        _end: f64,
    ) -> Result<()> {
        not_impl()
    }

    pub fn draw_ellipse(&mut self, _pen: impl Pen, _rect: Rect) -> Result<()> {
        not_impl()
    }

    pub fn fill_ellipse(&mut self, _brush: impl Brush, _rect: Rect) -> Result<()> {
        not_impl()
    }

    pub fn draw_line(&mut self, _pen: impl Pen, _start: Point, _end: Point) -> Result<()> {
        not_impl()
    }

    pub fn draw_rect(&mut self, _pen: impl Pen, _rect: Rect) -> Result<()> {
        not_impl()
    }

    pub fn fill_rect(&mut self, _brush: impl Brush, _rect: Rect) -> Result<()> {
        not_impl()
    }

    pub fn draw_round_rect(&mut self, _pen: impl Pen, _rect: Rect, _round: Size) -> Result<()> {
        not_impl()
    }

    pub fn fill_round_rect(&mut self, _brush: impl Brush, _rect: Rect, _round: Size) -> Result<()> {
        not_impl()
    }

    pub fn draw_str(
        &mut self,
        _brush: impl Brush,
        _font: DrawingFont,
        _pos: Point,
        _text: &str,
    ) -> Result<()> {
        not_impl()
    }

    pub fn measure_str(&self, _font: DrawingFont, _text: &str) -> Result<Size> {
        not_impl()
    }

    pub fn create_image(&self, _image: DynamicImage) -> Result<DrawingImage> {
        not_impl()
    }

    pub fn draw_image(
        &mut self,
        _image: &DrawingImage,
        _rect: Rect,
        _clip: Option<Rect>,
    ) -> Result<()> {
        not_impl()
    }

    pub fn create_path_builder(&self, _start: Point) -> Result<DrawingPathBuilder> {
        not_impl()
    }
}

pub struct DrawingImage;

impl DrawingImage {
    pub fn size(&self) -> Result<Size> {
        not_impl()
    }
}

pub struct DrawingPath;

pub struct DrawingPathBuilder;

impl DrawingPathBuilder {
    pub fn add_line(&mut self, _p: Point) -> Result<()> {
        not_impl()
    }

    pub fn add_arc(
        &mut self,
        _center: Point,
        _radius: Size,
        _start: f64,
        _end: f64,
        _clockwise: bool,
    ) -> Result<()> {
        not_impl()
    }

    pub fn add_bezier(&mut self, _p1: Point, _p2: Point, _p3: Point) -> Result<()> {
        not_impl()
    }

    pub fn build(self, _close: bool) -> Result<DrawingPath> {
        not_impl()
    }
}
