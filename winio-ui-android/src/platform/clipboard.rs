use std::{fmt::Debug, sync::Arc};

use jni::{
    objects::{Global, JObject},
    refs::{LoaderContext, Reference},
};
use jni_min_helper::DynamicProxy;
use winio_callback::SyncCallback;

use crate::{
    Error, Result, current_activity,
    java::android::content::{ClipboardManager, ClipboardManagerOnPrimaryClipChangedListener},
    platform::Data,
    runtime::AndroidRuntime,
    vm_exec,
};

pub struct Clipboard {
    manager: Global<ClipboardManager<'static>>,
    on_changed: Arc<SyncCallback>,
    listener: DynamicProxy,
}

impl Clipboard {
    pub fn new() -> Result<Self> {
        vm_exec(|env| {
            let act = current_activity(env)?;
            let name = env.new_string("clipboard")?;
            let service = act.as_context().get_system_service(env, name)?;
            if service.is_null() {
                return Err(Error::NotSupported);
            }
            let manager = unsafe { ClipboardManager::from_raw(env, service.into_raw()) };
            let manager = env.new_global_ref(manager)?;

            let on_changed = Arc::new(SyncCallback::new());
            let listener = DynamicProxy::build(
                env,
                &LoaderContext::None,
                [ClipboardManagerOnPrimaryClipChangedListener::class_name()],
                {
                    let on_changed = on_changed.clone();
                    move |_env, _method, _args| {
                        on_changed.signal::<AndroidRuntime>(());
                        Ok(JObject::null())
                    }
                },
            )?;
            manager.add_primary_clip_changed_listener(env, &listener)?;

            Ok(Self {
                manager,
                on_changed,
                listener,
            })
        })
    }

    pub async fn read(&mut self) -> Result<Option<Data>> {
        vm_exec(|env| {
            let clip = self.manager.get_primary_clip(env)?;
            Data::from_clip_data(env, &clip)
        })
    }

    pub async fn write(&mut self, data: &Data) -> Result<()> {
        vm_exec(|env| data.write_to(env, &self.manager))
    }

    pub fn clear(&mut self) -> Result<()> {
        vm_exec(|env| {
            self.manager.clear_primary_clip(env)?;
            Ok(())
        })
    }

    pub async fn wait_change(&self) {
        self.on_changed.wait().await
    }
}

impl Drop for Clipboard {
    fn drop(&mut self) {
        let _ = vm_exec(|env| {
            self.manager
                .remove_primary_clip_changed_listener(env, &self.listener)
        });
    }
}

impl Debug for Clipboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Clipboard").finish_non_exhaustive()
    }
}
