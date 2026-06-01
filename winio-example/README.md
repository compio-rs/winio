# How to Build the Example Program

Install [cargo-apk2](https://github.com/mzdk100/cargo-apk2):
```shell
cargo install cargo-apk2
```

**Note**: Currently, only the cargo-apk2 (version >= 1.3.0) tool is supported for building.

```shell
adb logcat -c
cargo apk2 run --example hello
```

Under the hood, your `main()` will be called on the creation of `rs.compio.winio.Activity`.

The `#[no_mangle]` on `main` therefore is needed, and DO NOT change it to `android_main`.
