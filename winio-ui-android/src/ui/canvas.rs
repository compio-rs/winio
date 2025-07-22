use {
    image::DynamicImage,
    std::marker::PhantomData,
    winio_handle::{AsWindow, impl_as_widget},
    super::BaseWidget,
    inherit_methods_macro::inherit_methods,
    winio_primitive::{
        BrushPen, DrawingFont, LinearGradientBrush, MouseButton, Point, RadialGradientBrush, Rect,
        Size, SolidColorBrush, Vector,
    },
};

/// Drawing brush.
pub trait Brush {}

impl<B: Brush> Brush for &'_ B {}

impl Brush for LinearGradientBrush {}

impl Brush for RadialGradientBrush {}

impl Brush for SolidColorBrush {}

/// Drawing pen.
pub trait Pen {}

impl<P: Pen> Pen for &'_ P {}

impl<B: Brush> Pen for BrushPen<B> {}

#[derive(Debug, Clone)]
pub struct DrawingImage;

impl DrawingImage {
    pub fn size(&self) -> Size {
        todo!()
    }
}

pub struct DrawingContext<'a> {
    _a: PhantomData<&'a ()>,
}

impl<'a> DrawingContext<'a> {
    pub fn draw_path<P>(&mut self, _pen: P, _path: &DrawingPath)
    where
        P: Pen,
    {
        todo!()
    }

    pub fn fill_path<B>(&mut self, _brush: B, _path: &DrawingPath)
    where
        B: Brush,
    {
        todo!()
    }

    pub fn draw_arc<P>(&mut self, _pen: P, _rect: Rect, _start: f64, _end: f64)
    where
        P: Pen,
    {
        todo!()
    }

    pub fn draw_pie<P>(&mut self, _pen: P, _rect: Rect, _start: f64, _end: f64)
    where
        P: Pen,
    {
        todo!()
    }

    pub fn fill_pie<B>(&mut self, _brush: B, _rect: Rect, _start: f64, _end: f64)
    where
        B: Brush,
    {
        todo!()
    }

    pub fn draw_ellipse<P>(&mut self, _pen: P, _rect: Rect)
    where
        P: Pen,
    {
        todo!()
    }

    pub fn fill_ellipse<B>(&mut self, _brush: B, _rect: Rect)
    where
        B: Brush,
    {
        todo!()
    }

    pub fn draw_line<P>(&mut self, _pen: P, _start: Point, _end: Point)
    where
        P: Pen,
    {
        todo!()
    }

    pub fn draw_rect<P>(&mut self, _pen: P, _rect: Rect)
    where
        P: Pen,
    {
        todo!()
    }

    pub fn fill_rect<B>(&mut self, _brush: B, _rect: Rect)
    where
        B: Brush,
    {
        todo!()
    }

    pub fn draw_round_rect<P>(&mut self, _pen: P, _rect: Rect, _round: Size)
    where
        P: Pen,
    {
        todo!()
    }

    pub fn fill_round_rect<B>(&mut self, _brush: B, _rect: Rect, _round: Size)
    where
        B: Brush,
    {
        todo!()
    }

    pub fn draw_str<B, S>(&mut self, _brush: B, _font: DrawingFont, _pos: Point, _text: S)
    where
        B: Brush,
        S: AsRef<str>,
    {
        todo!()
    }

    pub fn create_image(&self, _image: DynamicImage) -> DrawingImage {
        todo!()
    }

    pub fn draw_image(&mut self, _image_rep: &DrawingImage, _rect: Rect, _clip: Option<Rect>) {
        todo!()
    }

    pub fn create_path_builder(&self, _start: Point) -> DrawingPathBuilder {
        todo!()
    }
}

pub struct DrawingPath;

pub struct DrawingPathBuilder;

impl DrawingPathBuilder {
    pub fn add_line(&mut self, _p: Point) {
        todo!()
    }

    pub fn add_arc(
        &mut self,
        _center: Point,
        _radius: Size,
        _start: f64,
        _end: f64,
        _clockwise: bool,
    ) {
        todo!()
    }

    pub fn add_bezier(&mut self, _p1: Point, _p2: Point, _p3: Point) {
        todo!()
    }

    pub fn build(self, _close: bool) -> DrawingPath {
        todo!()
    }
}

#[derive(Debug)]
pub struct Canvas {
    inner: BaseWidget,
}

#[inherit_methods(from = "self.inner")]
impl Canvas {
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

    pub fn context(&mut self) -> DrawingContext<'_> {
        todo!()
    }

    pub fn is_visible(&self) -> bool {
        todo!()
    }

    pub fn set_visible(&mut self, _v: bool) {
        todo!()
    }

    pub fn is_enabled(&self) -> bool {
        todo!()
    }

    pub fn set_enabled(&mut self, _v: bool) {
        todo!()
    }

    pub fn loc(&self) -> Point {
        todo!()
    }

    pub fn set_loc(&mut self, _p: Point) {
        todo!()
    }

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);

    pub fn new<W>(_parent: W) -> Self
    where
        W: AsWindow,
    {
        todo!()
    }
}

impl_as_widget!(Canvas, inner);
