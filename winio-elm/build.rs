use cfg_aliases::cfg_aliases;

fn main() {
    cfg_aliases! {
        menu: { any(windows, target_os = "macos", not(any(target_os = "android", all(target_vendor = "apple", not(target_os = "macos"))))) },
    }
}
