use crate::{
    impl_listener,
    java::{
        android::{
            content::Context,
            view::View,
            widget::{Button, CompoundButton, TextView},
        },
        androidx::viewpager2::ViewPager2,
    },
};

jni::bind_java_type! {
    pub MaterialButton => com.google.android.material.button.MaterialButton,
    type_map {
        Button => android.widget.Button,
        View => android.view.View,
        TextView => android.widget.TextView,
        Context => android.content.Context,
    },
    constructors {
        fn new(context: &Context),
    },
    is_instance_of = {
        button = Button,
        view = View,
        text_view = TextView,
    }
}

jni::bind_java_type! {
    pub CheckBox => com.google.android.material.checkbox.MaterialCheckBox,
    type_map {
        Button => android.widget.Button,
        CompoundButton => android.widget.CompoundButton,
        View => android.view.View,
        TextView => android.widget.TextView,
        Context => android.content.Context,
    },
    constructors {
        fn new(context: &Context),
    },
    is_instance_of = {
        button = Button,
        compound_button = CompoundButton,
        view = View,
        text_view = TextView,
    }
}

jni::bind_java_type! {
    pub RadioButton => com.google.android.material.radiobutton.MaterialRadioButton,
    type_map {
        Button => android.widget.Button,
        CompoundButton => android.widget.CompoundButton,
        View => android.view.View,
        TextView => android.widget.TextView,
        Context => android.content.Context,
    },
    constructors {
        fn new(context: &Context),
    },
    is_instance_of = {
        button = Button,
        compound_button = CompoundButton,
        view = View,
        text_view = TextView,
    }
}

jni::bind_java_type! {
    pub ProgressBar => com.google.android.material.progressindicator.LinearProgressIndicator,
    type_map {
        View => android.view.View,
        Context => android.content.Context,
    },
    constructors {
        fn new(&Context),
    },
    methods {
        fn get_min() -> jint,
        fn set_min(min: jint),
        fn get_max() -> jint,
        fn set_max(max: jint),
        fn get_progress() -> jint,
        fn set_progress(progress: jint),
        fn is_indeterminate() -> jboolean,
        fn set_indeterminate(indeterminate: jboolean),
    },
    is_instance_of = {
        view = View,
    }
}

jni::bind_java_type! {
    pub Slider => com.google.android.material.slider.Slider,
    type_map {
        View => android.view.View,
        Context => android.content.Context,
        BaseOnChangeListener => com.google.android.material.slider.BaseOnChangeListener
    },
    constructors {
        fn new(&Context),
    },
    methods {
        fn get_value_from() -> jfloat,
        fn set_value_from(from: jfloat),
        fn get_value_to() -> jfloat,
        fn set_value_to(to: jfloat),
        fn get_value() -> jfloat,
        fn set_value(value: jfloat),

        fn get_tick_visibility_mode() -> jint,
        fn set_tick_visibility_mode(mode: jint),
        fn set_orientation(orient: jint),
        fn is_vertical() -> jboolean,

        fn add_on_change_listener(listener: &BaseOnChangeListener),
    },
    is_instance_of = {
        view = View,
    }
}

jni::bind_java_type! {
    pub BaseOnChangeListener => com.google.android.material.slider.BaseOnChangeListener,
}

impl_listener!(BaseOnChangeListener);

jni::bind_java_type! {
    pub SliderOnChangeListener => "com.google.android.material.slider.Slider$OnChangeListener",
    type_map {
        BaseOnChangeListener => com.google.android.material.slider.BaseOnChangeListener
    },
    is_instance_of = {
        base = BaseOnChangeListener,
    }
}

impl_listener!(SliderOnChangeListener);

jni::bind_java_type! {
    pub TabLayout => com.google.android.material.tabs.TabLayout,
    type_map {
        View => android.view.View,
        Context => android.content.Context,
        TabLayoutTab => "com.google.android.material.tabs.TabLayout$Tab",
        TabLayoutOnTabSelectedListener => "com.google.android.material.tabs.TabLayout$OnTabSelectedListener",
    },
    constructors {
        fn new(&Context),
    },
    methods {
        fn get_selected_tab_position() -> jint,
        fn get_tab_at(index: jint) -> TabLayoutTab,
        fn select_tab(tab: &TabLayoutTab),
        fn add_on_tab_selected_listener(listener: &TabLayoutOnTabSelectedListener),
    },
    is_instance_of = {
        view = View,
    }
}

jni::bind_java_type! {
    pub TabLayoutTab => "com.google.android.material.tabs.TabLayout$Tab",
    methods {
        fn set_text(text: &JCharSequence) -> TabLayoutTab,
    },
}

jni::bind_java_type! {
    pub TabLayoutMediator => com.google.android.material.tabs.TabLayoutMediator,
    type_map {
        TabLayout => com.google.android.material.tabs.TabLayout,
        ViewPager2 => androidx.viewpager2.widget.ViewPager2,
        TabLayoutMediatorTabConfigurationStrategy => "com.google.android.material.tabs.TabLayoutMediator$TabConfigurationStrategy",
    },
    constructors {
        fn new(&TabLayout, &ViewPager2, &TabLayoutMediatorTabConfigurationStrategy),
    },
    methods {
        fn attach(),
    }
}

jni::bind_java_type! {
    pub TabLayoutMediatorTabConfigurationStrategy => "com.google.android.material.tabs.TabLayoutMediator$TabConfigurationStrategy",
}

impl_listener!(TabLayoutMediatorTabConfigurationStrategy);

jni::bind_java_type! {
    pub TabLayoutOnTabSelectedListener => "com.google.android.material.tabs.TabLayout$OnTabSelectedListener",
}

impl_listener!(TabLayoutOnTabSelectedListener);
