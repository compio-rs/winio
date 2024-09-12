use std::{
    io,
    panic::resume_unwind,
    ptr::{null, null_mut},
};

use widestring::U16CString;
use windows_sys::Win32::{
    Foundation::{E_INVALIDARG, E_OUTOFMEMORY, S_OK},
    UI::{
        Controls::{
            TaskDialogIndirect, TASKDIALOGCONFIG, TASKDIALOGCONFIG_0, TASKDIALOGCONFIG_1,
            TDF_ALLOW_DIALOG_CANCELLATION, TDF_SIZE_TO_CONTENT, TD_ERROR_ICON, TD_INFORMATION_ICON,
            TD_WARNING_ICON,
        },
        WindowsAndMessaging::{IDCANCEL, IDCLOSE, IDNO, IDOK, IDRETRY, IDYES},
    },
};

use crate::{AsRawWindow, MessageBoxButton, MessageBoxResponse, MessageBoxStyle, Window};

async fn msgbox_custom(
    parent: Option<&Window>,
    msg: U16CString,
    title: U16CString,
    instr: U16CString,
    style: MessageBoxStyle,
    btns: MessageBoxButton,
) -> io::Result<MessageBoxResponse> {
    let parent_handle = parent
        .map(|p| p.as_raw_window() as isize)
        .unwrap_or_default();
    let (res, result) = compio::runtime::spawn_blocking(move || {
        let config = TASKDIALOGCONFIG {
            cbSize: std::mem::size_of::<TASKDIALOGCONFIG>() as _,
            hwndParent: parent_handle as _,
            hInstance: null_mut(),
            dwFlags: TDF_ALLOW_DIALOG_CANCELLATION | TDF_SIZE_TO_CONTENT,
            dwCommonButtons: btns as _,
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
            cButtons: 0,
            pButtons: null(),
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
            pfCallback: None,
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
        S_OK => Ok(match result {
            IDCANCEL => MessageBoxResponse::Cancel,
            IDNO => MessageBoxResponse::No,
            IDOK => MessageBoxResponse::Ok,
            IDRETRY => MessageBoxResponse::Retry,
            IDYES => MessageBoxResponse::Yes,
            IDCLOSE => MessageBoxResponse::Close,
            _ => unreachable!(),
        }),
        E_OUTOFMEMORY => Err(io::ErrorKind::OutOfMemory.into()),
        E_INVALIDARG => Err(io::ErrorKind::InvalidInput.into()),
        _ => Err(io::Error::from_raw_os_error(res)),
    }
}

#[derive(Debug, Clone)]
pub struct MessageBox {
    msg: U16CString,
    title: U16CString,
    instr: U16CString,
    style: MessageBoxStyle,
    btns: MessageBoxButton,
}

impl Default for MessageBox {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageBox {
    pub fn new() -> Self {
        Self {
            msg: U16CString::new(),
            title: U16CString::new(),
            instr: U16CString::new(),
            style: MessageBoxStyle::None,
            btns: MessageBoxButton::Ok,
        }
    }

    pub async fn show(self, parent: Option<&Window>) -> io::Result<MessageBoxResponse> {
        msgbox_custom(
            parent, self.msg, self.title, self.instr, self.style, self.btns,
        )
        .await
    }

    pub fn message(mut self, msg: impl AsRef<str>) -> Self {
        self.msg = U16CString::from_str_truncate(msg.as_ref());
        self
    }

    pub fn title(mut self, title: impl AsRef<str>) -> Self {
        self.title = U16CString::from_str_truncate(title.as_ref());
        self
    }

    pub fn instruction(mut self, instr: impl AsRef<str>) -> Self {
        self.instr = U16CString::from_str_truncate(instr);
        self
    }

    pub fn style(mut self, style: MessageBoxStyle) -> Self {
        self.style = style;
        self
    }

    pub fn buttons(mut self, btns: MessageBoxButton) -> Self {
        self.btns = btns;
        self
    }
}
