use std::{
    os::fd::{BorrowedFd, FromRawFd, IntoRawFd},
    path::Path,
};

use compio::fs::File;

use crate::{Result, current_activity, vm_exec};

jni::bind_java_type! {
    Context2 => android.content.Context,
    type_map {
        ContentResolver => android.content.ContentResolver,
    },
    methods {
        fn get_content_resolver() -> ContentResolver,
    }
}

jni::bind_java_type! {
    Uri => android.net.Uri,
    methods {
        static fn parse(uri: JString) -> Uri,
    }
}

jni::bind_java_type! {
    ContentResolver => android.content.ContentResolver,
    type_map {
        ParcelFileDescriptor => android.os.ParcelFileDescriptor,
        Uri => android.net.Uri,
    },
    methods {
        fn open_file_descriptor(uri: Uri, mode: JString) -> ParcelFileDescriptor,
    }
}

jni::bind_java_type! {
    ParcelFileDescriptor => android.os.ParcelFileDescriptor,
    methods {
        fn get_fd() -> jint,
    }
}

pub use compio::fs::File as UriFile;

pub async fn open_uri(uri: &Path) -> Result<File> {
    vm_exec(|env| {
        let act = current_activity(env)?;
        let context = env.cast_local::<Context2>(act)?;
        let resolver = context.get_content_resolver(env)?;
        let uri = env.new_string(uri.to_string_lossy())?;
        let uri = Uri::parse(env, uri)?;
        let mode = env.new_string("r")?;
        let pfd = resolver.open_file_descriptor(env, uri, mode)?;
        let fd = pfd.get_fd(env)?;
        let fd = unsafe { BorrowedFd::borrow_raw(fd) };
        let fd = fd.try_clone_to_owned()?;
        Result::Ok(unsafe { File::from_raw_fd(fd.into_raw_fd()) })
    })
}
