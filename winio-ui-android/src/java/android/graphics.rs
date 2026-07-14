jni::bind_java_type! {
    pub PaintStyle => "android.graphics.Paint$Style",
    fields {
        #[allow(non_snake_case)]
        static FILL: PaintStyle,
        #[allow(non_snake_case)]
        static STROKE: PaintStyle,
    },
}

jni::bind_java_type! {
    pub Shader => android.graphics.Shader,
}

jni::bind_java_type! {
    pub ShaderTileMode => "android.graphics.Shader$TileMode",
    fields {
        #[allow(non_snake_case)]
        static CLAMP: ShaderTileMode,
    }
}

jni::bind_java_type! {
    pub LinearGradient => android.graphics.LinearGradient,
    is_instance_of = {
        base: Shader,
    },
    type_map {
        Shader => android.graphics.Shader,
        ShaderTileMode => "android.graphics.Shader$TileMode",
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
            mode: &ShaderTileMode,
        ),
    },
}

jni::bind_java_type! {
    pub RadialGradient => android.graphics.RadialGradient,
    is_instance_of = {
        base: Shader,
    },
    type_map {
        Shader => android.graphics.Shader,
        ShaderTileMode => "android.graphics.Shader$TileMode",
    },
    constructors {
        fn new(
            cx: jfloat,
            cy: jfloat,
            radius: jfloat,
            colors: &[jint],
            positions: &[jfloat],
            mode: &ShaderTileMode,
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

jni::bind_java_type! {
    pub Typeface => android.graphics.Typeface,
    methods {
        static fn create(family: JString, style: jint) -> Typeface,
    }
}

jni::bind_java_type! {
    pub BitmapConfig => "android.graphics.Bitmap$Config",
    fields {
        #[allow(non_snake_case)]
        static ARGB_8888: BitmapConfig,
    },
}

jni::bind_java_type! {
    pub Bitmap => android.graphics.Bitmap,
    type_map {
        BitmapConfig => "android.graphics.Bitmap$Config",
    },
    methods {
        fn get_width() -> jint,
        fn get_height() -> jint,
        static fn create_bitmap(colors: &[jint], width: jint, height: jint, config: &BitmapConfig) -> Bitmap,
    },
}

jni::bind_java_type! {
    pub Rect => android.graphics.Rect,
    constructors {
        fn new(left: jint, top: jint, right: jint, bottom: jint),
    },
    fields {
        left: jint,
        top: jint,
        right: jint,
        bottom: jint,
    }
}

jni::bind_java_type! {
    pub Matrix => android.graphics.Matrix,
    constructors {
        fn new(),
    },
    methods {
        fn set_values(values: jfloat[]),
        fn get_values(values: jfloat[]),
    }
}

jni::bind_java_type! {
    pub Canvas => android.graphics.Canvas,
    type_map {
        Bitmap => android.graphics.Bitmap,
        Paint => android.graphics.Paint,
        Rect => android.graphics.Rect,
        Matrix => android.graphics.Matrix,
        Picture => android.graphics.Picture,
        Path => android.graphics.Path,
    },
    constructors {
        fn new(bitmap: &Bitmap),
    },
    methods {
        fn clip_rect(left: jfloat, top: jfloat, right: jfloat, bottom: jfloat) -> bool,
        #[allow(clippy::too_many_arguments)]
        fn draw_arc(left: jfloat, top: jfloat, right: jfloat, bottom: jfloat, start_angle: jfloat, sweep_angle: jfloat, use_center: bool, paint: &Paint),
        fn draw_bitmap(bitmap: &Bitmap, src: &Rect, dest: &Rect, paint: &Paint),
        fn draw_line(start_x: jfloat, start_y: jfloat, end_x: jfloat, end_y: jfloat, paint: &Paint),
        fn draw_oval(left: jfloat, top: jfloat, right: jfloat, bottom: jfloat, paint: &Paint),
        fn draw_path(path: &Path, paint: &Paint),
        fn draw_rect(left: jfloat, top: jfloat, right: jfloat, bottom: jfloat, paint: &Paint),
        #[allow(clippy::too_many_arguments)]
        fn draw_round_rect(left: jfloat, top: jfloat, right: jfloat, bottom: jfloat, rx: jfloat, ry: jfloat, paint: &Paint),
        fn get_matrix() -> Matrix,
        fn set_matrix(matrix: &Matrix),
        fn translate(dx: jfloat, dy: jfloat),
    }
}

jni::bind_java_type! {
    pub Picture => android.graphics.Picture,
    type_map {
        Canvas => android.graphics.Canvas,
    },
    constructors {
        fn new(),
    },
    methods {
        fn begin_recording(width: jint, height: jint) -> Canvas,
        fn end_recording(),
    }
}

jni::bind_java_type! {
    pub Drawable => android.graphics.drawable.Drawable,
}

jni::bind_java_type! {
    pub PictureDrawable => android.graphics.drawable.PictureDrawable,
    type_map {
        Drawable => android.graphics.drawable.Drawable,
        Picture => android.graphics.Picture,
    },
    constructors {
        fn new(picture: &Picture),
    },
    is_instance_of = {
        base = Drawable,
    }
}

jni::bind_java_type! {
    pub Path => android.graphics.Path,
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

jni::bind_java_type! {
    pub Insets => android.graphics.Insets,
    fields {
        left: jint,
        top: jint,
        right: jint,
        bottom: jint,
    }
}
