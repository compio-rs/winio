use {
    super::{BaseWidget, vm_exec_on_ui_thread},
    image::DynamicImage,
    inherit_methods_macro::inherit_methods,
    jni::{objects::GlobalRef,JNIEnv,errors::Result as JniResult},
    std::marker::PhantomData,
    winio_handle::{AsWindow, impl_as_widget},
    winio_primitive::{
        BrushPen, DrawingFont, LinearGradientBrush, MouseButton, Point, RadialGradientBrush, Rect,
        Size, SolidColorBrush, Vector,
    },
};

/// Drawing brush.
pub trait Brush {
    fn get_raw(&self, env: &mut JNIEnv) -> JniResult<GlobalRef>;
}

impl<B: Brush> Brush for &'_ B {
    fn get_raw(&self, env: &mut JNIEnv) -> JniResult<GlobalRef> {
        B::get_raw(self, env)
    }
}

impl Brush for LinearGradientBrush {
    fn get_raw(&self, _env: &mut JNIEnv) -> JniResult<GlobalRef> {
        todo!()
    }
}

impl Brush for RadialGradientBrush {
    fn get_raw(&self, _env: &mut JNIEnv) -> JniResult<GlobalRef> {
        todo!()
    }
}

impl Brush for SolidColorBrush {
    fn get_raw(&self, _env: &mut JNIEnv) -> JniResult<GlobalRef> {
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
    pub fn size(&self) -> Size {
        todo!()
    }
}

pub struct DrawingContext<'a> {
    inner: GlobalRef,
    _a: PhantomData<&'a ()>,
}

impl<'a> DrawingContext<'a> {
    pub fn draw_path<P>(&self, _pen: P, _path: &DrawingPath)
    where
        P: Pen,
    {
        todo!()
    }

    pub fn fill_path<B>(&self, _brush: B, _path: &DrawingPath)
    where
        B: Brush,
    {
        todo!()
    }

    pub fn draw_arc<P>(&self, _pen: P, _rect: Rect, _start: f64, _end: f64)
    where
        P: Pen,
    {
        todo!()
    }

    pub fn draw_pie<P>(&self, _pen: P, _rect: Rect, _start: f64, _end: f64)
    where
        P: Pen,
    {
        todo!()
    }

    pub fn fill_pie<B>(&self, _brush: B, _rect: Rect, _start: f64, _end: f64)
    where
        B: Brush,
    {
        todo!()
    }

    pub fn draw_ellipse<P>(&self, _pen: P, _rect: Rect)
    where
        P: Pen,
    {
        todo!()
    }

    pub fn fill_ellipse<B>(&self, _brush: B, _rect: Rect)
    where
        B: Brush,
    {
        todo!()
    }

    pub fn draw_line<P>(&self, _pen: P, _start: Point, _end: Point)
    where
        P: Pen,
    {
        todo!()
    }

    pub fn draw_rect<P>(&self, _pen: P, _rect: Rect)
    where
        P: Pen,
    {
        todo!()
    }

    pub fn fill_rect<B>(&self, _brush: B, _rect: Rect)
    where
        B: Brush,
    {
        todo!()
    }

    pub fn draw_round_rect<P>(&self, _pen: P, _rect: Rect, _round: Size)
    where
        P: Pen,
    {
        todo!()
    }

    pub fn fill_round_rect<B>(&self, _brush: B, _rect: Rect, _round: Size)
    where
        B: Brush,
    {
        todo!()
    }

    pub fn draw_str<B, S>(&self, _brush: B, _font: DrawingFont, _pos: Point, _text: S)
    where
        B: Brush,
        S: AsRef<str>,
    {
        todo!()
    }

    pub fn create_image(&self, _image: DynamicImage) -> DrawingImage {
        todo!()
    }

    pub fn draw_image(&self, _image_rep: &DrawingImage, _rect: Rect, _clip: Option<Rect>) {
        todo!()
    }

    pub fn create_path_builder(&self, _start: Point) -> DrawingPathBuilder {
        todo!()
    }
}

pub struct DrawingPath;

pub struct DrawingPathBuilder;

impl DrawingPathBuilder {
    pub fn add_line(&self, _p: Point) {
        todo!()
    }

    pub fn add_arc(&self, _center: Point, _radius: Size, _start: f64, _end: f64, _clockwise: bool) {
        todo!()
    }

    pub fn add_bezier(&self, _p1: Point, _p2: Point, _p3: Point) {
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

//noinspection SpellCheckingInspection
#[inherit_methods(from = "self.inner")]
impl Canvas {
    const WIDGET_CLASS: &'static str = "rs/compio/winio/Canvas";

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

    pub fn context(&self) -> DrawingContext<'_> {
        let w = self.inner.clone();
        let inner = vm_exec_on_ui_thread(move |mut env, _| {
            let ctx = env
                .call_method(
                    w.as_obj(),
                    "context",
                    format!("()L{}$DrawingContext;", Self::WIDGET_CLASS),
                    &[],
                )?
                .l()?;
            env.new_global_ref(ctx)
        })
        .unwrap();

        DrawingContext {
            inner,
            _a: Default::default(),
        }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&self, visible: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&self, enabled: bool);

    pub fn loc(&self) -> Point;

    pub fn set_loc(&self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&self, v: Size);

    pub fn new<W>(parent: W) -> Self
    where
        W: AsWindow,
    {
        BaseWidget::create(parent.as_window(), Self::WIDGET_CLASS)
    }
}

impl From<BaseWidget> for Canvas {
    fn from(value: BaseWidget) -> Self {
        Self { inner: value }
    }
}

impl_as_widget!(Canvas, inner);
