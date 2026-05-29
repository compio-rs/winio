use std::marker::PhantomData;

use compio_log::error;
use image::DynamicImage;
use inherit_methods_macro::inherit_methods;
use jni::{Env, objects::JPrimitiveArray, refs::Global};
use winio_handle::{AsContainer, impl_as_widget};
use winio_primitive::{
    BrushPen, DrawingFont, GradientStop, LinearGradientBrush, MouseButton, Point,
    RadialGradientBrush, Rect, Size, SolidColorBrush, Transform, Vector,
};

use crate::{BaseWidget, Result, vm_exec};

jni::bind_java_type! {
    PaintStyle => "android.graphics.Paint$Style",
    fields {
        #[allow(non_snake_case)]
        static FILL: PaintStyle,
        #[allow(non_snake_case)]
        static STROKE: PaintStyle,
    },
}

jni::bind_java_type! {
    Shader => android.graphics.Shader,
}

jni::bind_java_type! {
    TileMode => "android.graphics.Shader$TileMode",
    fields {
        #[allow(non_snake_case)]
        static CLAMP: TileMode,
    }
}

jni::bind_java_type! {
    LinearGradient => android.graphics.LinearGradient,
    is_instance_of = {
        base: Shader,
    },
    type_map {
        Shader => android.graphics.Shader,
        TileMode => "android.graphics.Shader$TileMode",
    },
    constructors {
        #[allow(clippy::too_many_arguments)]
        fn new(
            x0: jfloat,
            y0: jfloat,
            x1: jfloat,
            y1: jfloat,
            colors: &[jint],
            positions: &[jfloat],
            mode: &TileMode,
        ),
    },
}

jni::bind_java_type! {
    RadialGradient => android.graphics.RadialGradient,
    is_instance_of = {
        base: Shader,
    },
    type_map {
        Shader => android.graphics.Shader,
        TileMode => "android.graphics.Shader$TileMode",
    },
    constructors {
        fn new(
            cx: jfloat,
            cy: jfloat,
            radius: jfloat,
            colors: &[jint],
            positions: &[jfloat],
            mode: &TileMode,
        ),
    },
}

jni::bind_java_type! {
    pub Paint => android.graphics.Paint,
    type_map {
        PaintStyle => "android.graphics.Paint$Style",
        Shader => android.graphics.Shader,
    },
    constructors {
        fn new(),
    },
    methods {
        fn set_a_r_g_b(a: jint, r: jint, g: jint, b: jint),
        fn set_style(style: &PaintStyle),
        fn set_shader(shader: &Shader),
        fn set_stroke_width(width: jfloat),
    },
}

/// Drawing brush.
pub trait Brush {
    fn create_paint<'local>(&self, env: &mut Env<'local>) -> Result<Paint<'local>>;
}

impl<B: Brush> Brush for &'_ B {
    fn create_paint<'local>(&self, env: &mut Env<'local>) -> Result<Paint<'local>> {
        B::create_paint(self, env)
    }
}

impl Brush for SolidColorBrush {
    fn create_paint<'local>(&self, env: &mut Env<'local>) -> Result<Paint<'local>> {
        let paint = Paint::new(env)?;
        paint.set_a_r_g_b(
            env,
            self.color.a as _,
            self.color.r as _,
            self.color.g as _,
            self.color.b as _,
        )?;
        let style = PaintStyle::FILL(env)?;
        paint.set_style(env, &style)?;
        Ok(paint)
    }
}

fn colors_stops<'local>(
    env: &mut Env<'local>,
    stops: &[GradientStop],
) -> Result<(JPrimitiveArray<'local, i32>, JPrimitiveArray<'local, f32>)> {
    let mut colors = vec![];
    let mut positions = vec![];
    for stop in stops {
        colors.push(
            ((stop.color.a as i32) << 24)
                | ((stop.color.r as i32) << 16)
                | ((stop.color.g as i32) << 8)
                | (stop.color.b as i32),
        );
        positions.push(stop.pos as f32);
    }
    let jcolors = env.new_int_array(stops.len() as _)?;
    jcolors.set_region(env, 0, &colors)?;
    let jpositions = env.new_float_array(stops.len() as _)?;
    jpositions.set_region(env, 0, &positions)?;
    Ok((jcolors, jpositions))
}

impl Brush for LinearGradientBrush {
    fn create_paint<'local>(&self, env: &mut Env<'local>) -> Result<Paint<'local>> {
        let paint = Paint::new(env)?;
        let style = PaintStyle::FILL(env)?;
        paint.set_style(env, &style)?;
        let (jcolors, jpositions) = colors_stops(env, &self.stops)?;
        let mode = TileMode::CLAMP(env)?;
        let gradient = LinearGradient::new(
            env,
            self.start.x as f32,
            self.start.y as f32,
            self.end.x as f32,
            self.end.y as f32,
            &jcolors,
            &jpositions,
            &mode,
        )?;
        paint.set_shader(env, &gradient)?;
        Ok(paint)
    }
}

impl Brush for RadialGradientBrush {
    fn create_paint<'local>(&self, env: &mut Env<'local>) -> Result<Paint<'local>> {
        let paint = Paint::new(env)?;
        let style = PaintStyle::FILL(env)?;
        paint.set_style(env, &style)?;
        let (jcolors, jpositions) = colors_stops(env, &self.stops)?;
        let mode = TileMode::CLAMP(env)?;
        let gradient = RadialGradient::new(
            env,
            self.center.x as f32,
            self.center.y as f32,
            self.radius.width as f32,
            &jcolors,
            &jpositions,
            &mode,
        )?;
        paint.set_shader(env, &gradient)?;
        Ok(paint)
    }
}

/// Drawing pen.
pub trait Pen {
    fn create_paint<'local>(&self, env: &mut Env<'local>) -> Result<Paint<'local>>;
}

impl<P: Pen> Pen for &'_ P {
    fn create_paint<'local>(&self, env: &mut Env<'local>) -> Result<Paint<'local>> {
        P::create_paint(self, env)
    }
}

impl<B: Brush> Pen for BrushPen<B> {
    fn create_paint<'local>(&self, env: &mut Env<'local>) -> Result<Paint<'local>> {
        let paint = self.brush.create_paint(env)?;
        paint.set_stroke_width(env, self.width as _)?;
        let style = PaintStyle::STROKE(env)?;
        paint.set_style(env, &style)?;
        Ok(paint)
    }
}

#[derive(Debug, Clone)]
pub struct DrawingImage;

impl DrawingImage {
    pub fn size(&self) -> Result<Size> {
        todo!()
    }
}

jni::bind_java_type! {
    ACanvas => android.graphics.Canvas,
    type_map {
        Paint => android.graphics.Paint,
    },
    methods {
        fn draw_line(start_x: jfloat, start_y: jfloat, end_x: jfloat, end_y: jfloat, paint: &Paint),
        fn draw_rect(left: jfloat, top: jfloat, right: jfloat, bottom: jfloat, paint: &Paint),
    }
}

jni::bind_java_type! {
    SurfaceHolder => android.view.SurfaceHolder,
    type_map {
        ACanvas => android.graphics.Canvas,
    },
    methods {
        fn lock_canvas() -> ACanvas,
        fn unlock_canvas_and_post(canvas: &ACanvas),
    },
}

pub struct DrawingContext<'a> {
    holder: Global<SurfaceHolder<'static>>,
    canvas: Global<ACanvas<'static>>,
    closed: bool,
    _p: PhantomData<&'a Canvas>,
}

impl Drop for DrawingContext<'_> {
    fn drop(&mut self) {
        if let Err(e) = self.close_impl() {
            error!("failed to unlock canvas: {e:?}");
        }
    }
}

impl DrawingContext<'_> {
    fn new(holder: Global<SurfaceHolder<'static>>, canvas: Global<ACanvas<'static>>) -> Self {
        Self {
            holder,
            canvas,
            closed: false,
            _p: PhantomData,
        }
    }

    fn close_impl(&mut self) -> Result<()> {
        if !self.closed {
            vm_exec(|env| {
                self.holder.unlock_canvas_and_post(env, &self.canvas)?;
                self.closed = true;
                Ok(())
            })
        } else {
            Ok(())
        }
    }

    pub fn close(mut self) -> Result<()> {
        self.close_impl()
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

    pub fn draw_line(&mut self, pen: impl Pen, start: Point, end: Point) -> Result<()> {
        vm_exec(|env| {
            let paint = pen.create_paint(env)?;
            self.canvas.draw_line(
                env,
                start.x as f32,
                start.y as f32,
                end.x as f32,
                end.y as f32,
                &paint,
            )?;
            Ok(())
        })
    }

    pub fn draw_rect(&mut self, pen: impl Pen, rect: Rect) -> Result<()> {
        let rect = rect.to_box2d();
        vm_exec(|env| {
            let paint = pen.create_paint(env)?;
            self.canvas.draw_rect(
                env,
                rect.min.x as f32,
                rect.min.y as f32,
                rect.max.x as f32,
                rect.max.y as f32,
                &paint,
            )?;
            Ok(())
        })
    }

    pub fn fill_rect(&mut self, brush: impl Brush, rect: Rect) -> Result<()> {
        let rect = rect.to_box2d();
        vm_exec(|env| {
            let paint = brush.create_paint(env)?;
            self.canvas.draw_rect(
                env,
                rect.min.x as f32,
                rect.min.y as f32,
                rect.max.x as f32,
                rect.max.y as f32,
                &paint,
            )?;
            Ok(())
        })
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

#[inherit_methods(from = "self.inner")]
impl Canvas {
    const WIDGET_CLASS: &'static str = "android/view/SurfaceView";

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
        vm_exec(|env| {
            let holder = env
                .call_method(
                    self.inner.as_obj(),
                    jni::jni_str!("getHolder"),
                    jni::jni_sig!("()Landroid/view/SurfaceHolder;"),
                    &[],
                )?
                .l()?;
            let canvas = env
                .call_method(
                    &holder,
                    jni::jni_str!("lockCanvas"),
                    jni::jni_sig!("()Landroid/graphics/Canvas;"),
                    &[],
                )?
                .l()?;
            let holder = unsafe { SurfaceHolder::from_raw(env, holder.into_raw()) };
            let holder = env.new_global_ref(holder)?;
            let canvas = unsafe { ACanvas::from_raw(env, canvas.into_raw()) };
            let canvas = env.new_global_ref(canvas)?;
            Ok(DrawingContext::new(holder, canvas))
        })
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
        std::future::pending().await
    }
}

impl_as_widget!(Canvas, inner);
