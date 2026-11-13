use windows::Win32::System::Com::{
    COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE, CoInitializeEx, CoUninitialize,
};

#[derive(Debug)]
pub struct CoInit(());

impl CoInit {
    pub fn new() -> crate::Result<Self> {
        unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE).ok()? };
        Ok(Self(()))
    }
}

impl Drop for CoInit {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}
