use crate::java::{
    android::{content::Context, text::style::ClickableSpan, view::View, webkit::WebViewClient},
    androidx::RecyclerViewAdapter,
    lang::JRunnable,
};

jni::bind_java_type! {
    pub Activity => rs.compio.winio.Activity,
    type_map {
        View => android.view.View,
        Context => android.content.Context,
    },
    methods {
        fn set_content_view(view: &View),
    },
    native_methods {
        extern fn on_create_native(),
    },
    is_instance_of = {
        context = Context,
    }
}

jni::bind_java_type! {
    pub WinioClickableSpan => rs.compio.winio.ClickableSpan,
    type_map {
        ClickableSpan => android.text.style.ClickableSpan,
        JRunnable => java.lang.Runnable,
    },
    constructors {
        fn new(),
    },
    methods {
        fn set_on_click(listener: &JRunnable),
    },
    is_instance_of = {
        base = ClickableSpan,
    }
}

jni::bind_java_type! {
    pub WinioTabViewAdapter => rs.compio.winio.TabViewAdapter,
    type_map {
        RecyclerViewAdapter => "androidx.recyclerview.widget.RecyclerView$Adapter",
    },
    constructors {
        fn new(),
    },
    methods {
        fn get_pages() -> JList,

        fn notify_item_inserted(position: jint),
        fn notify_item_removed(position: jint),
        fn notify_item_range_removed(start: jint, count: jint),
    },
    is_instance_of = {
        base = RecyclerViewAdapter,
    }
}

jni::bind_java_type! {
    pub WinioWebViewClient => rs.compio.winio.WebViewClient,
    type_map {
        WebViewClient => android.webkit.WebViewClient,
        JRunnable => java.lang.Runnable,
    },
    constructors {
        fn new(),
    },
    methods {
        fn set_on_page_started(listener: &JRunnable),
        fn set_on_page_finished(listener: &JRunnable),
    },
    is_instance_of = {
        base = WebViewClient,
    }
}
