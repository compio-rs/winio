use std::{
    mem::MaybeUninit,
    panic::resume_unwind,
    ptr::{null, null_mut},
    sync::LazyLock,
};

use widestring::U16CString;
use windows_sys::{
    Win32::{
        Foundation::{BOOL, E_INVALIDARG, E_OUTOFMEMORY, HWND, LPARAM, LRESULT, S_OK, WPARAM},
        Graphics::Gdi::{CreateSolidBrush, HDC, Rectangle, SelectObject},
        UI::{
            Controls::{
                TASKDIALOG_BUTTON, TASKDIALOG_NOTIFICATIONS, TASKDIALOGCONFIG, TASKDIALOGCONFIG_0,
                TASKDIALOGCONFIG_1, TD_ERROR_ICON, TD_INFORMATION_ICON, TD_WARNING_ICON,
                TDF_ALLOW_DIALOG_CANCELLATION, TDF_SIZE_TO_CONTENT, TDN_CREATED,
                TDN_DIALOG_CONSTRUCTED, TaskDialogIndirect,
            },
            Shell::{DefSubclassProc, GetWindowSubclass, SetWindowSubclass},
            WindowsAndMessaging::{
                EnumChildWindows, GetClientRect, IDCANCEL, IDCLOSE, IDNO, IDOK, IDRETRY, IDYES,
                WM_CTLCOLORDLG, WM_ERASEBKGND,
            },
        },
    },
    core::HRESULT,
};

use crate::{
    AsRawWindow, AsWindow, MessageBoxButton, MessageBoxResponse, MessageBoxStyle,
    ui::{
        darkmode::{
            children_refresh_dark_mode, init_dark, is_dark_mode_allowed_for_app,
            window_use_dark_mode,
        },
        font::WinBrush,
    },
};

async fn msgbox_custom(
    parent: Option<impl AsWindow>,
    msg: U16CString,
    title: U16CString,
    instr: U16CString,
    style: MessageBoxStyle,
    btns: MessageBoxButton,
    cbtns: Vec<CustomButton>,
) -> MessageBoxResponse {
    let parent_handle = parent
        .map(|p| p.as_window().as_raw_window() as isize)
        .unwrap_or_default();
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
            pfCallback: Some(task_dialog_callback),
            lpCallbackData: 0,
            cxWidth: 0,
        };

        let mut result = 0;
        unsafe {
            init_dark();
        }
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

unsafe extern "system" fn task_dialog_callback(
    hwnd: HWND,
    msg: TASKDIALOG_NOTIFICATIONS,
    _wparam: WPARAM,
    _lparam: LPARAM,
    lprefdata: isize,
) -> HRESULT {
    match msg {
        TDN_CREATED | TDN_DIALOG_CONSTRUCTED => {
            window_use_dark_mode(hwnd);
            children_refresh_dark_mode(hwnd);
            children_add_subclass(hwnd);
        }
        _ => {}
    }
    if msg == TDN_CREATED {
        SetWindowSubclass(hwnd, Some(task_dialog_subclass), hwnd as _, lprefdata as _);
    }
    S_OK
}

unsafe fn children_add_subclass(handle: HWND) {
    unsafe extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        EnumChildWindows(hwnd, Some(enum_callback), lparam);
        if GetWindowSubclass(hwnd, Some(task_dialog_subclass), hwnd as _, null_mut()) == 0 {
            SetWindowSubclass(hwnd, Some(task_dialog_subclass), hwnd as _, 0);
        }
        1
    }

    EnumChildWindows(handle, Some(enum_callback), 0);
}

unsafe extern "system" fn task_dialog_subclass(
    hwnd: HWND,
    umsg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _uidsubclass: usize,
    _dwrefdata: usize,
) -> LRESULT {
    match umsg {
        WM_ERASEBKGND => {
            if is_dark_mode_allowed_for_app() {
                let hdc = wparam as HDC;
                let brush = DLG_GRAY_BACK.0;
                let old_brush = SelectObject(hdc, brush);
                let mut r = MaybeUninit::uninit();
                GetClientRect(hwnd, r.as_mut_ptr());
                let r = r.assume_init();
                Rectangle(hdc, r.left - 1, r.top - 1, r.right + 1, r.bottom + 1);
                SelectObject(hdc, old_brush);
            }
        }
        WM_CTLCOLORDLG => {
            if is_dark_mode_allowed_for_app() {
                println!("CTLCOLORDLG");
                return DLG_DARK_BACK.0 as _;
            }
        }
        _ => {}
    }
    DefSubclassProc(hwnd, umsg, wparam, lparam)
}

static DLG_DARK_BACK: LazyLock<WinBrush> =
    LazyLock::new(|| WinBrush(unsafe { CreateSolidBrush(0x00242424) }));

static DLG_GRAY_BACK: LazyLock<WinBrush> =
    LazyLock::new(|| WinBrush(unsafe { CreateSolidBrush(0x00333333) }));

#[derive(Debug, Clone)]
pub struct MessageBox {
    msg: U16CString,
    title: U16CString,
    instr: U16CString,
    style: MessageBoxStyle,
    btns: MessageBoxButton,
    cbtns: Vec<CustomButton>,
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
            btns: MessageBoxButton::None,
            cbtns: vec![],
        }
    }

    pub async fn show(self, parent: Option<impl AsWindow>) -> MessageBoxResponse {
        msgbox_custom(
            parent, self.msg, self.title, self.instr, self.style, self.btns, self.cbtns,
        )
        .await
    }

    pub fn message(&mut self, msg: &str) {
        self.msg = U16CString::from_str_truncate(msg);
    }

    pub fn title(&mut self, title: &str) {
        self.title = U16CString::from_str_truncate(title);
    }

    pub fn instruction(&mut self, instr: &str) {
        self.instr = U16CString::from_str_truncate(instr);
    }

    pub fn style(&mut self, style: MessageBoxStyle) {
        self.style = style;
    }

    pub fn buttons(&mut self, btns: MessageBoxButton) {
        self.btns = btns;
    }

    pub fn custom_button(&mut self, btn: CustomButton) {
        self.cbtns.push(btn);
    }

    pub fn custom_buttons(&mut self, btn: impl IntoIterator<Item = CustomButton>) {
        self.cbtns.extend(btn);
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
