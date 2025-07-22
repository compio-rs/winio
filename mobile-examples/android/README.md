# How to Build the Example Program

Install [cargo-apk2](https://github.com/mzdk100/cargo-apk2):
```shell
cargo install cargo-apk2
```

**Note**: Currently, only the cargo-apk2 (version >= 1.2.0) tool is supported for building.

```shell
adb logcat -c
cargo apk2 run -p winio-android-example
```