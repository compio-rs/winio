use {
    super::RUNTIME,
    android_activity::AndroidApp,
    futures_util::task::noop_waker,
    jni::{AttachGuard, JavaVM, errors::Result as JniResult,objects::JObject},
    std::{
        future::Future,
        task::{Context, Poll},
        time::Duration,
    },
};

pub struct Runtime {
    app: Option<AndroidApp>,
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl Runtime {
    pub fn new() -> Self {
        Self { app: None }
    }

    pub fn set_android_app(&mut self, android_app: AndroidApp) {
        self.app = Some(android_app);
    }

    pub fn get_android_app(&self) -> Option<&AndroidApp> {
        self.app.as_ref()
    }

    pub(crate) fn vm_exec<F, R>(&self, f: F) -> JniResult<R>
    where
        F: FnOnce(AttachGuard, JObject) -> JniResult<R>,
    {
        let Some(ref app) = self.app else {
            panic!(
                "There is no instance of AndroidApp provided, please use the set_android_app method to set it up first."
            );
        };
        let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr() as _)? };
        let env = vm.attach_current_thread()?;
        let activity = unsafe { JObject::from_raw(app.activity_as_ptr() as _) };
        f(env, activity)
    }

    fn enter<T, F: FnOnce() -> T>(&self, f: F) -> T {
        RUNTIME.set(self, f)
    }

    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.enter(|| {
            let mut future = Box::pin(future);
            let waker = noop_waker();
            let mut cx = Context::from_waker(&waker);
            loop {
                match future.as_mut().poll(&mut cx) {
                    Poll::Ready(val) => return val,
                    Poll::Pending => {
                        // Wait for event
                        if let Some(ref app) = self.app {
                            app.poll_events(Some(Duration::from_millis(10)), |_| ());
                        }
                    }
                }
            }
        })
    }
}
