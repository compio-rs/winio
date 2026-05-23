use jni::{
    Env,
    errors::Result,
    objects::{JDoubleArray, JIntArray, JObject, JString},
};
use winio_primitive::{Point, Size};

pub trait JObjectExt<O> {
    fn to(self, env: &mut Env<'_>) -> Result<O>;
}

impl<'a> JObjectExt<String> for JObject<'a> {
    fn to(self, env: &mut Env<'_>) -> Result<String> {
        let s = unsafe { JString::from_raw(env, self.into_raw()) };
        Ok(s.to_string())
    }
}

impl<'a> JObjectExt<Size> for JObject<'a> {
    fn to(self, env: &mut Env<'_>) -> Result<Size> {
        let a = unsafe { JDoubleArray::from_raw(env, self.into_raw() as _) };
        let mut buf = [0f64; 2];
        a.get_region(env, 0, &mut buf)?;

        Ok(Size {
            width: buf[0],
            height: buf[1],
            _unit: Default::default(),
        })
    }
}

impl<'a> JObjectExt<Point> for JObject<'a> {
    fn to(self, env: &mut Env<'_>) -> Result<Point> {
        let a = unsafe { JDoubleArray::from_raw(env, self.into_raw() as _) };
        let mut buf = [0f64; 2];
        a.get_region(env, 0, &mut buf)?;

        Ok(Point {
            x: buf[0],
            y: buf[1],
            _unit: Default::default(),
        })
    }
}

impl<'a> JObjectExt<(usize, usize)> for JObject<'a> {
    fn to(self, env: &mut Env<'_>) -> Result<(usize, usize)> {
        let a = unsafe { JIntArray::from_raw(env, self.into_raw() as _) };
        let mut buf = [0i32; 2];
        a.get_region(env, 0, &mut buf)?;

        Ok((buf[0] as _, buf[1] as _))
    }
}

impl<'a> JObjectExt<Option<usize>> for JObject<'a> {
    fn to(self, env: &mut Env<'_>) -> Result<Option<usize>> {
        if self.is_null() {
            return Ok(None);
        }

        Ok(Some(
            env.call_method(self, jni::jni_str!("intValue"), jni::jni_sig!("()I"), &[])?
                .i()? as _,
        ))
    }
}
