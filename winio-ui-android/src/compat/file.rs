use std::{
    ffi::OsString,
    os::fd::{BorrowedFd, FromRawFd, IntoRawFd},
    path::{Path, PathBuf},
};

use compio::fs::File;
use jni::{
    objects::{JObjectArray, JString},
    refs::Global,
};

use crate::{
    Result, current_activity,
    java::android::{
        content::Context,
        database::Cursor,
        net::Uri,
        provider::{DocumentsContract, DocumentsContractDocument},
    },
    vm_exec,
};

fn open_uri_with_mode(uri: &Path, mode: &str) -> Result<File> {
    vm_exec(|env| {
        let act = current_activity(env)?;
        let context = env.cast_local::<Context>(act)?;
        let resolver = context.get_content_resolver(env)?;
        let uri = env.new_string(uri.to_string_lossy())?;
        let uri = Uri::parse(env, uri)?;
        let mode = env.new_string(mode)?;
        let pfd = resolver.open_file_descriptor(env, uri, mode)?;
        let fd = pfd.get_fd(env)?;
        let fd = unsafe { BorrowedFd::borrow_raw(fd) };
        let fd = fd.try_clone_to_owned()?;
        Result::Ok(unsafe { File::from_raw_fd(fd.into_raw_fd()) })
    })
}

pub use compio::fs::File as UriFile;

pub async fn open_uri(uri: &Path) -> Result<File> {
    open_uri_with_mode(uri, "r")
}

pub async fn create_uri(uri: &Path) -> Result<File> {
    open_uri_with_mode(uri, "w")
}

pub async fn update_uri(uri: &Path) -> Result<File> {
    open_uri_with_mode(uri, "rw")
}

#[derive(Debug)]
pub struct UriDirEntry {
    name: String,
    uri: PathBuf,
    mime: String,
}

impl UriDirEntry {
    pub fn path(&self) -> PathBuf {
        self.uri.clone()
    }

    pub fn file_name(&self) -> OsString {
        OsString::from(&self.name)
    }

    pub fn file_type(&self) -> Result<UriFileType> {
        Ok(UriFileType {
            is_dir: self.mime == "vnd.android.document/directory",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UriFileType {
    is_dir: bool,
}

impl UriFileType {
    pub fn is_dir(&self) -> bool {
        self.is_dir
    }

    pub fn is_file(&self) -> bool {
        !self.is_dir
    }

    pub fn is_symlink(&self) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct UriReadDir {
    tree_uri: Global<Uri<'static>>,
    cursor: Global<Cursor<'static>>,
}

pub fn read_dir(uri: &Path) -> Result<UriReadDir> {
    vm_exec(|env| {
        let act = current_activity(env)?;
        let context = env.cast_local::<Context>(act)?;
        let resolver = context.get_content_resolver(env)?;
        let uri = env.new_string(uri.to_string_lossy())?;
        let uri = Uri::parse(env, uri)?;
        let docid = DocumentsContract::get_tree_document_id(env, &uri)?;
        let children_uri =
            DocumentsContract::build_child_documents_uri_using_tree(env, &uri, docid)?;
        let proj = [
            DocumentsContractDocument::COLUMN_DOCUMENT_ID(env)?,
            DocumentsContractDocument::COLUMN_DISPLAY_NAME(env)?,
            DocumentsContractDocument::COLUMN_MIME_TYPE(env)?,
        ];
        let projection = JObjectArray::<JString>::new(env, proj.len(), JString::null())?;
        for (i, col) in proj.into_iter().enumerate() {
            projection.set_element(env, i as _, col)?;
        }
        let cursor = resolver.query(
            env,
            children_uri,
            &projection,
            JString::null(),
            JObjectArray::<JString>::null(),
            JString::null(),
        )?;
        Ok(UriReadDir {
            tree_uri: env.new_global_ref(uri)?,
            cursor: env.new_global_ref(cursor)?,
        })
    })
}

impl Iterator for UriReadDir {
    type Item = Result<UriDirEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor.is_null() {
            return None;
        }
        vm_exec(|env| {
            let next = self.cursor.move_to_next(env)?;
            if !next {
                return Ok(None);
            }

            let docid_column = DocumentsContractDocument::COLUMN_DOCUMENT_ID(env)?;
            let docid_index = self.cursor.get_column_index(env, docid_column)?;
            let docid = self.cursor.get_string(env, docid_index)?;

            let mime_column = DocumentsContractDocument::COLUMN_MIME_TYPE(env)?;
            let mime_index = self.cursor.get_column_index(env, mime_column)?;
            let mime = self
                .cursor
                .get_string(env, mime_index)?
                .try_to_string(env)?;

            let name_column = DocumentsContractDocument::COLUMN_DISPLAY_NAME(env)?;
            let name_index = self.cursor.get_column_index(env, name_column)?;
            let name = self
                .cursor
                .get_string(env, name_index)?
                .try_to_string(env)?;

            let uri = DocumentsContract::build_document_uri_using_tree(env, &self.tree_uri, docid)?;
            let uri = uri.to_string(env)?.try_to_string(env)?;

            Ok(Some(UriDirEntry {
                name,
                uri: PathBuf::from(uri),
                mime,
            }))
        })
        .transpose()
    }
}
