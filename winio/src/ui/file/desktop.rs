use std::path::Path;

pub use compio::fs::File as UriFile;

pub async fn open_uri(uri: &Path) -> crate::Result<UriFile> {
    Ok(UriFile::open(uri).await?)
}
