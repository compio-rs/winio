use super::recyclerview::RecyclerViewAdapter;
use crate::java::android::{content::Context, view::View};

jni::bind_java_type! {
    pub ViewPager2 => androidx.viewpager2.widget.ViewPager2,
    type_map {
        View => android.view.View,
        Context => android.content.Context,
        RecyclerViewAdapter => "androidx.recyclerview.widget.RecyclerView$Adapter",
    },
    constructors {
        fn new(&Context),
    },
    methods {
        fn set_adapter(adapter: &RecyclerViewAdapter),
    },
    is_instance_of = {
        view = View,
    }
}
