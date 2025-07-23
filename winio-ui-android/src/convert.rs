use {
    jni::{
        JNIEnv,
        errors::Result,
        objects::{JDoubleArray, JObject, JString},
    },
    winio_primitive::{Point, Size},
};

pub trait JObjectExt<O> {
    fn to(self, env: &mut JNIEnv) -> Result<O>;
}

impl<'a> JObjectExt<String> for JObject<'a> {
    fn to(self, env: &mut JNIEnv) -> Result<String> {
        let s = JString::from(self);
        let ret = env.get_string(&s)?;
        let Ok(ret) = ret.to_str() else {
            return Ok(Default::default());
        };
        Ok(ret.to_string())
    }
}

impl<'a> JObjectExt<Size> for JObject<'a> {
    fn to(self, env: &mut JNIEnv) -> Result<Size> {
        let a = JDoubleArray::from(self);
        let mut buf = [0f64; 2];
        env.get_double_array_region(a, 0, &mut buf)?;

        Ok(Size {
            width: buf[0],
            height: buf[1],
            _unit: Default::default(),
        })
    }
}

impl<'a> JObjectExt<Point> for JObject<'a> {
    fn to(self, env: &mut JNIEnv) -> Result<Point> {
        let a = JDoubleArray::from(self);
        let mut buf = [0f64; 2];
        env.get_double_array_region(a, 0, &mut buf)?;

        Ok(Point {
            x: buf[0],
            y: buf[1],
            _unit: Default::default(),
        })
    }
}
