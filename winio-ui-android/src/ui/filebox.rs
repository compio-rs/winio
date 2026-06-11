use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use futures_channel::mpsc::UnboundedReceiver;
use futures_util::lock::Mutex as AsyncMutex;
use jni::{
    Env,
    objects::{JList, JObject, JString},
    refs::{Global, LoaderContext, Reference},
};
use jni_min_helper::DynamicProxy;
use winio_handle::AsWindow;

use crate::{Activity, Error, Result, impl_listener, vm_exec};

jni::bind_java_type! {
    ActivityResultCaller => androidx.activity.result.ActivityResultCaller,
    type_map {
        ActivityResultContract => androidx.activity.result.contract.ActivityResultContract,
        ActivityResultLauncher => androidx.activity.result.ActivityResultLauncher,
        ActivityResultCallback => androidx.activity.result.ActivityResultCallback,
    },
    methods {
        fn register_for_activity_result(contract: &ActivityResultContract, callback: &ActivityResultCallback) -> ActivityResultLauncher,
    },
}

jni::bind_java_type! {
    ActivityResultContract => androidx.activity.result.contract.ActivityResultContract,
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
    ActivityResultLauncher => androidx.activity.result.ActivityResultLauncher,
    methods {
        fn launch(input: &JObject),
    },
}

jni::bind_java_type! {
    ActivityResultCallback => androidx.activity.result.ActivityResultCallback,
}

impl_listener!(ActivityResultCallback);

jni::bind_java_type! {
    Uri => android.net.Uri,
    methods {
        fn to_string() -> JString,
    },
}

struct ProxyCallback {
    proxy: DynamicProxy,
    rx: Arc<AsyncMutex<UnboundedReceiver<Vec<PathBuf>>>>,
}

impl ProxyCallback {
    pub fn new(env: &mut Env, context: &Activity) -> jni::errors::Result<Self> {
        let (tx, rx) = futures_channel::mpsc::unbounded::<Vec<PathBuf>>();
        let proxy = DynamicProxy::build(
            env,
            &LoaderContext::FromObject(context),
            [ActivityResultCallback::class_name()],
            move |env, _method, args| {
                let result = args.get_element(env, 0)?;
                if env.is_instance_of(&result, Uri::class_name())? {
                    let uri = unsafe { Uri::from_raw(env, result.into_raw()) };
                    if !uri.is_null() {
                        let path = uri.to_string(env)?.try_to_string(env)?;
                        tx.unbounded_send(vec![PathBuf::from(path)]).ok();
                    } else {
                        tx.unbounded_send(vec![]).ok();
                    }
                } else if env.is_instance_of(&result, JList::class_name())? {
                    let list = unsafe { JList::from_raw(env, result.into_raw()) };
                    let mut paths = vec![];
                    for i in 0..list.size(env)? {
                        let item = list.get(env, i)?;
                        let uri = unsafe { Uri::from_raw(env, item.into_raw()) };
                        if !uri.is_null() {
                            let path = uri.to_string(env)?.try_to_string(env)?;
                            paths.push(PathBuf::from(path));
                        }
                    }
                    tx.unbounded_send(paths).ok();
                } else {
                    tx.unbounded_send(vec![]).ok();
                }
                Ok(JObject::null())
            },
        )?;
        Ok(Self {
            proxy,
            rx: Arc::new(AsyncMutex::new(rx)),
        })
    }

    pub fn callback<'local>(&self) -> &ActivityResultCallback<'local> {
        self.proxy.as_ref()
    }

    pub fn receiver(&self) -> Arc<AsyncMutex<UnboundedReceiver<Vec<PathBuf>>>> {
        self.rx.clone()
    }
}

static PROXY_CALLBACK: Mutex<Option<ProxyCallback>> = Mutex::new(None);

static LAUNCHER_GET_CONTENT: Mutex<Option<Global<ActivityResultLauncher<'static>>>> =
    Mutex::new(None);

static LAUNCHER_GET_MULTIPLE_CONTENTS: Mutex<Option<Global<ActivityResultLauncher<'static>>>> =
    Mutex::new(None);

static LAUNCHER_OPEN_DOCUMENT_TREE: Mutex<Option<Global<ActivityResultLauncher<'static>>>> =
    Mutex::new(None);

static LAUNCHER_CREATE_DOCUMENT: Mutex<Option<Global<ActivityResultLauncher<'static>>>> =
    Mutex::new(None);

pub(crate) fn register_launcher(env: &mut Env, act: &Activity) -> jni::errors::Result<()> {
    let mut proxy_callback = PROXY_CALLBACK.lock().unwrap();
    *proxy_callback = Some(ProxyCallback::new(env, act)?);
    let callback = proxy_callback.as_ref().unwrap().callback();

    let act = env.new_local_ref(act)?;
    let act = unsafe { ActivityResultCaller::from_raw(env, act.into_raw()) };

    let mut launcher_get_content = LAUNCHER_GET_CONTENT.lock().unwrap();
    let action = GetContent::new(env)?;
    let launcher = act.register_for_activity_result(env, action, callback)?;
    *launcher_get_content = Some(env.new_global_ref(launcher)?);

    let mut launcher_get_multiple_contents = LAUNCHER_GET_MULTIPLE_CONTENTS.lock().unwrap();
    let action = GetMultipleContents::new(env)?;
    let launcher = act.register_for_activity_result(env, action, callback)?;
    *launcher_get_multiple_contents = Some(env.new_global_ref(launcher)?);

    let mut launcher_open_document_tree = LAUNCHER_OPEN_DOCUMENT_TREE.lock().unwrap();
    let action = OpenDocumentTree::new(env)?;
    let launcher = act.register_for_activity_result(env, action, callback)?;
    *launcher_open_document_tree = Some(env.new_global_ref(launcher)?);

    let mut launcher_create_document = LAUNCHER_CREATE_DOCUMENT.lock().unwrap();
    let action = CreateDocument::new(env)?;
    let launcher = act.register_for_activity_result(env, action, callback)?;
    *launcher_create_document = Some(env.new_global_ref(launcher)?);

    Ok(())
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
    _parent: Option<impl AsWindow>,
    _title: String,
    filename: String,
    _filters: Vec<FileFilter>,
    open: bool,
    multiple: bool,
    folder: bool,
) -> Result<Vec<PathBuf>> {
    vm_exec(|env| {
        let (launcher, input) = if open && multiple {
            (LAUNCHER_GET_MULTIPLE_CONTENTS.lock().unwrap(), Some("*/*"))
        } else if folder {
            (LAUNCHER_OPEN_DOCUMENT_TREE.lock().unwrap(), None)
        } else if open {
            (LAUNCHER_GET_CONTENT.lock().unwrap(), Some("*/*"))
        } else {
            (
                LAUNCHER_CREATE_DOCUMENT.lock().unwrap(),
                Some(filename.as_str()),
            )
        };
        let input = if let Some(input) = input {
            env.new_string(input)?
        } else {
            JString::null()
        };
        launcher.as_ref().unwrap().launch(env, input)?;
        Result::Ok(())
    })?;
    let rx = PROXY_CALLBACK.lock().unwrap().as_ref().unwrap().receiver();
    rx.lock()
        .await
        .recv()
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
