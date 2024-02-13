use std::{
    borrow::Cow,
    io,
    ops::{BitOr, BitOrAssign},
    ptr::{null, null_mut},
};

use widestring::{U16CStr, U16CString};
use windows_sys::Win32::{
    Foundation::{E_INVALIDARG, E_OUTOFMEMORY, S_OK},
    UI::{
        Controls::{
            TaskDialogIndirect, TASKDIALOGCONFIG, TASKDIALOGCONFIG_0, TASKDIALOGCONFIG_1,
            TDCBF_CANCEL_BUTTON, TDCBF_CLOSE_BUTTON, TDCBF_NO_BUTTON, TDCBF_OK_BUTTON,
            TDCBF_RETRY_BUTTON, TDCBF_YES_BUTTON, TDF_ALLOW_DIALOG_CANCELLATION,
            TDF_SIZE_TO_CONTENT, TD_ERROR_ICON, TD_INFORMATION_ICON, TD_SHIELD_ICON,
            TD_WARNING_ICON,
        },
        WindowsAndMessaging::{IDCANCEL, IDCLOSE, IDNO, IDOK, IDRETRY, IDYES},
    },
};

use crate::ui::window::AsRawWindow;

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum MessageBoxStyle {
    #[default]
    None,
    Info,
    Warning,
    Error,
    Shield,
}

#[repr(i32)]
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum MessageBoxButton {
    #[default]
    Ok     = TDCBF_OK_BUTTON,
    Yes    = TDCBF_YES_BUTTON,
    No     = TDCBF_NO_BUTTON,
    Cancel = TDCBF_CANCEL_BUTTON,
    Retry  = TDCBF_RETRY_BUTTON,
    Close  = TDCBF_CLOSE_BUTTON,
}

impl BitOr for MessageBoxButton {
    type Output = MessageBoxButton;

    fn bitor(self, rhs: Self) -> Self::Output {
        unsafe { std::mem::transmute(self as i32 | rhs as i32) }
    }
}

impl BitOrAssign for MessageBoxButton {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

#[repr(i32)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MessageBoxResponse {
    Cancel = IDCANCEL,
    No     = IDNO,
    Ok     = IDOK,
    Retry  = IDRETRY,
    Yes    = IDYES,
    Close  = IDCLOSE,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CustomButton {
    pub result: i32,
    pub text: Cow<'static, str>,
}

impl CustomButton {
    pub fn new<S: Into<Cow<'static, str>>>(result: i32, text: S) -> Self {
        Self {
            result,
            text: text.into(),
        }
    }
}

fn msgbox_custom(
    parent: Option<impl AsRawWindow>,
    msg: &U16CStr,
    title: &U16CStr,
    instr: &U16CStr,
    style: MessageBoxStyle,
    btns: MessageBoxButton,
    default: i32,
) -> io::Result<MessageBoxResponse> {
    let config = TASKDIALOGCONFIG {
        cbSize: std::mem::size_of::<TASKDIALOGCONFIG>() as _,
        hwndParent: parent.map(|p| p.as_raw_window()).unwrap_or_default(),
        hInstance: 0,
        dwFlags: TDF_ALLOW_DIALOG_CANCELLATION | TDF_SIZE_TO_CONTENT,
        dwCommonButtons: btns as _,
        pszWindowTitle: title.as_ptr(),
        Anonymous1: TASKDIALOGCONFIG_0 {
            pszMainIcon: match style {
                MessageBoxStyle::None => null_mut(),
                MessageBoxStyle::Info => TD_INFORMATION_ICON,
                MessageBoxStyle::Warning => TD_WARNING_ICON,
                MessageBoxStyle::Error => TD_ERROR_ICON,
                MessageBoxStyle::Shield => TD_SHIELD_ICON,
            },
        },
        pszMainInstruction: instr.as_ptr(),
        pszContent: msg.as_ptr(),
        cButtons: 0,
        pButtons: null(),
        nDefaultButton: default,
        cRadioButtons: 0,
        pRadioButtons: null(),
        nDefaultRadioButton: 0,
        pszVerificationText: null(),
        pszExpandedInformation: null(),
        pszExpandedControlText: null(),
        pszCollapsedControlText: null(),
        Anonymous2: TASKDIALOGCONFIG_1 { hFooterIcon: 0 },
        pszFooter: null(),
        pfCallback: None,
        lpCallbackData: 0,
        cxWidth: 0,
    };

    let mut result = 0;
    let res = unsafe { TaskDialogIndirect(&config, &mut result, null_mut(), null_mut()) };
    match res {
        S_OK => Ok(unsafe { std::mem::transmute(result) }),
        E_OUTOFMEMORY => Err(io::ErrorKind::OutOfMemory.into()),
        E_INVALIDARG => Err(io::ErrorKind::InvalidInput.into()),
        _ => Err(io::Error::from_raw_os_error(res)),
    }
}

pub struct MessageBox<'a, W> {
    parent: Option<&'a W>,
    msg: U16CString,
    title: U16CString,
    instr: U16CString,
    style: MessageBoxStyle,
    btns: MessageBoxButton,
    def: i32,
}

impl<'a, W: AsRawWindow> MessageBox<'a, W> {
    pub fn new(parent: Option<&'a W>) -> Self {
        Self {
            parent,
            msg: U16CString::new(),
            title: U16CString::new(),
            instr: U16CString::new(),
            style: MessageBoxStyle::None,
            btns: MessageBoxButton::Ok,
            def: 0,
        }
    }

    pub fn show(&self) -> io::Result<MessageBoxResponse> {
        msgbox_custom(
            self.parent,
            &self.msg,
            &self.title,
            &self.instr,
            self.style,
            self.btns,
            self.def,
        )
    }

    pub fn message(&mut self, msg: impl AsRef<str>) -> &mut Self {
        self.msg = U16CString::from_str_truncate(msg.as_ref());
        self
    }

    pub fn title(&mut self, title: impl AsRef<str>) -> &mut Self {
        self.title = U16CString::from_str_truncate(title.as_ref());
        self
    }

    pub fn instruction(&mut self, instr: impl AsRef<str>) -> &mut Self {
        self.instr = U16CString::from_str_truncate(instr);
        self
    }

    pub fn style(&mut self, style: MessageBoxStyle) -> &mut Self {
        self.style = style;
        self
    }

    pub fn buttons(&mut self, btns: MessageBoxButton) -> &mut Self {
        self.btns = btns;
        self
    }

    pub fn default_button(&mut self, def: i32) -> &mut Self {
        self.def = def;
        self
    }
}
