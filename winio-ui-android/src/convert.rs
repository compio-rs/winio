use jni::{Env, errors::Result, objects::JCharSequence};

jni::bind_java_type! {
    JCharSequence2 => java.lang.CharSequence,
    methods {
        fn to_string() -> JString,
    }
}

pub(crate) trait JCharSequenceExt {
    fn try_to_string(self, env: &mut Env<'_>) -> Result<String>;
}

impl<'a> JCharSequenceExt for JCharSequence<'a> {
    fn try_to_string(self, env: &mut Env<'_>) -> Result<String> {
        let str = unsafe { JCharSequence2::from_raw(env, self.into_raw()) };
        str.to_string(env)?.try_to_string(env)
    }
}
