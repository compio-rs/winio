pub mod method;
pub mod style;

use super::graphics::{Canvas, Paint};
use crate::impl_listener;

jni::bind_java_type! {
    pub TextPaint => android.text.TextPaint,
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
    pub StaticLayout => android.text.StaticLayout,
    type_map {
        Canvas => android.graphics.Canvas,
    },
    methods {
        // fn get_width() -> jint,
        fn get_height() -> jint,
        fn get_line_count() -> jint,
        fn get_line_right(line: jint) -> jfloat,

        fn draw(canvas: &Canvas),
    },
}

jni::bind_java_type! {
    pub StaticLayoutBuilder => "android.text.StaticLayout$Builder",
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

jni::bind_java_type! {
    pub Editable => android.text.Editable,
    methods {
        fn to_string() -> JString,
    }
}

jni::bind_java_type! {
    pub TextWatcher => android.text.TextWatcher,
}

impl_listener!(TextWatcher);

jni::bind_java_type! {
    pub SpannableString => android.text.SpannableString,
    constructors {
        fn new(text: &JCharSequence),
    },
    methods {
        fn set_span(what: &JObject, start: i32, end: i32, flags: i32),
        fn to_string() -> JString,
    },
    is_instance_of = {
        char_sequence = JCharSequence,
    }
}

pub mod input_type {
    pub const TYPE_CLASS_TEXT: i32 = 0x1;
    pub const TYPE_TEXT_VARIATION_PASSWORD: i32 = 0x80;
    pub const TYPE_TEXT_FLAG_MULTI_LINE: i32 = 0x20000;
}

pub mod spanned {
    pub const SPAN_INCLUSIVE_EXCLUSIVE: i32 = 0x11;
}
