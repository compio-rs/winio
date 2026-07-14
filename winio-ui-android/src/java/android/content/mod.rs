pub mod res;

use super::{
    content::res::{Resources, ResourcesTheme},
    database::Cursor,
    net::Uri,
    os::ParcelFileDescriptor,
};
use crate::impl_listener;

jni::bind_java_type! {
    pub Context => android.content.Context,
    type_map {
        Resources => android.content.res.Resources,
        ResourcesTheme => "android.content.res.Resources$Theme",
    },
    methods {
        fn get_resources() -> Resources,
        fn get_theme() -> ResourcesTheme,
    }
}

jni::bind_java_type! {
    pub Context2 => android.content.Context,
    type_map {
        ContentResolver => android.content.ContentResolver,
    },
    methods {
        fn get_content_resolver() -> ContentResolver,
    }
}

jni::bind_java_type! {
    pub ContentResolver => android.content.ContentResolver,
    type_map {
        ParcelFileDescriptor => android.os.ParcelFileDescriptor,
        Uri => android.net.Uri,
        Cursor => android.database.Cursor,
    },
    methods {
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
