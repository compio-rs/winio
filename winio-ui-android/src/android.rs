#[cfg(feature = "once_cell_try")]
use std::sync::OnceLock;

use jni::{objects::JClassLoader, refs::Global};
use jni_min_helper::DexClassLoader;
#[cfg(not(feature = "once_cell_try"))]
use once_cell::sync::OnceCell as OnceLock;

use crate::{Result, vm_exec};

const DEX_DATA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/classes.dex"));

pub(crate) fn winio_class_loader() -> Result<&'static JClassLoader<'static>> {
    static CLASS_LOADER: OnceLock<Global<JClassLoader<'static>>> = OnceLock::new();
    CLASS_LOADER
        .get_or_try_init(|| {
            vm_exec(|env| {
                let dex_loader =
                    JClassLoader::get_system_class_loader(env)?.load_dex(env, DEX_DATA)?;
                Ok(env.new_global_ref(dex_loader)?)
            })
        })
        .map(|loader| &**loader)
}
