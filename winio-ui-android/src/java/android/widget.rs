use super::{
    content::Context,
    graphics::drawable::Drawable,
    text::{Editable, TextWatcher, method::MovementMethod},
    util::SparseBooleanArray,
    view::{
        View, ViewGroup, ViewGroupLayoutParams, ViewGroupMarginLayoutParams, ViewOnClickListener,
    },
};
use crate::impl_listener;

jni::bind_java_type! {
    pub ImageView => android.widget.ImageView,
    type_map {
        View => android.view.View,
        Context => android.content.Context,
        Drawable => android.graphics.drawable.Drawable,
    },
    constructors {
        fn new(context: &Context),
    },
    methods {
        fn set_image_drawable(drawable: &Drawable),
    },
    is_instance_of = {
        view = View,
    }
}

jni::bind_java_type! {
    pub TextView => android.widget.TextView,
    type_map {
        View => android.view.View,
        Context => android.content.Context,
        MovementMethod => android.text.method.MovementMethod,
    },
    constructors {
        fn new(&Context),
    },
    methods {
        fn get_text() -> JCharSequence,
        fn set_text(text: &JCharSequence),
        fn get_gravity() -> jint,
        fn set_gravity(gravity: jint),
        fn set_movement_method(method: &MovementMethod),
    },
    is_instance_of = {
        view = View,
    }
}

jni::bind_java_type! {
    pub Button => android.widget.Button,
    type_map {
        View => android.view.View,
        TextView => android.widget.TextView,
        Context => android.content.Context,
        ViewOnClickListener => "android.view.View$OnClickListener"
    },
    constructors {
        fn new(context: &Context),
    },
    methods {
        fn set_on_click_listener(listener: &ViewOnClickListener),
    },
    is_instance_of = {
        view = View,
        text_view = TextView,
    }
}

jni::bind_java_type! {
    pub CompoundButton => android.widget.CompoundButton,
    type_map {
        Button => android.widget.Button,
    },
    methods {
        fn is_checked() -> jboolean,
        fn set_checked(checked: jboolean),
    },
    is_instance_of = {
        button = Button,
    }
}

jni::bind_java_type! {
    pub EditText => android.widget.EditText,
    type_map {
        View => android.view.View,
        TextView => android.widget.TextView,
        Context => android.content.Context,
        Editable => android.text.Editable,
        TextWatcher => android.text.TextWatcher,
    },
    constructors {
        fn new(&Context),
    },
    methods {
        fn get_text() -> Editable,
        fn get_input_type() -> jint,
        fn set_input_type(ty: jint),
        fn add_text_changed_listener(listener: &TextWatcher),
    },
    is_instance_of = {
        view = View,
        text_view = TextView,
    }
}

jni::bind_java_type! {
    pub FrameLayout => android.widget.FrameLayout,
    type_map {
        View => android.view.View,
        ViewGroup => android.view.ViewGroup,
        Context => android.content.Context,
    },
    constructors {
        fn new(context: &Context),
    },
    is_instance_of = {
        view = View,
        view_group = ViewGroup,
    }
}

jni::bind_java_type! {
    pub FrameLayoutLayoutParams => "android.widget.FrameLayout$LayoutParams",
    type_map {
        ViewGroupLayoutParams => "android.view.ViewGroup$LayoutParams",
        ViewGroupMarginLayoutParams => "android.view.ViewGroup$MarginLayoutParams",
    },
    constructors {
        fn new(width: jint, height: jint),
    },
    fields {
        gravity: jint,
    },
    is_instance_of = {
        base = ViewGroupLayoutParams,
        margin = ViewGroupMarginLayoutParams,
    }
}

jni::bind_java_type! {
    pub ListView => android.widget.ListView,
    type_map {
        View => android.view.View,
        Context => android.content.Context,
        ListAdapter => android.widget.ListAdapter,
        SparseBooleanArray => android.util.SparseBooleanArray,
        AdapterViewOnItemClickListener => "android.widget.AdapterView$OnItemClickListener",
    },
    constructors {
        fn new(&Context),
    },
    methods {
        fn get_choice_mode() -> jint,
        fn set_choice_mode(mode: jint),
        fn set_adapter(adapter: &ListAdapter),
        fn set_item_checked(position: jint, value: jboolean),
        fn get_checked_item_positions() -> SparseBooleanArray,
        fn set_on_item_click_listener(listener: &AdapterViewOnItemClickListener),
    },
    is_instance_of = {
        view = View,
    }
}

jni::bind_java_type! {
    pub Spinner => android.widget.Spinner,
    type_map {
        View => android.view.View,
        Context => android.content.Context,
        SpinnerAdapter => android.widget.SpinnerAdapter,
        AdapterViewOnItemSelectedListener => "android.widget.AdapterView$OnItemSelectedListener",
    },
    constructors {
        fn new(&Context),
    },
    methods {
        fn get_selected_item_position() -> jint,
        fn set_selection(position: jint),
        fn set_adapter(adapter: &SpinnerAdapter),
        fn set_on_item_selected_listener(listener: &AdapterViewOnItemSelectedListener),
    },
    is_instance_of = {
        view = View,
    }
}

jni::bind_java_type! {
    pub ScrollView => android.widget.ScrollView,
    type_map {
        View => android.view.View,
        Context => android.content.Context,
        ViewGroup => android.view.ViewGroup,
    },
    constructors {
        fn new(&Context),
    },
    is_instance_of = {
        view = View,
        view_group = ViewGroup,
    }
}

jni::bind_java_type! {
    pub HorizontalScrollView => android.widget.HorizontalScrollView,
    type_map {
        View => android.view.View,
        Context => android.content.Context,
        ViewGroup => android.view.ViewGroup,
    },
    constructors {
        fn new(&Context),
    },
    is_instance_of = {
        view = View,
        view_group = ViewGroup,
    }
}

jni::bind_java_type! {
    pub LinearLayout => android.widget.LinearLayout,
    type_map {
        View => android.view.View,
        ViewGroup => android.view.ViewGroup,
        Context => android.content.Context,
    },
    constructors {
        fn new(&Context),
    },
    methods {
        fn set_orientation(orient: jint),
    },
    is_instance_of = {
        view = View,
        view_group = ViewGroup,
    }
}

jni::bind_java_type! {
    pub LinearLayoutLayoutParams => "android.widget.LinearLayout$LayoutParams",
    type_map {
        ViewGroupLayoutParams => "android.view.ViewGroup$LayoutParams",
    },
    constructors {
        fn new(width: jint, height: jint),
        fn with_weight(width: jint, height: jint, weight: jfloat),
    },
    is_instance_of = {
        base = ViewGroupLayoutParams,
    }
}

jni::bind_java_type! {
    pub ListAdapter => android.widget.ListAdapter,
}

jni::bind_java_type! {
    pub ArrayAdapter => android.widget.ArrayAdapter,
    type_map {
        Context => android.content.Context,
        SpinnerAdapter => android.widget.SpinnerAdapter,
        ListAdapter => android.widget.ListAdapter,
    },
    constructors {
        fn new(context: &Context, resource: jint, objects: &JList),
    },
    methods {
        fn set_drop_down_view_resource(resource: jint),
        fn notify_data_set_changed(),
    },
    is_instance_of = {
        spinner_adapter = SpinnerAdapter,
        list_adapter = ListAdapter,
    }
}

jni::bind_java_type! {
    pub SpinnerAdapter => android.widget.SpinnerAdapter,
}

jni::bind_java_type! {
    pub AdapterViewOnItemClickListener => "android.widget.AdapterView$OnItemClickListener",
}

impl_listener!(AdapterViewOnItemClickListener);

jni::bind_java_type! {
    pub AdapterViewOnItemSelectedListener => "android.widget.AdapterView$OnItemSelectedListener",
}

pub mod abs_list_view {
    pub const CHOICE_MODE_MULTIPLE: i32 = 2;
    pub const CHOICE_MODE_SINGLE: i32 = 1;
}

pub mod linear_layout {
    pub const VERTICAL: i32 = 1;
}

impl_listener!(AdapterViewOnItemSelectedListener);
