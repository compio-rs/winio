//! Icon graphics from [Twemoji](https://github.com/jdecked/twemoji), licensed
//! under [CC-BY 4.0](https://creativecommons.org/licenses/by/4.0/), and
//! [Octicons](https://github.com/primer/octicons), licensed under
//! [MIT](https://opensource.org/license/mit).

use winio::prelude::*;

use crate::Result;

pub fn github() -> Result<Image> {
    load(include_bytes!("../assets/icons/github.png"))
}

pub fn github_white() -> Result<Image> {
    load(include_bytes!("../assets/icons/github_white.png"))
}

#[cfg(feature = "media")]
pub fn play() -> Result<Image> {
    load(include_bytes!("../assets/icons/play.png"))
}

#[cfg(feature = "media")]
pub fn pause() -> Result<Image> {
    load(include_bytes!("../assets/icons/pause.png"))
}

#[cfg(feature = "webview")]
pub fn go() -> Result<Image> {
    load(include_bytes!("../assets/icons/go.png"))
}

#[cfg(feature = "webview")]
pub fn back() -> Result<Image> {
    load(include_bytes!("../assets/icons/back.png"))
}

#[cfg(feature = "webview")]
pub fn forward() -> Result<Image> {
    load(include_bytes!("../assets/icons/forward.png"))
}

#[cfg(feature = "webview")]
pub fn reload() -> Result<Image> {
    load(include_bytes!("../assets/icons/reload.png"))
}

#[cfg(feature = "webview")]
pub fn stop() -> Result<Image> {
    load(include_bytes!("../assets/icons/stop.png"))
}

fn load(bytes: &[u8]) -> Result<Image> {
    Ok(Image::new(image::load_from_memory(bytes)?)?)
}
