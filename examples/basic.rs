use windows_sys::{
    w,
    Win32::UI::WindowsAndMessaging::{MessageBoxW, IDYES, MB_YESNO},
};
use winio::{
    block_on,
    window::{AsRawWindow, Window},
};

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(compio_log::Level::TRACE)
        .init();

    block_on(async {
        let window = Window::new().await.unwrap();
        loop {
            window.close().await;
            if unsafe {
                MessageBoxW(
                    window.as_raw_window(),
                    w!("Close window?"),
                    w!("Basic example"),
                    MB_YESNO,
                )
            } == IDYES
            {
                window.destory().await.unwrap();
                break;
            }
        }
    })
}
