use super::{content::Context, view::View};
use crate::impl_listener;

jni::bind_java_type! {
    pub WebView => android.webkit.WebView,
    type_map {
        View => android.view.View,
        WebViewClient => android.webkit.WebViewClient,
        WebSettings => android.webkit.WebSettings,
        WebChromeClient => android.webkit.WebChromeClient,
        Context => android.content.Context,
        ValueCallback => android.webkit.ValueCallback,
    },
    constructors {
        fn new(context: &Context),
    },
    methods {
        fn get_url() -> JString,
        fn load_url(url: &JString),
        fn load_data(data: &JString, mime: &JString, encoding: &JString),
        fn can_go_forward() -> jboolean,
        fn go_forward(),
        fn can_go_back() -> jboolean,
        fn go_back(),
        fn reload(),
        fn stop_loading(),
        fn set_web_view_client(client: &WebViewClient),
        fn get_settings() -> WebSettings,
        fn set_web_chrome_client(client: &WebChromeClient),
        fn evaluate_javascript(script: &JString, callback: &ValueCallback),
    },
    is_instance_of = {
        view = View,
    }
}

jni::bind_java_type! {
    pub WebSettings => android.webkit.WebSettings,
    methods {
        fn set_java_script_enabled(enabled: bool),
    }
}

jni::bind_java_type! {
    pub ValueCallback => android.webkit.ValueCallback,
}

impl_listener!(ValueCallback);

jni::bind_java_type! {
    pub WebChromeClient => android.webkit.WebChromeClient,
    constructors {
        fn new(),
    },
}

jni::bind_java_type! {
    pub WebViewClient => android.webkit.WebViewClient,
}

jni::bind_java_type! {
    pub CookieManager => android.webkit.CookieManager,
    type_map {
        ValueCallback => android.webkit.ValueCallback,
    },
    methods {
        static fn get_instance() -> CookieManager,

        fn set_accept_cookie(accept: bool),
        fn set_cookie(url: &JString, value: &JString, callback: &ValueCallback),
        fn get_cookie(url: &JString) -> JString,
    }
}
