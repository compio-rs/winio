use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoUninitialize};

#[derive(Debug)]
pub struct CoInit(());

impl CoInit {
    pub fn new() -> crate::Result<Self> {
        unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()? };
        Ok(Self(()))
    }
}

impl Drop for CoInit {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}
