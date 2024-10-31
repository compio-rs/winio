use std::{
    mem::MaybeUninit,
    ptr::{null, null_mut},
    sync::{LazyLock, Once},
};

use slim_detours_sys::{DETOUR_INLINE_HOOK, SlimDetoursInlineHooks};
use sync_unsafe_cell::SyncUnsafeCell;
use widestring::U16CStr;
use windows_sys::{
    Win32::{
        Foundation::{BOOL, BOOLEAN, COLORREF, HWND, LPARAM, LRESULT, RECT, S_OK, WPARAM},
        Globalization::{CSTR_EQUAL, CompareStringW, LOCALE_ALL, NORM_IGNORECASE},
        Graphics::{
            Dwm::DwmSetWindowAttribute,
            Gdi::{
                CreateSolidBrush, DRAW_TEXT_FORMAT, FillRect, HDC, InvalidateRect, Rectangle,
                SelectObject,
            },
        },
        System::SystemServices::MAX_CLASS_NAME,
        UI::{
            Accessibility::{HCF_HIGHCONTRASTON, HIGHCONTRASTW},
            Controls::{
                BP_CHECKBOX, BP_RADIOBUTTON, DTBGOPTS, DTT_TEXTCOLOR, DTTOPTS, DrawThemeBackground,
                DrawThemeBackgroundEx, DrawThemeText, DrawThemeTextEx, GetThemeColor, HTHEME,
                PBS_DISABLED, PP_TRANSPARENTBAR, PROGRESS_CLASSW, SetWindowTheme,
                TASKDIALOG_NOTIFICATIONS, TDLG_MAININSTRUCTIONPANE, TDLG_PRIMARYPANEL,
                TDLG_SECONDARYPANEL, TDN_CREATED, TDN_DIALOG_CONSTRUCTED, TMT_TEXTCOLOR,
                WC_BUTTONW, WC_COMBOBOXW,
            },
            Shell::{DefSubclassProc, SetWindowSubclass},
            WindowsAndMessaging::{
                EnumChildWindows, GetClassNameW, GetClientRect, SPI_GETHIGHCONTRAST,
                SystemParametersInfoW, WM_CTLCOLORDLG, WM_ERASEBKGND, WM_SETTINGCHANGE,
            },
        },
    },
    core::{HRESULT, PCWSTR},
    w,
};

use crate::ui::font::WinBrush;

#[link(name = "ntdll")]
extern "system" {
    fn RtlGetNtVersionNumbers(major: *mut u32, minor: *mut u32, build: *mut u32);
}

#[repr(u32)]
#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub enum PreferredAppMode {
    Default    = 0,
    AllowDark  = 1,
    ForceDark  = 2,
    ForceLight = 3,
}

#[link(name = "uxtheme", kind = "raw-dylib")]
extern "system" {
    #[link_ordinal(132)]
    fn ShouldAppsUseDarkMode() -> BOOLEAN;
    #[link_ordinal(133)]
    fn AllowDarkModeForWindow(h: HWND, a: BOOLEAN) -> BOOLEAN;
    // build < 18362
    #[link_ordinal(135)]
    fn AllowDarkModeForApp(a: BOOLEAN) -> BOOLEAN;
    // build >= 18362
    #[link_ordinal(135)]
    fn SetPreferredAppMode(m: PreferredAppMode) -> PreferredAppMode;
    #[link_ordinal(136)]
    fn FlushMenuThemes();
}

#[inline]
unsafe fn get_nt_build() -> u32 {
    let mut build = 0;
    RtlGetNtVersionNumbers(null_mut(), null_mut(), &mut build);
    build &= !0xF0000000;
    build
}

pub unsafe fn set_preferred_app_mode(m: PreferredAppMode) -> PreferredAppMode {
    let build = get_nt_build();
    if build < 18362 {
        if AllowDarkModeForApp(
            if m == PreferredAppMode::AllowDark || m == PreferredAppMode::ForceDark {
                1
            } else {
                0
            },
        ) != 0
        {
            PreferredAppMode::AllowDark
        } else {
            PreferredAppMode::Default
        }
    } else {
        SetPreferredAppMode(m)
    }
}

pub unsafe fn is_dark_mode_allowed_for_app() -> bool {
    let mut hc: HIGHCONTRASTW = std::mem::zeroed();
    hc.cbSize = size_of::<HIGHCONTRASTW>() as u32;
    if SystemParametersInfoW(
        SPI_GETHIGHCONTRAST,
        hc.cbSize,
        std::ptr::addr_of_mut!(hc).cast(),
        0,
    ) == 0
    {
        return false;
    }
    ((hc.dwFlags & HCF_HIGHCONTRASTON) == 0) && (ShouldAppsUseDarkMode() != 0)
}

const DWMWA_USE_IMMERSIVE_DARK_MODE: u32 = 0x13;
const DWMWA_USE_IMMERSIVE_DARK_MODE_V2: u32 = 0x14;

pub unsafe fn window_use_dark_mode(h_wnd: HWND) -> HRESULT {
    let set_dark_mode = is_dark_mode_allowed_for_app() as BOOL;
    AllowDarkModeForWindow(h_wnd, set_dark_mode as BOOLEAN);
    let hr = DwmSetWindowAttribute(
        h_wnd,
        DWMWA_USE_IMMERSIVE_DARK_MODE_V2,
        std::ptr::addr_of!(set_dark_mode).cast(),
        size_of::<BOOL>() as _,
    );
    if hr != 0 {
        let hr = DwmSetWindowAttribute(
            h_wnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            std::ptr::addr_of!(set_dark_mode).cast(),
            size_of::<BOOL>() as _,
        );
        if hr != 0 {
            return hr;
        }
    }
    FlushMenuThemes();
    S_OK
}

// MISC: If in task dialog, set lparam to 1.
pub unsafe fn children_refresh_dark_mode(handle: HWND, lparam: LPARAM) {
    unsafe extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        control_use_dark_mode(hwnd, lparam != 0);
        InvalidateRect(hwnd, null(), 1);
        EnumChildWindows(hwnd, Some(enum_callback), lparam);
        1
    }

    EnumChildWindows(handle, Some(enum_callback), lparam);
}

#[inline]
fn u16_string_eq_ignore_case(s1: &U16CStr, s2: *const u16) -> bool {
    unsafe {
        CompareStringW(
            LOCALE_ALL,
            NORM_IGNORECASE,
            s1.as_ptr(),
            s1.len() as _,
            s2,
            -1,
        ) == CSTR_EQUAL
    }
}

pub unsafe fn control_use_dark_mode(hwnd: HWND, misc_task_dialog: bool) {
    let mut class = [0u16; MAX_CLASS_NAME as usize];
    GetClassNameW(hwnd, class.as_mut_ptr(), MAX_CLASS_NAME);
    let class = U16CStr::from_ptr_str(class.as_ptr());
    let subappname = if is_dark_mode_allowed_for_app() {
        if u16_string_eq_ignore_case(class, WC_COMBOBOXW) {
            w!("DarkMode_CFD")
        } else if u16_string_eq_ignore_case(class, PROGRESS_CLASSW)
            || (u16_string_eq_ignore_case(class, WC_BUTTONW) && misc_task_dialog)
        {
            null()
        } else {
            w!("DarkMode_Explorer")
        }
    } else if u16_string_eq_ignore_case(class, WC_BUTTONW) && misc_task_dialog {
        w!("DarkMode_Explorer")
    } else {
        null()
    };
    SetWindowTheme(hwnd, subappname, null());
}

const WHITE: COLORREF = 0x00FFFFFF;

type GetThemeColorFn = unsafe extern "system" fn(
    htheme: HTHEME,
    ipartid: i32,
    istateid: i32,
    ipropid: i32,
    pcolor: *mut COLORREF,
) -> HRESULT;
static TRUE_GET_THEME_COLOR: SyncUnsafeCell<GetThemeColorFn> = SyncUnsafeCell::new(GetThemeColor);

type DrawThemeTextFn = unsafe extern "system" fn(
    htheme: HTHEME,
    hdc: HDC,
    ipartid: i32,
    istateid: i32,
    psztext: PCWSTR,
    cchtext: i32,
    dwtextflags: DRAW_TEXT_FORMAT,
    dwtextflags2: u32,
    prect: *const RECT,
) -> HRESULT;
static TRUE_DRAW_THEME_TEXT: SyncUnsafeCell<DrawThemeTextFn> = SyncUnsafeCell::new(DrawThemeText);

type DrawThemeBackgroundFn = unsafe extern "system" fn(
    htheme: HTHEME,
    hdc: HDC,
    ipartid: i32,
    istateid: i32,
    prect: *const RECT,
    pcliprect: *const RECT,
) -> HRESULT;
static TRUE_DRAW_THEME_BACKGROUND: SyncUnsafeCell<DrawThemeBackgroundFn> =
    SyncUnsafeCell::new(DrawThemeBackground);

type DrawThemeBackgroundExFn = unsafe extern "system" fn(
    htheme: HTHEME,
    hdc: HDC,
    ipartid: i32,
    istateid: i32,
    prect: *const RECT,
    poptions: *const DTBGOPTS,
) -> HRESULT;
static TRUE_DRAW_THEME_BACKGROUND_EX: SyncUnsafeCell<DrawThemeBackgroundExFn> =
    SyncUnsafeCell::new(DrawThemeBackgroundEx);

#[inline]
unsafe fn detour_hooks() -> [DETOUR_INLINE_HOOK; 4] {
    [
        DETOUR_INLINE_HOOK {
            pszFuncName: null(),
            ppPointer: TRUE_GET_THEME_COLOR.get().cast(),
            pDetour: dark_get_theme_color as _,
        },
        DETOUR_INLINE_HOOK {
            pszFuncName: null(),
            ppPointer: TRUE_DRAW_THEME_TEXT.get().cast(),
            pDetour: dark_draw_theme_text as _,
        },
        DETOUR_INLINE_HOOK {
            pszFuncName: null(),
            ppPointer: TRUE_DRAW_THEME_BACKGROUND.get().cast(),
            pDetour: dark_draw_theme_background as _,
        },
        DETOUR_INLINE_HOOK {
            pszFuncName: null(),
            ppPointer: TRUE_DRAW_THEME_BACKGROUND_EX.get().cast(),
            pDetour: dark_draw_theme_background_ex as _,
        },
    ]
}

unsafe fn detour_attach() {
    let mut hooks = detour_hooks();
    SlimDetoursInlineHooks(1, hooks.len() as _, hooks.as_mut_ptr());
}

static DETOUR_GUARD: Once = Once::new();

pub unsafe fn init_dark() {
    DETOUR_GUARD.call_once(|| detour_attach());
}

unsafe extern "system" fn dark_get_theme_color(
    htheme: HTHEME,
    ipartid: i32,
    istateid: i32,
    ipropid: i32,
    pcolor: *mut COLORREF,
) -> HRESULT {
    let res = (*TRUE_GET_THEME_COLOR.get())(htheme, ipartid, istateid, ipropid, pcolor);

    if is_dark_mode_allowed_for_app() && ipropid == TMT_TEXTCOLOR as _ {
        if ipartid == TDLG_MAININSTRUCTIONPANE {
            *pcolor = increase(*pcolor, 150);
        } else {
            *pcolor = 0xFFFFFF;
        }
    }

    res
}

unsafe extern "system" fn dark_draw_theme_text(
    htheme: HTHEME,
    hdc: HDC,
    ipartid: i32,
    istateid: i32,
    psztext: PCWSTR,
    cchtext: i32,
    dwtextflags: DRAW_TEXT_FORMAT,
    dwtextflags2: u32,
    prect: *const RECT,
) -> HRESULT {
    if is_dark_mode_allowed_for_app()
        && (ipartid == BP_CHECKBOX || ipartid == BP_RADIOBUTTON)
        && istateid != PBS_DISABLED
    {
        let mut options: DTTOPTS = std::mem::zeroed();
        options.dwSize = std::mem::size_of::<DTTOPTS>() as _;
        options.dwFlags = DTT_TEXTCOLOR;
        options.crText = WHITE;
        DrawThemeTextEx(
            htheme,
            hdc,
            ipartid,
            istateid,
            psztext,
            cchtext,
            dwtextflags,
            prect.cast_mut(),
            &options,
        )
    } else {
        (*TRUE_DRAW_THEME_TEXT.get())(
            htheme,
            hdc,
            ipartid,
            istateid,
            psztext,
            cchtext,
            dwtextflags,
            dwtextflags2,
            prect,
        )
    }
}

unsafe extern "system" fn dark_draw_theme_background(
    htheme: HTHEME,
    hdc: HDC,
    ipartid: i32,
    istateid: i32,
    prect: *const RECT,
    pcliprect: *const RECT,
) -> HRESULT {
    let res = (*TRUE_DRAW_THEME_BACKGROUND.get())(htheme, hdc, ipartid, istateid, prect, pcliprect);
    if is_dark_mode_allowed_for_app() && ipartid == PP_TRANSPARENTBAR {
        FillRect(hdc, prect, DLG_GRAY_BACK.0);
    }
    res
}

unsafe extern "system" fn dark_draw_theme_background_ex(
    htheme: HTHEME,
    hdc: HDC,
    ipartid: i32,
    istateid: i32,
    prect: *const RECT,
    poptions: *const DTBGOPTS,
) -> HRESULT {
    if !is_dark_mode_allowed_for_app() {
        return (*TRUE_DRAW_THEME_BACKGROUND_EX.get())(
            htheme, hdc, ipartid, istateid, prect, poptions,
        );
    }
    match ipartid {
        TDLG_PRIMARYPANEL => {
            FillRect(hdc, prect, DLG_GRAY_BACK.0);
            S_OK
        }
        TDLG_SECONDARYPANEL => {
            FillRect(hdc, prect, DLG_DARK_BACK.0);
            S_OK
        }
        _ => {
            (*TRUE_DRAW_THEME_BACKGROUND_EX.get())(htheme, hdc, ipartid, istateid, prect, poptions)
        }
    }
}

static DLG_DARK_BACK: LazyLock<WinBrush> =
    LazyLock::new(|| WinBrush(unsafe { CreateSolidBrush(0x00242424) }));

static DLG_GRAY_BACK: LazyLock<WinBrush> =
    LazyLock::new(|| WinBrush(unsafe { CreateSolidBrush(0x00333333) }));

fn increase(c: COLORREF, inc: u32) -> COLORREF {
    let r = c & 0xFF;
    let r = (r + inc).min(0xFF);
    let g = (c >> 8) & 0xFF;
    let g = (g + inc).min(0xFF);
    let b = (c >> 16) & 0xFF;
    let b = (b + inc).min(0xFF);
    r + (g << 8) + (b << 16)
}

pub unsafe extern "system" fn task_dialog_callback(
    hwnd: HWND,
    msg: TASKDIALOG_NOTIFICATIONS,
    _wparam: WPARAM,
    _lparam: LPARAM,
    lprefdata: isize,
) -> HRESULT {
    match msg {
        TDN_CREATED | TDN_DIALOG_CONSTRUCTED => {
            window_use_dark_mode(hwnd);
            children_refresh_dark_mode(hwnd, 0);
        }
        _ => {}
    }
    if msg == TDN_CREATED {
        SetWindowSubclass(hwnd, Some(task_dialog_subclass), hwnd as _, lprefdata as _);
    }
    S_OK
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
        WM_SETTINGCHANGE => {
            window_use_dark_mode(hwnd);
            children_refresh_dark_mode(hwnd, 1);
        }
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
                return DLG_DARK_BACK.0 as _;
            }
        }
        _ => {}
    }
    DefSubclassProc(hwnd, umsg, wparam, lparam)
}
