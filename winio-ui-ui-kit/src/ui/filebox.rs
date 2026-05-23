use std::path::PathBuf;

use objc2::{
    DefinedClass, MainThreadMarker, MainThreadOnly, Message, define_class, msg_send,
    rc::{Allocated, Retained},
    runtime::ProtocolObject,
};
use objc2_foundation::{NSArray, NSObject, NSObjectProtocol, NSString, NSURL};
use objc2_ui_kit::{UIDocumentPickerDelegate, UIDocumentPickerViewController};
use objc2_uniform_type_identifiers::{UTType, UTTypeData, UTTypeDirectory};
use winio_callback::Callback;
use winio_handle::AsWindow;

use crate::{Error, GlobalRuntime, Result, catch, first_ui_window_scene, from_nsstring};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileFilter {
    name: String,
    pattern: String,
}

impl FileFilter {
    pub fn new(name: &str, pattern: &str) -> Self {
        Self {
            name: name.to_string(),
            pattern: pattern.to_string(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct FileBox {
    title: Retained<NSString>,
    filename: Retained<NSString>,
    filters: Vec<FileFilter>,
}

// SAFETY: NSString is thread-safe.
unsafe impl Send for FileBox {}
unsafe impl Sync for FileBox {}

impl FileBox {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(&mut self, title: &str) {
        self.title = NSString::from_str(title);
    }

    pub fn filename(&mut self, filename: &str) {
        self.filename = NSString::from_str(filename);
    }

    pub fn filters(&mut self, filters: impl IntoIterator<Item = FileFilter>) {
        self.filters = filters.into_iter().collect();
    }

    pub fn add_filter(&mut self, filter: FileFilter) {
        self.filters.push(filter);
    }

    pub async fn open(self, parent: Option<impl AsWindow>) -> Result<Option<PathBuf>> {
        Ok(filebox(parent, self.filters, false, false)
            .await?
            .into_iter()
            .next())
    }

    pub async fn open_multiple(self, parent: Option<impl AsWindow>) -> Result<Vec<PathBuf>> {
        filebox(parent, self.filters, true, false).await
    }

    pub async fn open_folder(self, parent: Option<impl AsWindow>) -> Result<Option<PathBuf>> {
        Ok(filebox(parent, self.filters, false, true)
            .await?
            .into_iter()
            .next())
    }

    pub async fn save(self, _parent: Option<impl AsWindow>) -> Result<Option<PathBuf>> {
        Err(Error::NotSupported)
    }
}

async fn filebox(
    parent: Option<impl AsWindow>,
    filters: Vec<FileFilter>,
    multiple: bool,
    folder: bool,
) -> Result<Vec<PathBuf>> {
    let mtm = MainThreadMarker::new().ok_or(Error::NotMainThread)?;
    let delegate = catch(|| {
        let ns_filters = if folder {
            vec![unsafe { UTTypeDirectory.retain() }]
        } else {
            let mut ns_filters = filters
                .into_iter()
                .filter_map(|f| {
                    let pattern = f.pattern;
                    if pattern == "*.*" || pattern == "*" {
                        Some(unsafe { UTTypeData.retain() })
                    } else {
                        UTType::typeWithFilenameExtension(&NSString::from_str(
                            pattern.strip_prefix("*.").unwrap_or(&pattern),
                        ))
                    }
                })
                .collect::<Vec<_>>();
            if ns_filters.is_empty() {
                ns_filters.push(unsafe { UTTypeData.retain() });
            }
            ns_filters
        };
        let ns_filters = NSArray::from_retained_slice(&ns_filters);
        let browser =
            UIDocumentPickerViewController::initForOpeningContentTypes(mtm.alloc(), &ns_filters);
        browser.setAllowsMultipleSelection(multiple);
        browser.setShouldShowFileExtensions(true);
        let delegate = FilePickerDelegate::new(mtm);
        let del_obj = ProtocolObject::from_ref(&*delegate);
        browser.setDelegate(Some(del_obj));
        let controller = if let Some(parent) = parent {
            parent.as_window().as_ui_kit().rootViewController()
        } else {
            first_ui_window_scene()?
                .and_then(|scene| scene.keyWindow())
                .and_then(|wnd| wnd.rootViewController())
        };
        if let Some(vc) = controller {
            vc.presentViewController_animated_completion(&browser, true, None);
        }
        Ok(delegate)
    })
    .flatten()?;
    let urls = delegate.ivars().on_pick.wait().await;
    Ok(urls
        .into_iter()
        .filter_map(|url| url.path().map(|s| from_nsstring(&s)).map(PathBuf::from))
        .collect())
}

#[derive(Debug, Default)]
struct FilePickerDelegateIvars {
    on_pick: Callback<Retained<NSArray<NSURL>>>,
}

define_class! {
    #[unsafe(super(NSObject))]
    #[name = "WinioFilePickerDelegateUIKit"]
    #[ivars = FilePickerDelegateIvars]
    #[thread_kind = MainThreadOnly]
    #[derive(Debug)]
    struct FilePickerDelegate;

    impl FilePickerDelegate {
        #[unsafe(method_id(init))]
        fn init(this: Allocated<Self>) -> Option<Retained<Self>> {
            let this = this.set_ivars(FilePickerDelegateIvars::default());
            unsafe { msg_send![super(this), init] }
        }
    }

    unsafe impl NSObjectProtocol for FilePickerDelegate {}

    #[allow(non_snake_case)]
    unsafe impl UIDocumentPickerDelegate for FilePickerDelegate {
        #[unsafe(method(documentPicker:didPickDocumentsAtURLs:))]
        unsafe fn documentPicker_didPickDocumentsAtURLs(
            &self,
            _controller: &UIDocumentPickerViewController,
            urls: &NSArray<NSURL>,
        ) {
            self.ivars().on_pick.signal::<GlobalRuntime>(urls.retain());
        }

        #[unsafe(method(documentPickerWasCancelled:))]
        fn documentPickerWasCancelled(&self, controller: &UIDocumentPickerViewController) {
            self.ivars().on_pick.signal::<GlobalRuntime>(NSArray::new());
        }
    }
}

impl FilePickerDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), init] }
    }
}
