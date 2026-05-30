use std::path::PathBuf;

use jni::{
    objects::{JList, JObject},
    refs::{LoaderContext, Reference},
};
use jni_min_helper::DynamicProxy;
use winio_handle::AsWindow;

use crate::{Activity, Error, Result, vm_exec};

jni::bind_java_type! {
    pub(crate) ActivityResultContract => androidx.activity.result.contract.ActivityResultContract,
}

jni::bind_java_type! {
    GetContent => "androidx.activity.result.contract.ActivityResultContracts$GetContent",
    type_map {
        ActivityResultContract => androidx.activity.result.contract.ActivityResultContract
    },
    constructors {
        fn new(),
    },
    is_instance_of = {
        base = ActivityResultContract,
    }
}

jni::bind_java_type! {
    GetMultipleContents => "androidx.activity.result.contract.ActivityResultContracts$GetMultipleContents",
    type_map {
        ActivityResultContract => androidx.activity.result.contract.ActivityResultContract
    },
    constructors {
        fn new(),
    },
    is_instance_of = {
        base = ActivityResultContract,
    }
}

jni::bind_java_type! {
    OpenDocumentTree => "androidx.activity.result.contract.ActivityResultContracts$OpenDocumentTree",
    type_map {
        ActivityResultContract => androidx.activity.result.contract.ActivityResultContract
    },
    constructors {
        fn new(),
    },
    is_instance_of = {
        base = ActivityResultContract,
    }
}

jni::bind_java_type! {
    CreateDocument => "androidx.activity.result.contract.ActivityResultContracts$CreateDocument",
    type_map {
        ActivityResultContract => androidx.activity.result.contract.ActivityResultContract
    },
    constructors {
        fn new(),
    },
    is_instance_of = {
        base = ActivityResultContract,
    }
}

jni::bind_java_type! {
    pub(crate) ActivityResultLauncher => androidx.activity.result.ActivityResultLauncher,
    methods {
        fn launch(input: &JObject),
    },
}

jni::bind_java_type! {
    Uri => android.net.Uri,
    methods {
        fn get_path() -> JString,
    },
}

#[derive(Debug, Default, Clone)]
pub struct FileBox {
    title: String,
    filename: String,
    filters: Vec<FileFilter>,
}

impl FileBox {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(&mut self, title: impl AsRef<str>) {
        self.title = title.as_ref().into();
    }

    pub fn filename(&mut self, filename: impl AsRef<str>) {
        self.filename = filename.as_ref().into();
    }

    pub fn filters(&mut self, filters: impl IntoIterator<Item = FileFilter>) {
        self.filters = filters.into_iter().collect();
    }

    pub fn add_filter(&mut self, filter: FileFilter) {
        self.filters.push(filter);
    }

    pub async fn open(self, parent: Option<impl AsWindow>) -> Result<Option<PathBuf>> {
        filebox(
            parent,
            self.title,
            self.filename,
            self.filters,
            true,
            false,
            false,
        )
        .await
        .map(|paths| paths.into_iter().next())
    }

    pub async fn open_multiple(self, parent: Option<impl AsWindow>) -> Result<Vec<PathBuf>> {
        filebox(
            parent,
            self.title,
            self.filename,
            self.filters,
            true,
            true,
            false,
        )
        .await
    }

    pub async fn open_folder(self, parent: Option<impl AsWindow>) -> Result<Option<PathBuf>> {
        filebox(
            parent,
            self.title,
            self.filename,
            self.filters,
            false,
            false,
            true,
        )
        .await
        .map(|paths| paths.into_iter().next())
    }

    pub async fn save(self, parent: Option<impl AsWindow>) -> Result<Option<PathBuf>> {
        filebox(
            parent,
            self.title,
            self.filename,
            self.filters,
            false,
            false,
            false,
        )
        .await
        .map(|paths| paths.into_iter().next())
    }
}

async fn filebox(
    parent: Option<impl AsWindow>,
    _title: String,
    filename: String,
    _filters: Vec<FileFilter>,
    open: bool,
    multiple: bool,
    folder: bool,
) -> Result<Vec<PathBuf>> {
    let mut rx = vm_exec(|env| {
        let act = if let Some(parent) = parent {
            let act = env.new_local_ref(parent.as_window().to_android())?;
            unsafe { Activity::from_raw(env, act.into_raw()) }
        } else {
            crate::current_activity(env)?
        };
        let (tx, rx) = futures_channel::mpsc::unbounded::<Vec<PathBuf>>();
        let proxy = DynamicProxy::build(
            env,
            &LoaderContext::FromObject(&act),
            [jni::jni_str!(
                "androidx/activity/result/ActivityResultCallback"
            )],
            move |env, _method, args| {
                let result = args.get_element(env, 0)?;
                if env.is_instance_of(&result, Uri::class_name())? {
                    let uri = unsafe { Uri::from_raw(env, result.into_raw()) };
                    let path = uri.get_path(env)?.try_to_string(env)?;
                    tx.unbounded_send(vec![PathBuf::from(path)]).ok();
                } else if env.is_instance_of(&result, JList::class_name())? {
                    let list = unsafe { JList::from_raw(env, result.into_raw()) };
                    let mut paths = vec![];
                    for i in 0..list.size(env)? {
                        let item = list.get(env, i)?;
                        let uri = unsafe { Uri::from_raw(env, item.into_raw()) };
                        let path = uri.get_path(env)?.try_to_string(env)?;
                        paths.push(PathBuf::from(path));
                    }
                    tx.unbounded_send(paths).ok();
                } else {
                    tx.unbounded_send(vec![]).ok();
                }
                Ok(JObject::null())
            },
        )?;
        let (launcher, input) = if open && multiple {
            let action = GetMultipleContents::new(env)?;
            (
                act.register_for_activity_result(env, action, proxy.as_ref())?,
                "*/*",
            )
        } else if folder {
            let action = OpenDocumentTree::new(env)?;
            (
                act.register_for_activity_result(env, action, proxy.as_ref())?,
                "*/*",
            )
        } else if open {
            let action = GetContent::new(env)?;
            (
                act.register_for_activity_result(env, action, proxy.as_ref())?,
                "*/*",
            )
        } else {
            let action = CreateDocument::new(env)?;
            (
                act.register_for_activity_result(env, action, proxy.as_ref())?,
                filename.as_str(),
            )
        };
        let input = env.new_string(input)?;
        launcher.launch(env, input)?;
        Result::Ok(rx)
    })?;
    rx.recv()
        .await
        .map_err(|e| Error::Io(std::io::Error::other(e)))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileFilter {
    name: String,
    pattern: String,
}

impl FileFilter {
    pub fn new(name: impl AsRef<str>, pattern: impl AsRef<str>) -> Self {
        Self {
            name: name.as_ref().into(),
            pattern: pattern.as_ref().into(),
        }
    }
}
