use {
    jni::{
        JNIEnv,
        objects::{JObject,GlobalRef},
        sys::{JNI_VERSION_1_6, jint},
    },
    std::{
        env::set_var,
        fs::File,
        io::{BufRead, BufReader},
        os::{raw::c_void, unix::prelude::*},
        thread::spawn,
    },
};


#[derive(Clone, Debug)]
pub enum ActivityMessage {
    Created,
    Start,
    Resume,
    Pause,
    Stop,
    Destroy,
    ConfigChanged(GlobalRef),
    LowMemory,
}

fn wake(_sender: JObject, _event: ActivityMessage) {
}

#[unsafe(no_mangle)]
extern "C" fn Java_rs_compio_winio_onStart(_env: JNIEnv, activity: JObject) {
    wake(activity, ActivityMessage::Start);
}

#[unsafe(no_mangle)]
extern "C" fn Java_rs_compio_winio_onResume(_env: JNIEnv, activity: JObject) {
    wake(activity, ActivityMessage::Resume);
}

#[unsafe(no_mangle)]
extern "C" fn Java_rs_compio_winio_onPause(_env: JNIEnv, activity: JObject) {
    wake(activity, ActivityMessage::Pause);
}

#[unsafe(no_mangle)]
extern "C" fn Java_rs_compio_winio_onStop(_env: JNIEnv, activity: JObject) {
    wake(activity, ActivityMessage::Stop);
}

#[unsafe(no_mangle)]
extern "C" fn Java_rs_compio_winio_on_destroy(_env: JNIEnv, activity: JObject) {
    wake(activity, ActivityMessage::Destroy);
}

#[unsafe(no_mangle)]
extern "C" fn Java_rs_compio_winio_on_configuration_changed(env: JNIEnv, activity: JObject, new_config: JObject) {
    wake(activity, ActivityMessage::ConfigChanged(env.new_global_ref(new_config).unwrap()));
}

#[unsafe(no_mangle)]
extern "C" fn Java_rs_compio_winio_Activity_on_low_memory(_env: JNIEnv, activity: JObject) {
    wake(activity, ActivityMessage::LowMemory);
}

#[unsafe(no_mangle)]
extern "C" fn Java_rs_compio_winio_Activity_on_create(_env: JNIEnv, activity: JObject) {
    wake(activity, ActivityMessage::Created)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn JNI_OnLoad(_: *mut jni::JavaVM, _: *mut c_void) -> jint {
    #[cfg(debug_assertions)]
    unsafe {
        set_var("RUST_BACKTRACE", "full");
    }

    let mut log_pipe: [RawFd; 2] = Default::default();
    unsafe {
        libc::pipe(log_pipe.as_mut_ptr());
        libc::dup2(log_pipe[1], libc::STDOUT_FILENO);
        libc::dup2(log_pipe[1], libc::STDERR_FILENO);
    }
    spawn(move || unsafe {
        let file = File::from_raw_fd(log_pipe[0]);
        let mut reader = BufReader::new(file);
        let mut buffer = String::new();
        loop {
            buffer.clear();
            if let Ok(len) = reader.read_line(&mut buffer) {
                if len == 0 {
                    break;
                }
                log::info!("{}", buffer);
            }
        }
    });

    unsafe extern "system" {
        fn main();
    }
    spawn(|| unsafe { main() });

    JNI_VERSION_1_6
}
