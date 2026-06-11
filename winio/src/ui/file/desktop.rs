use std::path::Path;

pub use compio::fs::File as UriFile;

pub async fn open_uri(uri: &Path) -> crate::Result<UriFile> {
    Ok(UriFile::open(uri).await?)
}

pub async fn create_uri(uri: &Path) -> crate::Result<UriFile> {
    Ok(UriFile::create(uri).await?)
}

pub async fn update_uri(uri: &Path) -> crate::Result<UriFile> {
    Ok(compio::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(uri)
        .await?)
}
