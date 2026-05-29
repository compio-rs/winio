use std::sync::Arc;

use compio_log::error;
use image::DynamicImage;
use inherit_methods_macro::inherit_methods;
use jni::{
    Env,
    objects::JPrimitiveArray,
    refs::{Global, LoaderContext},
};
use jni_min_helper::{DynamicProxy, JBoolean};
use winio_callback::SyncCallback;
use winio_handle::{AsContainer, impl_as_widget};
use winio_primitive::{
    BrushPen, DrawingFont, GradientStop, HAlign, LinearGradientBrush, MouseButton, Point,
    RadialGradientBrush, Rect, Size, SolidColorBrush, Transform, VAlign, Vector,
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
        Typeface => android.graphics.Typeface,
    },
    constructors {
        fn new(),
    },
    methods {
        fn set_a_r_g_b(a: jint, r: jint, g: jint, b: jint),
        fn set_style(style: &PaintStyle),
        fn set_shader(shader: &Shader) -> Shader,
        fn set_stroke_width(width: jfloat),
        fn set_text_size(size: jfloat),
        fn set_typeface(typeface: &Typeface) -> Typeface,
    },
}

mod typeface {
    pub const NORMAL: i32 = 0x0;
    pub const BOLD: i32 = 0x1;
    pub const ITALIC: i32 = 0x2;
}

jni::bind_java_type! {
    Typeface => android.graphics.Typeface,
    methods {
        static fn create(family: JString, style: jint) -> Typeface,
    }
}

jni::bind_java_type! {
    TextPaint => android.text.TextPaint,
    type_map {
        Paint => android.graphics.Paint,
    },
    constructors {
        fn new(),
        fn with_paint(paint: &Paint),
    },
    is_instance_of = {
        base: Paint,
    },
}

jni::bind_java_type! {
    StaticLayout => android.text.StaticLayout,
    type_map {
        ACanvas => android.graphics.Canvas,
    },
    methods {
        // fn get_width() -> jint,
        fn get_height() -> jint,
        fn get_line_count() -> jint,
        fn get_line_right(line: jint) -> jfloat,

        fn draw(canvas: &ACanvas),
    },
}

jni::bind_java_type! {
    StaticLayoutBuilder => "android.text.StaticLayout$Builder",
    type_map {
        StaticLayout => android.text.StaticLayout,
        TextPaint => android.text.TextPaint,
    },
    methods {
        static fn obtain(
            source: JCharSequence,
            start: jint,
            end: jint,
            paint: &TextPaint,
            width: jint,
        ) -> StaticLayoutBuilder,
        fn build() -> StaticLayout,
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

jni::bind_java_type! {
    BitmapConfig => "android.graphics.Bitmap$Config",
    fields {
        #[allow(non_snake_case)]
        static ARGB_8888: BitmapConfig,
    }
}

jni::bind_java_type! {
    Bitmap => android.graphics.Bitmap,
    type_map {
        BitmapConfig => "android.graphics.Bitmap$Config",
    },
    methods {
        fn get_width() -> jint,
        fn get_height() -> jint,
        static fn create_bitmap(colors: &[jint], width: jint, height: jint, config: &BitmapConfig) -> Bitmap,
    },
}

#[derive(Debug)]
pub struct DrawingImage {
    bitmap: Global<Bitmap<'static>>,
}

impl DrawingImage {
    fn new(image: DynamicImage) -> Result<Self> {
        vm_exec(|env| {
            let rgba = image.to_rgba8();
            let (width, height) = rgba.dimensions();
            let pixels = rgba
                .pixels()
                .map(|p| {
                    ((p[3] as i32) << 24)
                        | ((p[0] as i32) << 16)
                        | ((p[1] as i32) << 8)
                        | (p[2] as i32)
                })
                .collect::<Vec<_>>();
            let jcolors = env.new_int_array(pixels.len())?;
            jcolors.set_region(env, 0, &pixels)?;
            let config = BitmapConfig::ARGB_8888(env)?;
            let bitmap = Bitmap::create_bitmap(env, &jcolors, width as _, height as _, &config)?;
            let bitmap = env.new_global_ref(bitmap)?;
            Ok(Self { bitmap })
        })
    }

    pub fn size(&self) -> Result<Size> {
        vm_exec(|env| {
            let width = self.bitmap.get_width(env)? as f64;
            let height = self.bitmap.get_height(env)? as f64;
            Ok(Size::new(width, height))
        })
    }
}

jni::bind_java_type! {
    ARect => android.graphics.Rect,
    constructors {
        fn new(left: jint, top: jint, right: jint, bottom: jint),
    },
    methods {
        fn left() -> jint,
        fn top() -> jint,
        fn right() -> jint,
        fn bottom() -> jint,
    },
}

jni::bind_java_type! {
    AMatrix => android.graphics.Matrix,
    constructors {
        fn new(),
    },
    methods {
        fn set_values(values: jfloat[]),
        fn get_values(values: jfloat[]),
    }
}

jni::bind_java_type! {
    ACanvas => android.graphics.Canvas,
    type_map {
        AMatrix => android.graphics.Matrix,
        ARect => android.graphics.Rect,
        Bitmap => android.graphics.Bitmap,
        Paint => android.graphics.Paint,
        Path => android.graphics.Path,
    },
    methods {
        fn clip_rect(left: jfloat, top: jfloat, right: jfloat, bottom: jfloat) -> bool,
        #[allow(clippy::too_many_arguments)]
        fn draw_arc(left: jfloat, top: jfloat, right: jfloat, bottom: jfloat, start_angle: jfloat, sweep_angle: jfloat, use_center: bool, paint: &Paint),
        fn draw_bitmap(bitmap: &Bitmap, src: &ARect, dest: &ARect, paint: &Paint),
        fn draw_line(start_x: jfloat, start_y: jfloat, end_x: jfloat, end_y: jfloat, paint: &Paint),
        fn draw_oval(left: jfloat, top: jfloat, right: jfloat, bottom: jfloat, paint: &Paint),
        fn draw_path(path: &Path, paint: &Paint),
        fn draw_rect(left: jfloat, top: jfloat, right: jfloat, bottom: jfloat, paint: &Paint),
        #[allow(clippy::too_many_arguments)]
        fn draw_round_rect(left: jfloat, top: jfloat, right: jfloat, bottom: jfloat, rx: jfloat, ry: jfloat, paint: &Paint),
        fn get_matrix() -> AMatrix,
        fn restore(),
        fn save() -> jint,
        fn set_matrix(matrix: &AMatrix),
        fn translate(dx: jfloat, dy: jfloat),
    }
}

jni::bind_java_type! {
    Picture => android.graphics.Picture,
    type_map {
        ACanvas => android.graphics.Canvas,
    },
    constructors {
        fn new(),
    },
    methods {
        fn begin_recording(width: jint, height: jint) -> ACanvas,
        fn end_recording(),
    },
}

jni::bind_java_type! {
    Drawable => android.graphics.drawable.Drawable,
}

jni::bind_java_type! {
    PictureDrawable => android.graphics.drawable.PictureDrawable,
    type_map {
        Drawable => android.graphics.drawable.Drawable,
        Picture => android.graphics.Picture,
    },
    constructors {
        fn new(picture: &Picture),
    },
    is_instance_of = {
        base: Drawable,
    },
}

pub struct DrawingContext<'a> {
    picture: Global<Picture<'static>>,
    canvas: Global<ACanvas<'static>>,
    closed: bool,
    parent: &'a Canvas,
}

impl Drop for DrawingContext<'_> {
    fn drop(&mut self) {
        if let Err(e) = self.close_impl() {
            error!("failed to unlock canvas: {e:?}");
        }
    }
}

impl<'a> DrawingContext<'a> {
    fn new(
        parent: &'a Canvas,
        picture: Global<Picture<'static>>,
        canvas: Global<ACanvas<'static>>,
    ) -> Self {
        Self {
            picture,
            canvas,
            closed: false,
            parent,
        }
    }

    fn close_impl(&mut self) -> Result<()> {
        if !self.closed {
            vm_exec(|env| {
                self.picture.end_recording(env)?;
                self.closed = true;
                let drawable = PictureDrawable::new(env, &self.picture)?;
                env.call_method(
                    self.parent.inner.as_obj(),
                    jni::jni_str!("setImageDrawable"),
                    jni::jni_sig!("(Landroid/graphics/drawable/Drawable;)V"),
                    &[(&drawable).into()],
                )?;
                Ok(())
            })
        } else {
            Ok(())
        }
    }

    pub fn close(mut self) -> Result<()> {
        self.close_impl()
    }

    pub fn set_transform(&mut self, transform: Transform) -> Result<()> {
        vm_exec(|env| {
            let matrix = AMatrix::new(env)?;
            let values = [
                transform.m11 as f32,
                transform.m12 as f32,
                transform.m21 as f32,
                transform.m22 as f32,
                transform.m31 as f32,
                transform.m32 as f32,
                0.0,
                0.0,
                1.0,
            ];
            let arr = env.new_float_array(values.len())?;
            arr.set_region(env, 0, &values)?;
            matrix.set_values(env, &arr)?;
            self.canvas.set_matrix(env, &matrix)?;
            Ok(())
        })
    }

    pub fn transform(&self) -> Result<Transform> {
        vm_exec(|env| {
            let matrix = self.canvas.get_matrix(env)?;
            let arr = env.new_float_array(9)?;
            matrix.get_values(env, &arr)?;
            let mut values = [0.0; 9];
            arr.get_region(env, 0, &mut values)?;
            Ok(Transform::new(
                values[0] as f64,
                values[1] as f64,
                values[2] as f64,
                values[3] as f64,
                values[4] as f64,
                values[5] as f64,
            ))
        })
    }

    pub fn draw_path(&mut self, pen: impl Pen, path: &DrawingPath) -> Result<()> {
        vm_exec(|env| {
            let paint = pen.create_paint(env)?;
            self.canvas.draw_path(env, &path.path, &paint)?;
            Ok(())
        })
    }

    pub fn fill_path(&mut self, brush: impl Brush, path: &DrawingPath) -> Result<()> {
        vm_exec(|env| {
            let paint = brush.create_paint(env)?;
            self.canvas.draw_path(env, &path.path, &paint)?;
            Ok(())
        })
    }

    pub fn draw_arc(&mut self, pen: impl Pen, rect: Rect, start: f64, end: f64) -> Result<()> {
        let rect = rect.to_box2d();
        vm_exec(|env| {
            let paint = pen.create_paint(env)?;
            self.canvas.draw_arc(
                env,
                rect.min.x as f32,
                rect.min.y as f32,
                rect.max.x as f32,
                rect.max.y as f32,
                to_degree(start as f32),
                to_degree((end - start) as f32),
                false,
                &paint,
            )?;
            Ok(())
        })
    }

    pub fn draw_pie(&mut self, pen: impl Pen, rect: Rect, start: f64, end: f64) -> Result<()> {
        let rect = rect.to_box2d();
        vm_exec(|env| {
            let paint = pen.create_paint(env)?;
            self.canvas.draw_arc(
                env,
                rect.min.x as f32,
                rect.min.y as f32,
                rect.max.x as f32,
                rect.max.y as f32,
                to_degree(start as f32),
                to_degree((end - start) as f32),
                true,
                &paint,
            )?;
            Ok(())
        })
    }

    pub fn fill_pie(&mut self, brush: impl Brush, rect: Rect, start: f64, end: f64) -> Result<()> {
        let rect = rect.to_box2d();
        vm_exec(|env| {
            let paint = brush.create_paint(env)?;
            self.canvas.draw_arc(
                env,
                rect.min.x as f32,
                rect.min.y as f32,
                rect.max.x as f32,
                rect.max.y as f32,
                to_degree(start as f32),
                to_degree((end - start) as f32),
                true,
                &paint,
            )?;
            Ok(())
        })
    }

    pub fn draw_ellipse(&mut self, pen: impl Pen, rect: Rect) -> Result<()> {
        let rect = rect.to_box2d();
        vm_exec(|env| {
            let paint = pen.create_paint(env)?;
            self.canvas.draw_oval(
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

    pub fn fill_ellipse(&mut self, brush: impl Brush, rect: Rect) -> Result<()> {
        let rect = rect.to_box2d();
        vm_exec(|env| {
            let paint = brush.create_paint(env)?;
            self.canvas.draw_oval(
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

    pub fn draw_round_rect(&mut self, pen: impl Pen, rect: Rect, round: Size) -> Result<()> {
        let rect = rect.to_box2d();
        vm_exec(|env| {
            let paint = pen.create_paint(env)?;
            self.canvas.draw_round_rect(
                env,
                rect.min.x as f32,
                rect.min.y as f32,
                rect.max.x as f32,
                rect.max.y as f32,
                round.width as f32,
                round.height as f32,
                &paint,
            )?;
            Ok(())
        })
    }

    pub fn fill_round_rect(&mut self, brush: impl Brush, rect: Rect, round: Size) -> Result<()> {
        let rect = rect.to_box2d();
        vm_exec(|env| {
            let paint = brush.create_paint(env)?;
            self.canvas.draw_round_rect(
                env,
                rect.min.x as f32,
                rect.min.y as f32,
                rect.max.x as f32,
                rect.max.y as f32,
                round.width as f32,
                round.height as f32,
                &paint,
            )?;
            Ok(())
        })
    }

    fn create_layout<'local>(
        &self,
        env: &mut Env<'local>,
        brush: Option<impl Brush>,
        font: &DrawingFont,
        text: &str,
    ) -> Result<(StaticLayout<'local>, Size)> {
        let mut style = typeface::NORMAL;
        if font.bold {
            style |= typeface::BOLD;
        }
        if font.italic {
            style |= typeface::ITALIC;
        }
        let family = env.new_string(&font.family)?;
        let typeface = Typeface::create(env, family, style)?;
        let paint = if let Some(brush) = brush {
            let paint = brush.create_paint(env)?;
            TextPaint::with_paint(env, paint)?
        } else {
            TextPaint::new(env)?
        };
        paint.as_base().set_typeface(env, &typeface)?;
        paint.as_base().set_text_size(env, font.size as f32)?;
        let text = env.new_string(text)?;
        let length = env
            .call_method(&text, jni::jni_str!("length"), jni::jni_sig!("()I"), &[])?
            .i()?;
        let builder = StaticLayoutBuilder::obtain(
            env,
            text,
            0,
            length,
            &paint,
            self.parent.latest_size.width as _,
        )?;
        let layout = builder.build(env)?;
        let height = layout.get_height(env)? as f64;
        let mut width = 0.0f64;
        let count = layout.get_line_count(env)?;
        for i in 0..count {
            let line_width = layout.get_line_right(env, i)? as f64;
            width = width.max(line_width);
        }
        Ok((layout, Size::new(width, height)))
    }

    pub fn draw_str(
        &mut self,
        brush: impl Brush,
        font: DrawingFont,
        pos: Point,
        text: &str,
    ) -> Result<()> {
        vm_exec(|env| {
            let (layout, size) = self.create_layout(env, Some(brush), &font, text)?;
            let width = size.width;
            let height = size.height;
            let mut x = pos.x;
            let mut y = pos.y;
            match font.halign {
                HAlign::Center => {
                    x -= width / 2.0;
                }
                HAlign::Right => {
                    x -= width;
                }
                _ => {}
            }
            match font.valign {
                VAlign::Center => {
                    y -= height / 2.0;
                }
                VAlign::Bottom => {
                    y -= height;
                }
                _ => {}
            }
            self.canvas.translate(env, x as _, y as _)?;
            layout.draw(env, &self.canvas)?;
            self.canvas.translate(env, -x as _, -y as _)?;
            Ok(())
        })
    }

    pub fn measure_str(&self, font: DrawingFont, text: &str) -> Result<Size> {
        vm_exec(|env| {
            let (_, size) = self.create_layout(env, None::<SolidColorBrush>, &font, text)?;
            Ok(size)
        })
    }

    pub fn create_image(&self, image: DynamicImage) -> Result<DrawingImage> {
        DrawingImage::new(image)
    }

    pub fn draw_image(
        &mut self,
        image: &DrawingImage,
        rect: Rect,
        clip: Option<Rect>,
    ) -> Result<()> {
        vm_exec(|env| {
            if let Some(clip) = clip {
                self.canvas.save(env)?;
                let clip = clip.to_box2d();
                self.canvas.clip_rect(
                    env,
                    clip.min.x as f32,
                    clip.min.y as f32,
                    clip.max.x as f32,
                    clip.max.y as f32,
                )?;
            }

            let size = image.size()?;
            let src = ARect::new(env, 0, 0, size.width as _, size.height as _)?;
            let rect = rect.to_box2d();
            let dest = ARect::new(
                env,
                rect.min.x as _,
                rect.min.y as _,
                rect.max.x as _,
                rect.max.y as _,
            )?;
            let paint = Paint::new(env)?;
            let style = PaintStyle::FILL(env)?;
            paint.set_style(env, style)?;
            self.canvas
                .draw_bitmap(env, &image.bitmap, src, dest, paint)?;

            if clip.is_some() {
                self.canvas.restore(env)?;
            }
            Ok(())
        })
    }

    pub fn create_path_builder(&self, start: Point) -> Result<DrawingPathBuilder> {
        DrawingPathBuilder::new(start)
    }
}

jni::bind_java_type! {
    Path => android.graphics.Path,
    constructors {
        fn new(),
    },
    methods {
        #[allow(clippy::too_many_arguments)]
        fn arc_to(left: jfloat, top: jfloat, right: jfloat, bottom: jfloat, start_angle: jfloat, sweep_angle: jfloat, force_move_to: bool),
        fn close(),
        #[allow(clippy::too_many_arguments)]
        fn cubic_to(x1: jfloat, y1: jfloat, x2: jfloat, y2: jfloat, x3: jfloat, y3: jfloat),
        fn line_to(x: jfloat, y: jfloat),
        fn move_to(x: jfloat, y: jfloat),
    }
}

const fn to_degree(radian: f32) -> f32 {
    radian * 180.0 / std::f32::consts::PI
}

pub struct DrawingPath {
    path: Global<Path<'static>>,
}

pub struct DrawingPathBuilder {
    path: Global<Path<'static>>,
}

impl DrawingPathBuilder {
    fn new(point: Point) -> Result<Self> {
        vm_exec(|env| {
            let path = Path::new(env)?;
            path.move_to(env, point.x as f32, point.y as f32)?;
            let path = env.new_global_ref(path)?;
            Ok(Self { path })
        })
    }

    pub fn add_line(&mut self, p: Point) -> Result<()> {
        vm_exec(|env| {
            self.path.line_to(env, p.x as f32, p.y as f32)?;
            Ok(())
        })
    }

    pub fn add_arc(
        &mut self,
        center: Point,
        radius: Size,
        start: f64,
        end: f64,
        clockwise: bool,
    ) -> Result<()> {
        let startp = Point::new(
            center.x + radius.width * start.cos(),
            center.y + radius.height * start.sin(),
        );
        self.add_line(startp)?;

        vm_exec(|env| {
            let left = center.x - radius.width;
            let top = center.y - radius.height;
            let right = center.x + radius.width;
            let bottom = center.y + radius.height;
            let sweep = end - start;
            let start = if clockwise { start } else { end };
            self.path.arc_to(
                env,
                left as f32,
                top as f32,
                right as f32,
                bottom as f32,
                to_degree(start as f32),
                to_degree(sweep as f32),
                true,
            )?;
            Ok(())
        })
    }

    pub fn add_bezier(&mut self, p1: Point, p2: Point, p3: Point) -> Result<()> {
        vm_exec(|env| {
            self.path.cubic_to(
                env,
                p1.x as f32,
                p1.y as f32,
                p2.x as f32,
                p2.y as f32,
                p3.x as f32,
                p3.y as f32,
            )?;
            Ok(())
        })
    }

    pub fn build(self, close: bool) -> Result<DrawingPath> {
        if close {
            vm_exec(|env| Ok(self.path.close(env)?))?;
        }
        Ok(DrawingPath { path: self.path })
    }
}

#[derive(Debug)]
pub struct Canvas {
    inner: BaseWidget,
    on_down: Arc<SyncCallback>,
    on_up: Arc<SyncCallback>,
    on_move: Arc<SyncCallback<Point>>,
    #[allow(dead_code)]
    touch_proxy: DynamicProxy,
    latest_size: Size,
}

#[inherit_methods(from = "self.inner")]
impl Canvas {
    const WIDGET_CLASS: &'static str = "android/widget/ImageView";

    pub fn new(parent: impl AsContainer) -> Result<Self> {
        vm_exec(|env| {
            let inner = BaseWidget::new_with_env(env, parent.as_container(), Self::WIDGET_CLASS)?;
            let on_down = Arc::new(SyncCallback::new());
            let on_up = Arc::new(SyncCallback::new());
            let on_move = Arc::new(SyncCallback::new());
            let touch_proxy = DynamicProxy::build(
                env,
                &LoaderContext::None,
                [jni::jni_str!("android.view.View$OnTouchListener")],
                {
                    let on_down = on_down.clone();
                    let on_up = on_up.clone();
                    let on_move = on_move.clone();
                    move |env, _method, args| {
                        const ACTION_DOWN: i32 = 0x0;
                        const ACTION_UP: i32 = 0x1;
                        const ACTION_MOVE: i32 = 0x2;

                        let event = args.get_element(env, 1)?;
                        let action = env
                            .call_method(
                                &event,
                                jni::jni_str!("getAction"),
                                jni::jni_sig!("()I"),
                                &[],
                            )?
                            .i()?;
                        match action & 0xFF {
                            ACTION_DOWN => {
                                on_down.signal(());
                            }
                            ACTION_UP => {
                                on_up.signal(());
                            }
                            ACTION_MOVE => {
                                let x = env
                                    .call_method(
                                        &event,
                                        jni::jni_str!("getX"),
                                        jni::jni_sig!("()F"),
                                        &[],
                                    )?
                                    .f()?;
                                let y = env
                                    .call_method(
                                        &event,
                                        jni::jni_str!("getY"),
                                        jni::jni_sig!("()F"),
                                        &[],
                                    )?
                                    .f()?;
                                let point = Point::new(x as f64, y as f64);
                                on_move.signal(point);
                            }
                            _ => {}
                        }
                        Ok(JBoolean::new(env, true)?.into())
                    }
                },
            )?;
            env.call_method(
                inner.as_obj(),
                jni::jni_str!("setOnTouchListener"),
                jni::jni_sig!("(Landroid/view/View$OnTouchListener;)V"),
                &[touch_proxy.as_ref().into()],
            )?;
            Ok(Self {
                inner,
                on_down,
                on_up,
                on_move,
                touch_proxy,
                latest_size: Size::zero(),
            })
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, visible: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size> {
        let size = self.latest_size;
        if size == Size::zero() {
            self.inner.size()
        } else {
            Ok(size)
        }
    }

    pub fn set_size(&mut self, v: Size) -> Result<()> {
        self.latest_size = v;
        self.inner.set_size(v)
    }

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn context(&self) -> Result<DrawingContext<'_>> {
        vm_exec(|env| {
            let picture = Picture::new(env)?;
            let picture = env.new_global_ref(picture)?;
            let canvas = picture.begin_recording(
                env,
                self.latest_size.width as _,
                self.latest_size.height as _,
            )?;
            let canvas = env.new_global_ref(canvas)?;
            Ok(DrawingContext::new(self, picture, canvas))
        })
    }

    pub async fn wait_mouse_move(&self) -> Point {
        self.on_move.wait().await
    }

    pub async fn wait_mouse_down(&self) -> MouseButton {
        self.on_down.wait().await;
        MouseButton::Left
    }

    pub async fn wait_mouse_up(&self) -> MouseButton {
        self.on_up.wait().await;
        MouseButton::Left
    }

    pub async fn wait_mouse_wheel(&self) -> Vector {
        std::future::pending().await
    }
}

impl_as_widget!(Canvas, inner);
