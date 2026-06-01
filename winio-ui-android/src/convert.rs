use jni::{
    Env,
    objects::{JCharSequence, JObject, JString},
};

use crate::Result;

pub trait JObjectExt<O> {
    fn to(self, env: &mut Env<'_>) -> Result<O>;
}

impl<'a> JObjectExt<String> for JObject<'a> {
    fn to(self, env: &mut Env<'_>) -> Result<String> {
        let s = unsafe { JString::from_raw(env, self.into_raw()) };
        Ok(s.try_to_string(env)?)
    }
}

impl<'a> JObjectExt<String> for JCharSequence<'a> {
    fn to(self, env: &mut Env<'_>) -> Result<String> {
        let s = unsafe { JString::from_raw(env, self.into_raw()) };
        Ok(s.try_to_string(env)?)
    }
}

impl<'a> JObjectExt<String> for JString<'a> {
    fn to(self, env: &mut Env<'_>) -> Result<String> {
        Ok(self.try_to_string(env)?)
    }
}
