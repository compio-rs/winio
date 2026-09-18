# AGENTS.md

This file provides guidance for AI agents working in this repository.

## Project overview

winio is a single-threaded asynchronous GUI framework for Rust, based on [compio](https://github.com/compio-rs/compio) (an async runtime built on io-uring and IOCP) and an ELM-style architecture. It provides native widgets across desktop and mobile platforms through a common API.

The framework follows the Elm Architecture:

- **Components** (`winio-elm::Component`) hold UI state and react to **messages**; they emit **events** to their parent.
- Messages flow down the component tree; events flow up.
- A component owns its children as `Child<T>` fields.
- `Root<T>` runs a component tree and yields events.

## Repository layout

This is a Cargo workspace with these crates:

| Crate                     | Purpose                                                                                                           |
| ------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| `winio`                   | The public API: widgets, UI helpers, backend selection (`sys`)                                                    |
| `winio-elm`               | ELM primitives: `Component`, `Child`, `Root`, `Prop`/`PropSource`, macros (`init!`, `start!`, `update_children!`) |
| `winio-primitive`         | Basic types: `Point`, `Size`, `Rect`, `Color`, `Font`, `HAlign`, `VAlign`, ...                                    |
| `winio-layout`            | Layout containers (`Grid`, `StackPanel`) and the `layout!` macro                                                  |
| `winio-handle`            | Native handle abstractions (`AsWindow`, `AsWidget`, `AsContainer`)                                                |
| `winio-callback`          | Callback helpers                                                                                                  |
| `winio-pollable`          | Runtime-polling helpers                                                                                           |
| `winio-example`           | The example application, exercising all widgets                                                                   |
| `winio-ui-qt`             | Qt backend                                                                                                        |
| `winio-ui-gtk`            | GTK 4 backend                                                                                                     |
| `winio-ui-app-kit`        | macOS (AppKit) backend                                                                                            |
| `winio-ui-ui-kit`         | iOS (UIKit) backend                                                                                               |
| `winio-ui-win32`          | Win32 backend                                                                                                     |
| `winio-ui-winui`          | WinUI 3 backend                                                                                                   |
| `winio-ui-android`        | Android backend                                                                                                   |
| `winio-ui-stub`           | No-op stub backend (fallback)                                                                                     |
| `winio-ui-apple-common`   | Shared code for the Apple backends                                                                                |
| `winio-ui-windows-common` | Shared code for the Windows backends                                                                              |

The active backend is selected in `winio/src/lib.rs` with features and `cfg` attributes (e.g. `win32` feature *on Windows* prefers `winio-ui-win32` over `winio-ui-winui`, and `qt` feature *on Linux* prefers `winio-ui-qt` over `winio-ui-gtk`). `sys` is the platform backend alias used throughout the `winio` crate. On Windows or Linux, if a backend is not selected by the feature, `winio-ui-stub` will be the fallback.

## Common commands

Commands are grouped by host OS.

The code should be formatted after changes.
```sh
cargo fmt
```

WinUI bindings are manually generated and published. If a WinRT API is missing, report it instead of working around it.

### Linux

The reference environment for this checkout is WSL, with Qt 6 dev packages, `cargo xwin` (Windows), `cargo ndk` (`ANDROID_NDK_HOME`, Android) and the Apple targets (through `osxcross`) installed.

```sh
# Default platform (Qt backend on Linux)
cargo clippy --workspace --features all,nightly

# GTK backend
cargo clippy --no-default-features --features all,gtk,nightly

# Stub backend
cargo clippy --no-default-features --workspace --features all,nightly

# Win32 (MSVC target via xwin)
cargo xwin clippy --target x86_64-pc-windows-msvc --workspace --features all,nightly,windows-dark-mode

# Win32 (MSVC target via xwin)
cargo xwin clippy --target x86_64-pc-windows-msvc --no-default-features --workspace --features all,nightly,windows-dark-mode,winui,winui-enable-cbs,winui-webview-system,winui-content-dialog

# macOS
cargo clippy --target x86_64-apple-darwin --workspace --features all,nightly

# iOS (Mac Catalyst)
cargo clippy --target x86_64-apple-ios-macabi --workspace --features all,nightly

# Android
cargo ndk -t x86_64 clippy -p winio-example --features all,nightly
```

Cross-compiled Windows executables are run by the user on the Windows host; do not use `wine`.

### Windows

Build with the MSVC Rust toolchain (`x86_64-pc-windows-msvc`). Win32 is the default backend. WinUI must be opted in and is mutually exclusive with `win32`; the `winui-*` sub-features are `winio` features.

```sh
# Win32 (default)
cargo clippy --workspace --features all,nightly,windows-dark-mode

# WinUI 3
cargo clippy --workspace --no-default-features --features all,nightly,windows-dark-mode,winui,winui-enable-cbs,winui-webview-system,winui-content-dialog

# Stub backend
cargo clippy --no-default-features --workspace --features all,nightly

# Android
cargo ndk -t x86_64 clippy -p winio-example --features all,nightly
```

### macOS

AppKit is selected automatically on macOS; UIKit is selected automatically on iOS and Mac Catalyst.

```sh
# macOS
cargo clippy --workspace --features all,nightly

# iOS / Mac Catalyst
cargo clippy -p winio --target aarch64-apple-ios-macabi

# Android
cargo ndk -t arm64-v8a clippy -p winio-example --features all,nightly
```

### Tests

```sh
cargo test
```

## Cross-backend API changes

A public `winio` API usually forwards to `sys` (`winio/src/ui/*.rs`) and must be implemented in **every** backend crate. A host build only compiles the host's own backend (Qt or GTK on Linux, Win32/WinUI on Windows, AppKit on macOS), so after changing `winio/src/lib.rs` or a `sys` interface, run the matching checks for every backend you touched; from Linux, the cross-checks above are usually the cheapest way to cover Windows and Apple.

## Widget implementation patterns

- Widgets live in `winio/src/widgets/<name>.rs`; each one implements `Component` and is also implemented in every backend under `winio-ui-*/src/widgets/`.
- Properties are exposed as:
  - `Set*` messages (`SetText(String)`, `SetPos(usize)`, ...) — the public API, can be sent by users.
  - `Change*` messages — internal, driven by native input events; mark them `#[doc(hidden)]`.
- Setter pattern (compare against the native value, apply only on change):

  ```rust
  pub fn set_pos(&mut self, v: usize) -> Result<()> {
      if v != self.pos()? {
          self.widget.set_pos(v)?;
          self.pos_prop.notify(v);
      }
      Ok(())
  }
  ```

- Input events: `start()` loops over `wait_click()`/`wait_change()`/... and posts a `Change*` message; the message handler reads the native value, notifies the bound `PropSource`, and emits the event.
- User-observable values (e.g. `font()`) must be read back from the native widget, not from a stored copy.
- Shared conversion helpers between widgets of the same backend go in the backend's `widgets/mod.rs` (or the crate's shared module) as `pub(crate)` functions, e.g. `text_block_to_font` / `font_to_text_block`. The `Prop`/`PropSource` binding types live in `winio-elm/src/bind.rs`. `PropSource` holds listeners only; `Prop<'a, T>` is a temporary view created by `as_prop(value)`.

## Documentation requirements

Several crates enforce `#![warn(missing_docs)]` (e.g. `winio-elm`, `winio-layout`); keep their public items documented and run `cargo doc -p <crate> --no-deps` to catch broken links.

Conventions:

- Every widget type needs a one- or two-sentence summary of what it is and what it does (write it in your own words, no copied phrasing).
- Use `## Platform specific` in the doc comment of any method whose behavior differs per backend (or that returns `Error::NotSupported` on some backend).
- `layout!` / `init!` / `start!` / `update_children!` macro docs describe the entry syntax, attributes support (`#[cfg(...)]`), and usage in the component lifecycle.

## Backend-specific notes

- **Qt** (`winio-ui-qt`): C++ helpers live in `*.hpp`/`*.cpp` next to the Rust files; cxx bridges are declared in the `ffi` module of the `.rs` file.
- **GTK** (`winio-ui-gtk`): `Adjustment::set_value` emits `value-changed` synchronously; guard against feedback loops when binding slider positions.
- **Windows** (`winio-ui-win32`, `winio-ui-winui`): DPI handling via  `platform/dpi.rs` (`to_logical`/`to_device`). `refresh_font` resets fonts on DPI change; custom fonts are tracked in a thread-local `HWND -> (Font, WinFont)` map so they can be recreated with the new DPI (see `winio-ui-win32/src/platform/font.rs`). Common abstractions are put into `winio-ui-windows-common`.
- **Android** (`winio-ui-android`): JNI bindings use `jni::bind_java_type!` in `src/java/android/`; the entry point is `android_main(app: AndroidApp)`; use `vm_exec` for JNI calls.
- **Apple** (`winio-ui-app-kit`, `winio-ui-ui-kit`): Common abstractions are put into `winio-ui-apple-common`.
- **Stub** (`winio-ui-stub`): unimplemented methods use `not_impl!()` (panics).

## Code style

- 4-space indentation, `rustfmt` (see `rustfmt.toml`).
- Error handling via the crate-specific `Result` aliases and the `syscall!` macro on Windows.
- Keep changes minimal and focused; do not rewrite files wholesale when a targeted edit suffices.
