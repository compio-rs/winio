use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use futures_channel::mpsc::UnboundedReceiver;
use futures_util::{TryFutureExt, lock::Mutex as AsyncMutex};
use jni::{
    Env,
    objects::{JList, JObject, JString},
    refs::{Global, LoaderContext, Reference},
};
use jni_min_helper::DynamicProxy;
use winio_handle::AsWindow;

use crate::{
    Error, Result,
    java::{
        android::net::Uri,
        androidx::activity::{
            ActivityResultCallback, ActivityResultCaller, ActivityResultLauncher, CreateDocument,
            GetContent, GetMultipleContents, OpenDocumentTree,
        },
        custom::Activity,
    },
    vm_exec,
};

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

    pub fn open(
        self,
        parent: Option<impl AsWindow>,
    ) -> Result<impl Future<Output = Result<Option<PathBuf>>> + 'static> {
        filebox(
            parent,
            self.title,
            self.filename,
            self.filters,
            true,
            false,
            false,
        )
        .map(|fut| fut.map_ok(|paths| paths.into_iter().next()))
    }

    pub fn open_multiple(
        self,
        parent: Option<impl AsWindow>,
    ) -> Result<impl Future<Output = Result<Vec<PathBuf>>> + 'static> {
        filebox(
            parent,
            self.title,
            self.filename,
            self.filters,
            true,
            true,
            false,
        )
    }

    pub fn open_folder(
        self,
        parent: Option<impl AsWindow>,
    ) -> Result<impl Future<Output = Result<Option<PathBuf>>> + 'static> {
        filebox(
            parent,
            self.title,
            self.filename,
            self.filters,
            false,
            false,
            true,
        )
        .map(|fut| fut.map_ok(|paths| paths.into_iter().next()))
    }

    pub fn save(
        self,
        parent: Option<impl AsWindow>,
    ) -> Result<impl Future<Output = Result<Option<PathBuf>>> + 'static> {
        filebox(
            parent,
            self.title,
            self.filename,
            self.filters,
            false,
            false,
            false,
        )
        .map(|fut| fut.map_ok(|paths| paths.into_iter().next()))
    }
}

fn filebox(
    _parent: Option<impl AsWindow>,
    _title: String,
    filename: String,
    _filters: Vec<FileFilter>,
    open: bool,
    multiple: bool,
    folder: bool,
) -> Result<impl Future<Output = Result<Vec<PathBuf>>> + 'static> {
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
    Ok(async move {
        rx.lock()
            .await
            .recv()
            .await
            .map_err(|e| Error::Io(std::io::Error::other(e)))
    })
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
