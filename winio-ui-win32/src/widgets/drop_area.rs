use std::{fmt::Debug, rc::Rc};

use inherit_methods_macro::inherit_methods;
use windows::Win32::{
    Foundation::POINTL,
    System::{
        Com::IDataObject,
        Ole::{
            DROPEFFECT_COPY, IDropTarget, IDropTarget_Impl, OleInitialize, OleUninitialize,
            RegisterDragDrop, RevokeDragDrop,
        },
        SystemServices::MODIFIERKEYS_FLAGS,
    },
};
use windows_core::{Ref, implement};
use winio_callback::Callback;
use winio_handle::{AsContainer, AsWidget};
use winio_pollable::GlobalRuntime;
use winio_primitive::{Point, Size};

use crate::{Data, Result, View, platform::drag_drop::from_idata_object};

#[derive(Debug)]
pub struct DropArea {
    view: View,
    on_drop: Rc<Callback<Data>>,
    _target: IDropTarget,
    _ole: OleGuard,
}

#[derive(Debug)]
struct OleGuard;

impl OleGuard {
    fn new() -> Result<Self> {
        unsafe { OleInitialize(None)? };
        Ok(Self)
    }
}

impl Drop for OleGuard {
    fn drop(&mut self) {
        unsafe { OleUninitialize() };
    }
}

#[implement(IDropTarget, Agile = false)]
struct DropTarget {
    on_drop: Rc<Callback<Data>>,
}

impl IDropTarget_Impl for DropTarget_Impl {
    fn DragEnter(
        &self,
        _pdataobj: Ref<IDataObject>,
        _grfkeystate: MODIFIERKEYS_FLAGS,
        _pt: &POINTL,
        pdweffect: *mut windows::Win32::System::Ole::DROPEFFECT,
    ) -> windows_core::Result<()> {
        unsafe { *pdweffect = DROPEFFECT_COPY };
        Ok(())
    }

    fn DragOver(
        &self,
        _grfkeystate: MODIFIERKEYS_FLAGS,
        _pt: &POINTL,
        pdweffect: *mut windows::Win32::System::Ole::DROPEFFECT,
    ) -> windows_core::Result<()> {
        unsafe { *pdweffect = DROPEFFECT_COPY };
        Ok(())
    }

    fn DragLeave(&self) -> windows_core::Result<()> {
        Ok(())
    }

    fn Drop(
        &self,
        pdataobj: Ref<IDataObject>,
        _grfkeystate: MODIFIERKEYS_FLAGS,
        _pt: &POINTL,
        pdweffect: *mut windows::Win32::System::Ole::DROPEFFECT,
    ) -> windows_core::Result<()> {
        unsafe { *pdweffect = DROPEFFECT_COPY };
        if let Some(data) = from_idata_object(pdataobj.ok()?)? {
            self.on_drop.signal::<GlobalRuntime>(data);
        }
        Ok(())
    }
}

#[inherit_methods(from = "self.view")]
impl DropArea {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let ole = OleGuard::new()?;
        let view = View::new(parent)?;
        let on_drop = Rc::new(Callback::new());
        let target: IDropTarget = DropTarget {
            on_drop: on_drop.clone(),
        }
        .into();
        let hwnd = view.as_widget().as_win32();
        unsafe {
            RegisterDragDrop(hwnd, &target)?;
        }
        Ok(Self {
            view,
            on_drop,
            _target: target,
            _ole: ole,
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, visible: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, point: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, size: Size) -> Result<()>;

    pub async fn wait_drop(&self) -> Data {
        self.on_drop.wait().await
    }
}

impl Drop for DropArea {
    fn drop(&mut self) {
        let hwnd = self.view.as_widget().as_win32();
        unsafe {
            let _ = RevokeDragDrop(hwnd);
        }
    }
}

winio_handle::impl_as_widget!(DropArea, view);
winio_handle::impl_as_container!(DropArea, view);
