use jni::{Env, errors::Result, objects::JCharSequence};

use crate::java::lang::JCharSequence2;

pub(crate) trait JCharSequenceExt {
    fn try_to_string(self, env: &mut Env<'_>) -> Result<String>;
}

impl<'a> JCharSequenceExt for JCharSequence<'a> {
    fn try_to_string(self, env: &mut Env<'_>) -> Result<String> {
        let str = unsafe { JCharSequence2::from_raw(env, self.into_raw()) };
        str.to_string(env)?.try_to_string(env)
    }
}

macro_rules! impl_listener {
    ($ty:ident) => {
        impl<'local> core::convert::AsRef<$ty<'local>> for jni_min_helper::DynamicProxy {
            fn as_ref(&self) -> &$ty<'local> {
                unsafe {
                    core::mem::transmute(core::convert::AsRef::<jni::objects::JObject>::as_ref(
                        self,
                    ))
                }
            }
        }
    };
}

pub(crate) use impl_listener;
