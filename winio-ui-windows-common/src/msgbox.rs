use std::{
    panic::resume_unwind,
    ptr::{null, null_mut},
};

use widestring::U16CString;
use windows_sys::Win32::{
    Foundation::{E_INVALIDARG, E_OUTOFMEMORY, HWND, S_OK},
    UI::{
        Controls::{
            PFTASKDIALOGCALLBACK, TASKDIALOG_BUTTON, TASKDIALOGCONFIG, TASKDIALOGCONFIG_0,
            TASKDIALOGCONFIG_1, TD_ERROR_ICON, TD_INFORMATION_ICON, TD_WARNING_ICON,
            TDF_ALLOW_DIALOG_CANCELLATION, TDF_SIZE_TO_CONTENT, TaskDialogIndirect,
        },
        WindowsAndMessaging::{IDCANCEL, IDCLOSE, IDNO, IDOK, IDRETRY, IDYES},
    },
};
use winio_primitive::{MessageBoxButton, MessageBoxResponse, MessageBoxStyle};

#[allow(clippy::too_many_arguments)]
pub async fn msgbox(
    parent: Option<HWND>,
    msg: U16CString,
    title: U16CString,
    instr: U16CString,
    style: MessageBoxStyle,
    btns: MessageBoxButton,
    cbtns: Vec<CustomButton>,
    callback: PFTASKDIALOGCALLBACK,
) -> MessageBoxResponse {
    let parent_handle = parent.map(|p| p as isize).unwrap_or_default();
    let (res, result) = compio::runtime::spawn_blocking(move || {
        let cbtn_ptrs = cbtns
            .iter()
            .map(|b| TASKDIALOG_BUTTON {
                nButtonID: b.result as _,
                pszButtonText: b.text.as_ptr(),
            })
            .collect::<Vec<_>>();
        let config = TASKDIALOGCONFIG {
            cbSize: std::mem::size_of::<TASKDIALOGCONFIG>() as _,
            hwndParent: parent_handle as _,
            hInstance: null_mut(),
            dwFlags: TDF_ALLOW_DIALOG_CANCELLATION | TDF_SIZE_TO_CONTENT,
            dwCommonButtons: btns.bits(),
            pszWindowTitle: title.as_ptr(),
            Anonymous1: TASKDIALOGCONFIG_0 {
                pszMainIcon: match style {
                    MessageBoxStyle::None => null_mut(),
                    MessageBoxStyle::Info => TD_INFORMATION_ICON,
                    MessageBoxStyle::Warning => TD_WARNING_ICON,
                    MessageBoxStyle::Error => TD_ERROR_ICON,
                },
            },
            pszMainInstruction: instr.as_ptr(),
            pszContent: msg.as_ptr(),
            cButtons: cbtn_ptrs.len() as _,
            pButtons: if cbtn_ptrs.is_empty() {
                null()
            } else {
                cbtn_ptrs.as_ptr()
            },
            nDefaultButton: 0,
            cRadioButtons: 0,
            pRadioButtons: null(),
            nDefaultRadioButton: 0,
            pszVerificationText: null(),
            pszExpandedInformation: null(),
            pszExpandedControlText: null(),
            pszCollapsedControlText: null(),
            Anonymous2: TASKDIALOGCONFIG_1 {
                hFooterIcon: null_mut(),
            },
            pszFooter: null(),
            pfCallback: callback,
            lpCallbackData: 0,
            cxWidth: 0,
        };

        let mut result = 0;
        let res = unsafe { TaskDialogIndirect(&config, &mut result, null_mut(), null_mut()) };
        (res, result)
    })
    .await
    .unwrap_or_else(|e| resume_unwind(e));

    match res {
        S_OK => match result {
            IDCANCEL => MessageBoxResponse::Cancel,
            IDNO => MessageBoxResponse::No,
            IDOK => MessageBoxResponse::Ok,
            IDRETRY => MessageBoxResponse::Retry,
            IDYES => MessageBoxResponse::Yes,
            IDCLOSE => MessageBoxResponse::Close,
            _ => MessageBoxResponse::Custom(result as _),
        },
        E_OUTOFMEMORY => panic!(
            "{:?}",
            std::io::Error::from(std::io::ErrorKind::OutOfMemory)
        ),
        E_INVALIDARG => panic!(
            "{:?}",
            std::io::Error::from(std::io::ErrorKind::InvalidInput)
        ),
        _ => panic!("{:?}", std::io::Error::from_raw_os_error(res)),
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CustomButton {
    pub result: u16,
    pub text: U16CString,
}

impl CustomButton {
    pub fn new(result: u16, text: &str) -> Self {
        Self {
            result,
            text: U16CString::from_str_truncate(text),
        }
    }
}
