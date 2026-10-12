use cfg_aliases::cfg_aliases;

fn main() {
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if os == "windows" {
        match env.as_str() {
            "msvc" => {
                println!("cargo:rustc-link-arg=/STACK:8388608");
            }
            "gnu" | "gnullvm" => {
                println!("cargo:rustc-link-arg=-Wl,--stack,8388608");
            }
            _ => {}
        }
    }
    cfg_aliases! {
        menu: { any(windows, target_os = "macos", not(any(target_os = "android", all(target_vendor = "apple", not(target_os = "macos"))))) },
    }
}
