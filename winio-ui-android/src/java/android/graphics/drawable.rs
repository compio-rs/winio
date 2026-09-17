use super::{Bitmap, Picture};

jni::bind_java_type! {
    pub Drawable => android.graphics.drawable.Drawable,
}

jni::bind_java_type! {
    pub BitmapDrawable => android.graphics.drawable.BitmapDrawable,
    type_map {
        Drawable => android.graphics.drawable.Drawable,
        Bitmap => android.graphics.Bitmap,
    },
    constructors {
        fn new(bitmap: &Bitmap),
    },
    is_instance_of = {
        base = Drawable,
    }
}

jni::bind_java_type! {
    pub PictureDrawable => android.graphics.drawable.PictureDrawable,
    type_map {
        Drawable => android.graphics.drawable.Drawable,
        Picture => android.graphics.Picture,
    },
    constructors {
        fn new(picture: &Picture),
    },
    is_instance_of = {
        base = Drawable,
    }
}
