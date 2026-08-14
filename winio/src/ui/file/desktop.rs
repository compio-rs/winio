use std::{
    ffi::{OsStr, OsString},
    path::Path,
};

pub use compio::fs::File as UriFile;

pub async fn open_uri(uri: &OsStr) -> crate::Result<UriFile> {
    Ok(UriFile::open(Path::new(uri)).await?)
}

pub async fn create_uri(uri: &OsStr) -> crate::Result<UriFile> {
    Ok(UriFile::create(Path::new(uri)).await?)
}

pub async fn update_uri(uri: &OsStr) -> crate::Result<UriFile> {
    Ok(compio::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(Path::new(uri))
        .await?)
}

pub use std::fs::FileType as UriFileType;

#[derive(Debug)]
pub struct UriDirEntry(std::fs::DirEntry);

impl UriDirEntry {
    pub fn path(&self) -> OsString {
        self.0.path().into_os_string()
    }

    pub fn file_name(&self) -> OsString {
        self.0.file_name()
    }

    pub fn file_type(&self) -> std::io::Result<UriFileType> {
        self.0.file_type()
    }
}

#[derive(Debug)]
pub struct UriReadDir(std::fs::ReadDir);

impl Iterator for UriReadDir {
    type Item = std::io::Result<UriDirEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|res| res.map(UriDirEntry))
    }
}

pub fn read_dir(uri: &OsStr) -> std::io::Result<UriReadDir> {
    Ok(UriReadDir(std::fs::read_dir(Path::new(uri))?))
}
