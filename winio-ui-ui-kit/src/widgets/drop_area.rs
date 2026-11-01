use std::{cell::RefCell, fmt::Debug, rc::Rc};

use block2::RcBlock;
use inherit_methods_macro::inherit_methods;
use objc2::{
    DeclaredClass, MainThreadOnly, Message, define_class, msg_send,
    rc::{Allocated, Retained},
    runtime::ProtocolObject,
};
use objc2_foundation::{
    MainThreadMarker, NSArray, NSData, NSError, NSObjectProtocol, NSRect, NSString,
};
use objc2_ui_kit::{
    UIDragDropSession, UIDropInteraction, UIDropInteractionDelegate, UIDropOperation,
    UIDropProposal, UIDropSession, UIView,
};
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{
    GlobalRuntime, Result, catch, file_url_uti, html_uti, platform::Data, png_uti, text_uti,
    tiff_uti, widgets::Widget,
};

/// A container that receives dragged data.
#[derive(Debug)]
pub struct DropArea {
    handle: Widget,
    view: Retained<DropView>,
}

#[inherit_methods(from = "self.handle")]
impl DropArea {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let parent = parent.as_container();
        let mtm = parent.as_ui_kit().mtm();

        catch(|| {
            let view = DropView::new(mtm);
            let proposal = UIDropProposal::initWithDropOperation(
                mtm.alloc::<UIDropProposal>(),
                UIDropOperation::Copy,
            );
            *view.ivars().proposal.borrow_mut() = Some(proposal);
            let interaction = UIDropInteraction::initWithDelegate(
                mtm.alloc::<UIDropInteraction>(),
                ProtocolObject::from_ref(&*view),
            );
            view.addInteraction(ProtocolObject::from_ref(&*interaction));
            let handle = Widget::from_uiview(parent, view.clone().into_super())?;

            Ok(Self { handle, view })
        })
        .flatten()
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, s: Size) -> Result<()>;

    pub async fn wait_drop(&self) -> Data {
        self.view.ivars().on_drop.wait().await
    }
}

winio_handle::impl_as_widget!(DropArea, handle);
winio_handle::impl_as_container!(DropArea, handle);

#[derive(Debug)]
struct DropViewIvars {
    on_drop: Rc<Callback<Data>>,
    proposal: RefCell<Option<Retained<UIDropProposal>>>,
}

impl Default for DropViewIvars {
    fn default() -> Self {
        Self {
            on_drop: Rc::new(Callback::new()),
            proposal: RefCell::new(None),
        }
    }
}

#[derive(Debug, Default)]
struct PendingDrop {
    remaining: usize,
    entries: Vec<(Retained<NSString>, Retained<NSData>)>,
}

define_class! {
    #[unsafe(super(UIView))]
    #[name = "WinioDropViewUIKit"]
    #[ivars = DropViewIvars]
    #[thread_kind = MainThreadOnly]
    #[derive(Debug)]
    struct DropView;

    #[allow(non_snake_case)]
    impl DropView {
        #[unsafe(method_id(initWithFrame:))]
        fn initWithFrame(this: Allocated<Self>, frame: NSRect) -> Option<Retained<Self>> {
            let this = this.set_ivars(DropViewIvars::default());
            unsafe { msg_send![super(this), initWithFrame: frame] }
        }
    }

    unsafe impl NSObjectProtocol for DropView {}

    #[allow(non_snake_case)]
    unsafe impl UIDropInteractionDelegate for DropView {
        #[unsafe(method(dropInteraction:canHandleSession:))]
        unsafe fn dropInteraction_canHandleSession(
            &self,
            _interaction: &UIDropInteraction,
            session: &ProtocolObject<dyn UIDropSession>,
        ) -> bool {
            !session.items().is_empty()
        }

        #[unsafe(method_id(dropInteraction:sessionDidUpdate:))]
        unsafe fn dropInteraction_sessionDidUpdate(
            &self,
            _interaction: &UIDropInteraction,
            _session: &ProtocolObject<dyn UIDropSession>,
        ) -> Retained<UIDropProposal> {
            self.ivars()
                .proposal
                .borrow()
                .as_ref()
                .expect("drop proposal initialized")
                .clone()
        }

        #[unsafe(method(dropInteraction:performDrop:))]
        unsafe fn dropInteraction_performDrop(
            &self,
            _interaction: &UIDropInteraction,
            session: &ProtocolObject<dyn UIDropSession>,
        ) {
            let pending = Rc::new(RefCell::new(PendingDrop::default()));
            for item in session.items().iter() {
                let provider = item.itemProvider();
                let types = provider.registeredTypeIdentifiers();
                let Some(uti) = provider_type(&types) else {
                    continue;
                };

                pending.borrow_mut().remaining += 1;
                let load_uti = uti.clone();
                let pending = pending.clone();
                let on_drop = self.ivars().on_drop.clone();
                let block = RcBlock::new(move |data: *mut NSData, _error: *mut NSError| {
                    if let Some(data) = unsafe { data.as_ref() } {
                        pending
                            .borrow_mut()
                            .entries
                            .push((uti.clone(), data.retain()));
                    }
                    let done = {
                        let mut pending = pending.borrow_mut();
                        pending.remaining -= 1;
                        pending.remaining == 0
                    };
                    if done {
                        let entries = std::mem::take(&mut pending.borrow_mut().entries);
                        if !entries.is_empty() {
                            on_drop.signal::<GlobalRuntime>(Data::from_loaded(entries));
                        }
                    }
                });
                unsafe {
                    provider
                        .loadDataRepresentationForTypeIdentifier_completionHandler(&load_uti, &block);
                }
            }
        }
    }
}

fn provider_type(types: &NSArray<NSString>) -> Option<Retained<NSString>> {
    for preferred in [
        file_url_uti(),
        png_uti(),
        tiff_uti(),
        html_uti(),
        text_uti(),
    ] {
        if types.iter().any(|uti| uti.isEqualToString(preferred)) {
            return Some(preferred.retain());
        }
    }
    types.iter().next()
}

impl DropView {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), initWithFrame: NSRect::ZERO] }
    }
}
