pub mod res;

use super::{
    content::res::{Resources, ResourcesTheme},
    database::Cursor,
    net::Uri,
    os::ParcelFileDescriptor,
};
use crate::{
    impl_listener,
    java::lang::{File, InputStream},
};

jni::bind_java_type! {
    pub Context => android.content.Context,
    type_map {
        Resources => android.content.res.Resources,
        ResourcesTheme => "android.content.res.Resources$Theme",
        ContentResolver => android.content.ContentResolver,
        File => java.io.File,
    },
    methods {
        fn get_resources() -> Resources,
        fn get_theme() -> ResourcesTheme,
        fn get_content_resolver() -> ContentResolver,
        fn get_system_service(name: JString) -> JObject,
        fn get_package_name() -> JString,
        fn get_cache_dir() -> File,
    }
}

jni::bind_java_type! {
    pub ClipboardManagerOnPrimaryClipChangedListener => "android.content.ClipboardManager$OnPrimaryClipChangedListener",
}

impl_listener!(ClipboardManagerOnPrimaryClipChangedListener);

jni::bind_java_type! {
    pub ClipboardManager => android.content.ClipboardManager,
    type_map {
        ClipData => android.content.ClipData,
        ClipboardManagerOnPrimaryClipChangedListener => "android.content.ClipboardManager$OnPrimaryClipChangedListener",
    },
    methods {
        fn set_primary_clip(clip: &ClipData),
        fn get_primary_clip() -> ClipData,
        fn clear_primary_clip(),
        fn add_primary_clip_changed_listener(listener: &ClipboardManagerOnPrimaryClipChangedListener),
        fn remove_primary_clip_changed_listener(listener: &ClipboardManagerOnPrimaryClipChangedListener),
    },
}

jni::bind_java_type! {
    pub ClipDescription => android.content.ClipDescription,
    methods {
        fn get_mime_type_count() -> jint,
        fn get_mime_type(index: jint) -> JString,
        fn has_mime_type(mime_type: JString) -> jboolean,
    },
}

jni::bind_java_type! {
    pub ClipData => android.content.ClipData,
    type_map {
        Uri => android.net.Uri,
        ContentResolver => android.content.ContentResolver,
        ClipDescription => android.content.ClipDescription,
        ClipDataItem => "android.content.ClipData$Item",
    },
    methods {
        static fn new_plain_text(label: JCharSequence, text: JCharSequence) -> ClipData,
        static fn new_html_text(label: JCharSequence, text: JCharSequence, html_text: JString) -> ClipData,
        static fn new_raw_uri(label: JCharSequence, uri: &Uri) -> ClipData,
        static fn new_uri(resolver: &ContentResolver, label: JCharSequence, uri: &Uri) -> ClipData,
        fn get_description() -> ClipDescription,
        fn get_item_count() -> jint,
        fn get_item_at(index: jint) -> ClipDataItem,
        fn add_item(item: &ClipDataItem),
    },
}

jni::bind_java_type! {
    pub ClipDataItem => "android.content.ClipData$Item",
    type_map {
        Uri => android.net.Uri,
    },
    constructors {
        fn with_text(text: JCharSequence),
        fn with_uri(uri: &Uri),
    },
    methods {
        fn get_text() -> JCharSequence,
        fn get_html_text() -> JString,
        fn get_uri() -> Uri,
    },
}

jni::bind_java_type! {
    pub ContentResolver => android.content.ContentResolver,
    type_map {
        ParcelFileDescriptor => android.os.ParcelFileDescriptor,
        Uri => android.net.Uri,
        Cursor => android.database.Cursor,
        InputStream => java.io.InputStream,
    },
    methods {
        fn open_input_stream(uri: &Uri) -> InputStream,
        fn open_file_descriptor(uri: Uri, mode: JString) -> ParcelFileDescriptor,
        fn query(uri: Uri, projection: JString[], selection: JString, selection_args: JString[], sort_order: JString) -> Cursor,
    }
}

jni::bind_java_type! {
    pub DialogInterfaceOnClickListener => "android.content.DialogInterface$OnClickListener",
}

impl_listener!(DialogInterfaceOnClickListener);

jni::bind_java_type! {
    pub DialogInterfaceOnCancelListener => "android.content.DialogInterface$OnCancelListener",
}

impl_listener!(DialogInterfaceOnCancelListener);

jni::bind_java_type! {
    pub DialogInterfaceOnDismissListener => "android.content.DialogInterface$OnDismissListener",
}

impl_listener!(DialogInterfaceOnDismissListener);
