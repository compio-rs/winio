use super::{
    content::Context,
    graphics::{Insets, Rect},
};
use crate::impl_listener;

jni::bind_java_type! {
    pub MotionEvent => android.view.MotionEvent,
    methods {
        fn get_action() -> jint,
        fn get_action_button() -> jint,
        fn get_x() -> jfloat,
        fn get_y() -> jfloat,
        fn get_axis_value(axis: jint) -> jfloat,
    },
}

jni::bind_java_type! {
    pub View => "android.view.View",
    type_map {
        Context => android.content.Context,
        ViewParent => android.view.ViewParent,
        ViewGroupLayoutParams => "android.view.ViewGroup$LayoutParams",
        ViewOnLayoutChangeListener => "android.view.View$OnLayoutChangeListener",
        ViewOnTouchListener => "android.view.View$OnTouchListener",
        WindowInsets => android.view.WindowInsets,
    },
    constructors {
        fn new(context: &Context),
    },
    methods {
        fn get_x() -> jfloat,
        fn get_y() -> jfloat,
        fn set_x(x: jfloat),
        fn set_y(y: jfloat),
        fn get_width() -> jint,
        fn get_height() -> jint,
        fn get_layout_params() -> ViewGroupLayoutParams,
        fn set_layout_params(params: &ViewGroupLayoutParams),
        fn measure(width_spec: jint, height_spec: jint),
        fn get_measured_width() -> jint,
        fn get_measured_height() -> jint,
        fn get_minimum_width() -> jint,
        fn get_minimum_height() -> jint,
        fn get_visibility() -> jint,
        fn set_visibility(visibility: jint),
        fn is_enabled() -> jboolean,
        fn set_enabled(enabled: jboolean),
        fn get_parent() -> ViewParent,
        fn add_on_layout_change_listener(listener: &ViewOnLayoutChangeListener),
        fn set_on_touch_listener(listener: &ViewOnTouchListener),
        fn get_root_window_insets() -> WindowInsets,
    }
}

jni::bind_java_type! {
    pub ViewParent => android.view.ViewParent,
}

jni::bind_java_type! {
    pub ViewGroupLayoutParams => "android.view.ViewGroup$LayoutParams",
    fields {
        width: jint,
        height: jint,
    }
}

jni::bind_java_type! {
    pub ViewGroupMarginLayoutParams => "android.view.ViewGroup$MarginLayoutParams",
    type_map {
        ViewGroupLayoutParams => "android.view.ViewGroup$LayoutParams",
    },
    constructors {
        fn new(width: jint, height: jint),
    },
    fields {
        left_margin: jint,
        top_margin: jint,
        right_margin: jint,
        bottom_margin: jint,
    },
    is_instance_of = {
        base = ViewGroupLayoutParams,
    }
}

jni::bind_java_type! {
    pub ViewOnLayoutChangeListener => "android.view.View$OnLayoutChangeListener",
}

impl_listener!(ViewOnLayoutChangeListener);

jni::bind_java_type! {
    pub ViewOnTouchListener => "android.view.View$OnTouchListener",
}

impl_listener!(ViewOnTouchListener);

jni::bind_java_type! {
    pub ViewOnClickListener => "android.view.View$OnClickListener",
}

impl_listener!(ViewOnClickListener);

jni::bind_java_type! {
    pub ViewGroup => android.view.ViewGroup,
    type_map {
        View => android.view.View,
    },
    methods {
        fn add_view(view: &View),
        fn remove_view(view: &View),
    },
    is_instance_of = {
        view = View,
    }
}

jni::bind_java_type! {
    pub WindowInsets => android.view.WindowInsets,
    type_map {
        Insets => android.graphics.Insets,
    },
    methods {
        fn get_insets_ignoring_visibility(type_mask: jint) -> Insets,
    }
}

jni::bind_java_type! {
    pub WindowInsetsType => "android.view.WindowInsets$Type",
    methods {
        static fn system_bars() -> jint,
    }
}

jni::bind_java_type! {
    pub SurfaceView => android.view.SurfaceView,
    type_map {
        View => android.view.View,
        Context => android.content.Context,
        SurfaceHolder => android.view.SurfaceHolder,
    },
    constructors {
        fn new(context: &Context),
    },
    methods {
        fn get_holder() -> SurfaceHolder,
    },
    is_instance_of = {
        view = View,
    }
}

jni::bind_java_type! {
    pub SurfaceHolder => android.view.SurfaceHolder,
    type_map {
        Surface => android.view.Surface,
        Rect => android.graphics.Rect,
    },
    methods {
        fn get_surface() -> Surface,
        fn get_surface_frame() -> Rect,
    }
}

jni::bind_java_type! {
    pub Surface => android.view.Surface,
}

pub mod gravity {
    pub const CENTER_HORIZONTAL: i32 = 0x1;
    pub const CENTER_VERTICAL: i32 = 0x10;
    pub const FILL_HORIZONTAL: i32 = 0x7;
    pub const LEFT: i32 = 0x3;
    pub const RIGHT: i32 = 0x5;
    pub const TOP: i32 = 0x30;
}

pub mod motion_event {
    pub const ACTION_DOWN: i32 = 0x0;
    pub const ACTION_UP: i32 = 0x1;
    pub const ACTION_MOVE: i32 = 0x2;
    pub const ACTION_HOVER_MOVE: i32 = 0x7;
    pub const ACTION_SCROLL: i32 = 0x8;

    pub const AXIS_VSCROLL: i32 = 0x9;
    pub const AXIS_HSCROLL: i32 = 0xA;

    pub const BUTTON_PRIMARY: i32 = 0x1;
    pub const BUTTON_SECONDARY: i32 = 0x2;
    pub const BUTTON_TERTIARY: i32 = 0x4;
}
