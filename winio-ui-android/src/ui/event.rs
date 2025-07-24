#[macro_export]
macro_rules! define_event {
    ($var_name: ident, $fn_name: ident) => {
        static $var_name: std::sync::LazyLock<
            std::sync::Mutex<std::collections::HashMap<i32, Vec<oneshot::Sender<()>>>>,
        > = std::sync::LazyLock::new(Default::default);

        #[allow(non_snake_case)]
        #[unsafe(no_mangle)]
        extern "C" fn $fn_name(env: jni::JNIEnv, obj: jni::objects::JObject) {
            let Ok(w) = env.new_global_ref(obj) else {
                return;
            };
            let w: BaseWidget = w.into();

            if let Ok(mut lock) = $var_name.lock()
                && let Some(mut senders) = lock.remove(&w.hash_code())
            {
                drop(lock);
                while let Some(sender) = senders.pop() {
                    let _ = sender.send(());
                }
            }
        }
    };
}

#[macro_export]
macro_rules! recv_event {
    ($widget: expr, $var_name: ident) => {
        if let Ok(mut lock) = $var_name.lock() {
            let (tx, rx) = oneshot::channel();
            let hash_code = $widget.inner.hash_code();
            if let Some(senders) = lock.get_mut(&hash_code) {
                senders.push(tx);
            } else {
                lock.insert(hash_code, vec![tx]);
            }
            drop(lock);
            let _ = rx.await;
        }
    };
}
