use std::ops::{Deref, DerefMut};

use windows::Win32::System::{
    Com::{DATADIR_GET, FORMATETC, IDataObject, STGMEDIUM, TYMED_HGLOBAL, TYMED_NULL},
    Ole::ReleaseStgMedium,
};
use windows_sys::Win32::Foundation::S_OK;

use crate::{Data, Result, platform::data::HGlobal};

pub fn from_idata_object(object: &IDataObject) -> Result<Option<Data>> {
    let enumerator = unsafe { object.EnumFormatEtc(DATADIR_GET as u32)? };
    let mut data = Data::new()?;
    loop {
        let mut format = FORMATETC::default();
        let res = unsafe { enumerator.Next(1, &mut format, None) };
        if res.0 != S_OK {
            break;
        }
        let Ok(medium) = (unsafe { object.GetData(&format) }) else {
            continue;
        };
        let mut medium = StgMedium(medium);
        if medium.tymed & TYMED_HGLOBAL != 0 {
            let hglobal = unsafe { medium.u.hGlobal };
            if let Some(hglobal) = unsafe { HGlobal::from_raw(hglobal) } {
                medium.tymed = TYMED_NULL;
                data.insert_hglobal(format.cfFormat as u32, hglobal);
            }
        }
    }
    if data.is_empty() {
        Ok(None)
    } else {
        Ok(Some(data))
    }
}

struct StgMedium(STGMEDIUM);

impl Drop for StgMedium {
    fn drop(&mut self) {
        unsafe { ReleaseStgMedium(&mut self.0) };
    }
}

impl Deref for StgMedium {
    type Target = STGMEDIUM;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for StgMedium {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
