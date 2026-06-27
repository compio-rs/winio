# Winio

Winio is a single-threaded asynchronous GUI runtime.
The GUI part is powered by native backends, and it is compatible with [`compio`](https://github.com/compio-rs/compio).
All IO requests could be issued in the same thread as GUI, without blocking the user interface!

## Platform support

| Backend | Platform                                                           |
| ------- | ------------------------------------------------------------------ |
| Win32   | Windows 10 1607+ <br> Windows 10 1809+ (`windows-dark-mode`)       |
| WinUI   | Windows 11 21H2+ <br> WinUI (3) 1.0+ <br> WinUI (3) 1.2+ (`media`) |
| Qt      | Qt 5.15+ <br> Qt 6.0+                                              |
| GTK     | GTK 4.14+ <br> WebkitGtk 6 2.42+ (`webview`)                       |
| AppKit  | macOS 11.0+                                                        |
| UIKit   | iOS 13.0+ <br> Mac Catalyst 13.1+                                  |
| Android | Android SDK 36+ <br> Android NDK 27+                               |

> [!WARNING] Selecting backends
> On systems other than macOS, iOS, and Android, you have to select only one backend by enabling features. The default one is `win32` for Windows and `qt` for others.

> [!WARNING] WGPU support
> Some platforms doesn't work well:
> * iOS simulator
> * Mac Catalyst
> * Android simulator
> * Qt
> * GTK

## Example

Read the [example](winio-example) and learn more!

| Backend              | Light                                        | Dark                                       |
| -------------------- | -------------------------------------------- | ------------------------------------------ |
| Win32                | ![Win32 Light](assets/win32.light.png)       | ![Win32 Dark](assets/win32.dark.png)       |
| WinUI 3              | ![WinUI Light](assets/winui.light.png)       | ![WinUI Dark](assets/winui.dark.png)       |
| Qt 6                 | ![Qt Light](assets/qt.light.png)             | ![Qt Dark](assets/qt.dark.png)             |
| GTK 4                | ![GTK Light](assets/gtk.light.png)           | ![GTK Dark](assets/gtk.dark.png)           |
| AppKit               | ![macOS Light](assets/mac.light.png)         | ![macOS Dark](assets/mac.dark.png)         |
| UIKit (Mac Catalyst) | ![Catalyst Light](assets/catalyst.light.png) | ![Catalyst Dark](assets/catalyst.dark.png) |
| UIKit (iOS)          | ![iOS Light](assets/ios.light.png)           | ![iOS Dark](assets/ios.dark.png)           |
| Android View         | ![Android Light](assets/android.light.png)   | ![Android Dark](assets/android.dark.png)   |

## Quick start

Winio follows ELM-like design, inspired by [`yew`](https://yew.rs/) and [`relm4`](https://relm4.org/).
The application starts with a root `Component`:

```rust
use winio::prelude::*;

struct MainModel {
    window: Child<Window>,
}

enum MainMessage {
    Noop,
    Close,
}

impl Component for MainModel {
    type Error = Error;
    type Event = ();
    type Init<'a> = ();
    type Message = MainMessage;

    async fn init(_init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        // create & initialize the window
        init! {
            window: Window = (()) => {
                text: "Example",
                size: Size::new(800.0, 600.0),
            }
        }
        window.show()?;
        Ok(Self { window })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        // listen to events
        start! {
            sender, default: MainMessage::Noop,
            self.window => {
                WindowEvent::Close => MainMessage::Close,
            }
        }
    }

    async fn update_children(&mut self) -> Result<bool> {
        // update the window
        update_children!(self.window)
    }

    async fn update(&mut self, message: Self::Message, sender: &ComponentSender<Self>) -> Result<bool> {
        // deal with custom messages
        match message {
            MainMessage::Noop => Ok(false),
            MainMessage::Close => {
                // the root component output stops the application
                sender.output(());
                // need not to call `render`
                Ok(false)
            }
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) -> Result<()> {
        let csize = self.window.client_size()?;
        // adjust layout and draw widgets here
        Ok(())
    }

    fn render_children(&mut self) -> Result<()> {
        self.window.render()
    }
}
```

It is recommended to set the lib name to "main" for convenience.
```toml
[lib]
name = "main"
```

All platforms except Android start with `main`. You should add the code below to `main.rs`:
```rust
#[cfg(not(target_os = "android"))]
fn main() -> winio::Result<()> {
    use main::MainModel;
    use winio::prelude::*;

    App::builder()
        .name("rs.compio.winio.example")
        .build()?
        .block_on(MainModel::run_until_event(()))
}

#[cfg(target_os = "android")]
fn main() {
    unreachable!("Android entry point is `android_main` in `android.rs`")
}
```
> [!NOTE] iOS notes
> `WindowEvent::Close` will never be emitted, and the application will exit if the window (Mac Catalyst) or the app (iOS) closes. `block_on` doesn't return in that case.

The Android entry point is the `android_main` method:
```rust
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    let app = App::builder()
        .android_app(app)
        .build()
        .expect("cannot create app");
    app.spawn(|| async {
        if let Err(e) = MainModel::run_until_event(()).await {
            compio_log::error!("App error: {e:?}");
        }
    })
}
```
> [!NOTE] Android notes
> * `android_main` might be called multiple times, but the lifetime of each calling don't overlap.
> * `android_main` runs on a dedicate thread, while all code of `winio` execute on the main thread.
> * You have to do the following to create a complete Android project with `winio`.

## Integrate into an Android app
To integrate the `winio` app into an Android app with id "rs.compio.winio.example", the main activity should inherit `rs.compio.winio.Activity`:
```java
package rs.compio.winio.example;

import rs.compio.winio.Activity;

public class MainActivity extends Activity {
    static {
        System.loadLibrary("main");
    }
}
```
Add the following to the `<activity>` section of `AndroidManifest.xml`:
```xml
<meta-data android:name="android.app.lib_name" android:value="main" />
```
Put the project folder "android" beside the "src" folder of the rust project, and modify the "dependencyResolutionManagement" part of `android/settings.gradle`
```gradle
dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
        maven {
            url = findWinioUiAndroidProject()
            metadataSources.artifact()
        }
    }
}
// ...
import groovy.json.JsonSlurper

String findWinioUiAndroidProject() {
    def dependencyText = providers.exec {
        it.workingDir = new File("../")
        commandLine("cargo", "metadata", "--format-version", "1", "--manifest-path", "Cargo.toml")
    }.standardOutput.asText.get()

    def dependencyJson = new JsonSlurper().parseText(dependencyText)
    def manifestPath = file(dependencyJson.packages.find { it.name == "winio-ui-android" }.manifest_path)
    return new File(manifestPath.parentFile, "maven").path
}
```
and `gradle/libs.versions.toml`
```toml
[libraries]
# ...
winio = { module = "compio:winio", version = "latest.release" }
```
and `android/app/build.gradle`
```gradle
dependencies {
    // ...

    implementation libs.winio
}

[
        Debug  : null,
        Profile: '--release',
        Release: '--release'
].each {
    def taskPostfix = it.key
    def profileMode = it.value
    tasks.configureEach { task ->
        if (task.name == "assemble$taskPostfix") {
            task.dependsOn "cargoBuild$taskPostfix"
        }
    }
    tasks.register("cargoBuild$taskPostfix", Exec) {
        workingDir "../.."
        environment 'CARGO_TERM_COLOR', 'always'

        def cmdArgs = [
                'cargo', 'ndk',
                '-o', 'android/app/src/main/jniLibs',
                '-t', 'arm64-v8a',
                '-t', 'armeabi-v7a',
                '-t', 'x86_64',
                'rustc',
                '--crate-type', 'cdylib', '--lib',
        ]
        if (profileMode != null) {
            cmdArgs.add(profileMode)
        }

        commandLine cmdArgs
    }
}
```
