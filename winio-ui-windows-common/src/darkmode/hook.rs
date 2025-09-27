use std::{
    collections::BTreeMap,
    mem::MaybeUninit,
    ptr::{null, null_mut},
    sync::{LazyLock, Mutex, Once},
};

use slim_detours_sys::{DETOUR_INLINE_HOOK, SlimDetoursInlineHooks};
use sync_unsafe_cell::SyncUnsafeCell;
use widestring::U16CStr;
#[cfg(target_pointer_width = "64")]
use windows_sys::Win32::UI::WindowsAndMessaging::SetClassLongPtrW;
#[cfg(not(target_pointer_width = "64"))]
use windows_sys::Win32::UI::WindowsAndMessaging::SetClassLongW as SetClassLongPtrW;
use windows_sys::{
    Win32::{
        Foundation::{COLORREF, E_INVALIDARG, HWND, LPARAM, LRESULT, RECT, S_OK, WPARAM},
        Graphics::{
            Dwm::DwmSetWindowAttribute,
            Gdi::{
                BLACK_BRUSH, CreateSolidBrush, DC_BRUSH, DRAW_TEXT_FORMAT, FillRect, FrameRect,
                GetStockObject, HDC, InvalidateRect, SetDCBrushColor, WHITE_BRUSH,
            },
        },
        System::SystemServices::MAX_CLASS_NAME,
        UI::{
            Accessibility::{HCF_HIGHCONTRASTON, HIGHCONTRASTW},
            Controls::{
                BP_CHECKBOX, BP_RADIOBUTTON, CloseThemeData, DTBG_CLIPRECT, DTBGOPTS,
                DTT_TEXTCOLOR, DTTOPTS, DrawThemeBackground, DrawThemeBackgroundEx,
                DrawThemeParentBackground, DrawThemeText, DrawThemeTextEx, GetThemeColor, HTHEME,
                OPEN_THEME_DATA_FLAGS, OpenThemeData, OpenThemeDataEx, PBS_DISABLED,
                PFTASKDIALOGCALLBACK, PP_TRANSPARENTBAR, PROGRESS_CLASSW, SetWindowTheme,
                TABP_AEROWIZARDBODY, TABP_BODY, TABP_PANE, TABP_TABITEM, TABP_TABITEMBOTHEDGE,
                TABP_TABITEMLEFTEDGE, TABP_TABITEMRIGHTEDGE, TABP_TOPTABITEM,
                TABP_TOPTABITEMBOTHEDGE, TABP_TOPTABITEMLEFTEDGE, TABP_TOPTABITEMRIGHTEDGE,
                TASKDIALOG_NOTIFICATIONS, TDLG_MAININSTRUCTIONPANE, TDLG_PRIMARYPANEL,
                TDLG_SECONDARYPANEL, TDN_CREATED, TDN_DIALOG_CONSTRUCTED, TIS_DISABLED,
                TIS_FOCUSED, TIS_HOT, TIS_NORMAL, TIS_SELECTED, TMT_TEXTCOLOR, WC_BUTTONW,
                WC_COMBOBOXW, WC_EDITW, WC_TABCONTROLW,
            },
            HiDpi::OpenThemeDataForDpi,
            Shell::{DefSubclassProc, SetWindowSubclass},
            WindowsAndMessaging::{
                ES_MULTILINE, EnumChildWindows, GCLP_HBRBACKGROUND, GWL_STYLE, GetClassNameW,
                GetClientRect, GetWindowLongPtrW, SPI_GETHIGHCONTRAST, SystemParametersInfoW,
                WM_CTLCOLORDLG, WM_SETTINGCHANGE,
            },
        },
    },
    core::{BOOL, HRESULT, PCWSTR},
    w,
};

use super::{PreferredAppMode, WHITE, WinBrush, u16_string_eq_ignore_case};
use crate::darkmode::u16_string_starts_with_ignore_case;

#[link(name = "ntdll")]
extern "system" {
    fn RtlGetNtVersionNumbers(major: *mut u32, minor: *mut u32, build: *mut u32);
}

#[link(name = "uxtheme", kind = "raw-dylib")]
extern "system" {
    #[link_ordinal(132)]
    fn ShouldAppsUseDarkMode() -> bool;
    #[link_ordinal(133)]
    fn AllowDarkModeForWindow(h: HWND, a: bool) -> bool;
    // build < 18362
    #[link_ordinal(135)]
    fn AllowDarkModeForApp(a: bool) -> bool;
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

pub fn set_preferred_app_mode(m: PreferredAppMode) -> PreferredAppMode {
    unsafe {
        let build = get_nt_build();
        if build < 18362 {
            if AllowDarkModeForApp(
                m == PreferredAppMode::AllowDark || m == PreferredAppMode::ForceDark,
            ) {
                PreferredAppMode::AllowDark
            } else {
                PreferredAppMode::Default
            }
        } else {
            SetPreferredAppMode(m)
        }
    }
}

pub fn is_dark_mode_allowed_for_app() -> bool {
    unsafe {
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
        ((hc.dwFlags & HCF_HIGHCONTRASTON) == 0) && ShouldAppsUseDarkMode()
    }
}

const DWMWA_USE_IMMERSIVE_DARK_MODE: u32 = 0x13;
const DWMWA_USE_IMMERSIVE_DARK_MODE_V2: u32 = 0x14;

/// # Safety
/// `h_wnd` should be valid.
pub unsafe fn window_use_dark_mode(h_wnd: HWND) -> HRESULT {
    let set_dark_mode = is_dark_mode_allowed_for_app();
    let brush = if set_dark_mode {
        GetStockObject(BLACK_BRUSH)
    } else {
        GetStockObject(WHITE_BRUSH)
    };
    SetClassLongPtrW(h_wnd, GCLP_HBRBACKGROUND, brush as _);
    AllowDarkModeForWindow(h_wnd, set_dark_mode);
    let set_dark_mode = set_dark_mode as BOOL;
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

type OpenThemeDataFn = unsafe extern "system" fn(hwnd: HWND, pszclasslist: PCWSTR) -> HTHEME;
static TRUE_OPEN_THEME_DATA: SyncUnsafeCell<OpenThemeDataFn> = SyncUnsafeCell::new(OpenThemeData);

type OpenThemeDataExFn = unsafe extern "system" fn(
    hwnd: HWND,
    pszclasslist: PCWSTR,
    dwflags: OPEN_THEME_DATA_FLAGS,
) -> HTHEME;
static TRUE_OPEN_THEME_DATA_EX: SyncUnsafeCell<OpenThemeDataExFn> =
    SyncUnsafeCell::new(OpenThemeDataEx);

type OpenThemeDataForDpiFn =
    unsafe extern "system" fn(hwnd: HWND, pszclasslist: PCWSTR, dpi: u32) -> HTHEME;
static TRUE_OPEN_THEME_DATA_FOR_DPI: SyncUnsafeCell<OpenThemeDataForDpiFn> =
    SyncUnsafeCell::new(OpenThemeDataForDpi);

type CloseThemeDataFn = unsafe extern "system" fn(htheme: HTHEME) -> HRESULT;
static TRUE_CLOSE_THEME_DATA: SyncUnsafeCell<CloseThemeDataFn> =
    SyncUnsafeCell::new(CloseThemeData);

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

type DrawThemeParentBackgroundFn =
    unsafe extern "system" fn(hwnd: HWND, hdc: HDC, prect: *const RECT) -> HRESULT;
static TRUE_DRAW_THEME_PARENT_BACKGROUND: SyncUnsafeCell<DrawThemeParentBackgroundFn> =
    SyncUnsafeCell::new(DrawThemeParentBackground);

#[inline]
unsafe fn detour_hooks() -> [DETOUR_INLINE_HOOK; 9] {
    [
        DETOUR_INLINE_HOOK {
            pszFuncName: null(),
            ppPointer: TRUE_OPEN_THEME_DATA.get().cast(),
            pDetour: dark_open_theme_data as _,
        },
        DETOUR_INLINE_HOOK {
            pszFuncName: null(),
            ppPointer: TRUE_OPEN_THEME_DATA_EX.get().cast(),
            pDetour: dark_open_theme_data_ex as _,
        },
        DETOUR_INLINE_HOOK {
            pszFuncName: null(),
            ppPointer: TRUE_OPEN_THEME_DATA_FOR_DPI.get().cast(),
            pDetour: dark_open_theme_data_for_dpi as _,
        },
        DETOUR_INLINE_HOOK {
            pszFuncName: null(),
            ppPointer: TRUE_CLOSE_THEME_DATA.get().cast(),
            pDetour: dark_close_theme_data as _,
        },
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
        DETOUR_INLINE_HOOK {
            pszFuncName: null(),
            ppPointer: TRUE_DRAW_THEME_PARENT_BACKGROUND.get().cast(),
            pDetour: dark_draw_theme_parent_background as _,
        },
    ]
}

fn detour_attach() {
    unsafe {
        let mut hooks = detour_hooks();
        SlimDetoursInlineHooks(1, hooks.len() as _, hooks.as_mut_ptr());
    }
}

static DETOUR_GUARD: Once = Once::new();

pub fn init_dark() {
    DETOUR_GUARD.call_once(detour_attach);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ThemeType {
    Button,
    TaskDialog,
    Tab,
    Progress,
}

static HTHEME_MAP: Mutex<BTreeMap<HTHEME, (usize, ThemeType)>> = Mutex::new(BTreeMap::new());

unsafe fn on_theme_open(pszclasslist: PCWSTR, htheme: HTHEME) {
    let class_list = U16CStr::from_ptr_str(pszclasslist);
    let ty = if u16_string_starts_with_ignore_case(class_list, w!("Button")) {
        Some(ThemeType::Button)
    } else if u16_string_starts_with_ignore_case(class_list, w!("TaskDialog")) {
        Some(ThemeType::TaskDialog)
    } else if u16_string_eq_ignore_case(class_list, w!("Tab")) {
        Some(ThemeType::Tab)
    } else if u16_string_eq_ignore_case(class_list, w!("Progress"))
        || u16_string_eq_ignore_case(class_list, w!("Indeterminate::Progress"))
    {
        Some(ThemeType::Progress)
    } else {
        None
    };
    if let Some(ty) = ty {
        HTHEME_MAP
            .lock()
            .unwrap()
            .entry(htheme)
            .and_modify(|e| e.0 += 1)
            .or_insert((1, ty));
    }
}

unsafe extern "system" fn dark_open_theme_data(hwnd: HWND, pszclasslist: PCWSTR) -> HTHEME {
    let htheme = (*TRUE_OPEN_THEME_DATA.get())(hwnd, pszclasslist);
    if htheme == 0 {
        return htheme;
    }
    on_theme_open(pszclasslist, htheme);
    htheme
}

unsafe extern "system" fn dark_open_theme_data_ex(
    hwnd: HWND,
    pszclasslist: PCWSTR,
    dwflags: OPEN_THEME_DATA_FLAGS,
) -> HTHEME {
    let htheme = (*TRUE_OPEN_THEME_DATA_EX.get())(hwnd, pszclasslist, dwflags);
    if htheme == 0 {
        return htheme;
    }
    on_theme_open(pszclasslist, htheme);
    htheme
}

unsafe extern "system" fn dark_open_theme_data_for_dpi(
    hwnd: HWND,
    pszclasslist: PCWSTR,
    dpi: u32,
) -> HTHEME {
    let htheme = (*TRUE_OPEN_THEME_DATA_FOR_DPI.get())(hwnd, pszclasslist, dpi);
    if htheme == 0 {
        return htheme;
    }
    on_theme_open(pszclasslist, htheme);
    htheme
}

unsafe extern "system" fn dark_close_theme_data(htheme: HTHEME) -> HRESULT {
    let mut map = HTHEME_MAP.lock().unwrap();
    if let Some((count, _)) = map.get_mut(&htheme) {
        *count -= 1;
        if *count == 0 {
            map.remove(&htheme);
        }
    }
    (*TRUE_CLOSE_THEME_DATA.get())(htheme)
}

unsafe extern "system" fn dark_get_theme_color(
    htheme: HTHEME,
    ipartid: i32,
    istateid: i32,
    ipropid: i32,
    pcolor: *mut COLORREF,
) -> HRESULT {
    let res = (*TRUE_GET_THEME_COLOR.get())(htheme, ipartid, istateid, ipropid, pcolor);

    if is_dark_mode_allowed_for_app() {
        if let Some((_, ThemeType::TaskDialog)) = HTHEME_MAP.lock().unwrap().get(&htheme) {
            if ipropid == TMT_TEXTCOLOR as _ {
                if ipartid == TDLG_MAININSTRUCTIONPANE {
                    *pcolor = increase(*pcolor, 150);
                } else {
                    *pcolor = WHITE;
                }
            }
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
    if is_dark_mode_allowed_for_app() {
        if let Some((_, ty)) = HTHEME_MAP.lock().unwrap().get(&htheme) {
            if (ty == &ThemeType::Button
                && ((ipartid == BP_CHECKBOX || ipartid == BP_RADIOBUTTON)
                    && istateid != PBS_DISABLED))
                || (ty == &ThemeType::Tab)
            {
                let mut options: DTTOPTS = std::mem::zeroed();
                options.dwSize = std::mem::size_of::<DTTOPTS>() as _;
                options.dwFlags = DTT_TEXTCOLOR;
                options.crText = WHITE;
                return DrawThemeTextEx(
                    htheme,
                    hdc,
                    ipartid,
                    istateid,
                    psztext,
                    cchtext,
                    dwtextflags,
                    prect.cast_mut(),
                    &options,
                );
            }
        }
    }
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

unsafe extern "system" fn dark_draw_theme_background(
    htheme: HTHEME,
    hdc: HDC,
    ipartid: i32,
    istateid: i32,
    prect: *const RECT,
    pcliprect: *const RECT,
) -> HRESULT {
    let options = DTBGOPTS {
        dwSize: std::mem::size_of::<DTBGOPTS>() as _,
        dwFlags: if pcliprect.is_null() {
            0
        } else {
            DTBG_CLIPRECT
        },
        rcClip: if pcliprect.is_null() {
            RECT::default()
        } else {
            *pcliprect
        },
    };
    dark_draw_theme_background_ex(htheme, hdc, ipartid, istateid, prect, &options)
}

unsafe extern "system" fn dark_draw_theme_background_ex(
    htheme: HTHEME,
    hdc: HDC,
    ipartid: i32,
    istateid: i32,
    prect: *const RECT,
    poptions: *const DTBGOPTS,
) -> HRESULT {
    if is_dark_mode_allowed_for_app() {
        if let Some((_, ty)) = HTHEME_MAP.lock().unwrap().get(&htheme) {
            match ty {
                ThemeType::Progress => {
                    if ipartid == PP_TRANSPARENTBAR {
                        FillRect(hdc, prect, DLG_GRAY_BACK.0);
                        return S_OK;
                    }
                }
                ThemeType::Tab => match ipartid {
                    TABP_TABITEM
                    | TABP_TABITEMLEFTEDGE
                    | TABP_TABITEMRIGHTEDGE
                    | TABP_TABITEMBOTHEDGE
                    | TABP_TOPTABITEM
                    | TABP_TOPTABITEMLEFTEDGE
                    | TABP_TOPTABITEMRIGHTEDGE
                    | TABP_TOPTABITEMBOTHEDGE => {
                        let f = match istateid {
                            TIS_NORMAL => -0.75,
                            TIS_HOT => -0.8,
                            TIS_FOCUSED | TIS_SELECTED => -0.86,
                            TIS_DISABLED => -0.6,
                            _ => return E_INVALIDARG,
                        };
                        let res = adjust_luma(hdc, WHITE, prect, poptions, f);
                        if res >= 0 {
                            return res;
                        }
                    }
                    TABP_PANE | TABP_BODY | TABP_AEROWIZARDBODY => {
                        let res = adjust_luma(hdc, WHITE, prect, poptions, -0.86);
                        if res >= 0 {
                            FrameRect(hdc, prect, DLG_GRAY_BACK.0);
                            return res;
                        }
                    }
                    _ => {}
                },
                ThemeType::TaskDialog => {
                    match ipartid {
                        TDLG_PRIMARYPANEL => {
                            FillRect(hdc, prect, DLG_GRAY_BACK.0);
                            return S_OK;
                        }
                        TDLG_SECONDARYPANEL => {
                            FillRect(hdc, prect, DLG_DARK_BACK.0);
                            return S_OK;
                        }
                        _ => {}
                    };
                }
                _ => {}
            }
        }
    }
    (*TRUE_DRAW_THEME_BACKGROUND_EX.get())(htheme, hdc, ipartid, istateid, prect, poptions)
}

unsafe extern "system" fn dark_draw_theme_parent_background(
    hwnd: HWND,
    hdc: HDC,
    prect: *const RECT,
) -> HRESULT {
    if is_dark_mode_allowed_for_app() {
        let mut class_name = [0u16; MAX_CLASS_NAME as usize];
        GetClassNameW(hwnd, class_name.as_mut_ptr(), MAX_CLASS_NAME);
        let class_name = U16CStr::from_ptr_str(class_name.as_ptr());
        if u16_string_eq_ignore_case(class_name, WC_TABCONTROLW) {
            let rect = if prect.is_null() {
                let mut rect = MaybeUninit::uninit();
                GetClientRect(hwnd, rect.as_mut_ptr());
                rect.assume_init()
            } else {
                *prect
            };
            FillRect(hdc, &rect, DLG_GRAY_BACK.0);
            return S_OK;
        }
    }
    (*TRUE_DRAW_THEME_PARENT_BACKGROUND.get())(hwnd, hdc, prect)
}

fn delta_colorref_luma(cr: u32, d: i32) -> u32 {
    fn clamp(val: i32) -> u8 {
        val.clamp(0, 0xFF) as u8
    }
    let r = clamp((cr & 0xFF) as i32 + d);
    let g = clamp(((cr >> 8) & 0xFF) as i32 + d);
    let b = clamp(((cr >> 16) & 0xFF) as i32 + d);
    (r as u32) | ((g as u32) << 8) | ((b as u32) << 16)
}

fn adjust_luma(
    hdc: HDC,
    color: u32,
    prect: *const RECT,
    poptions: *const DTBGOPTS,
    delta: f64,
) -> HRESULT {
    let color = delta_colorref_luma(color, (delta * 255.0) as _);
    unsafe {
        SetDCBrushColor(hdc, color);
        let clip = if let Some(opt) = poptions.as_ref() {
            if (opt.dwFlags & DTBG_CLIPRECT) != 0 {
                &opt.rcClip
            } else {
                null()
            }
        } else {
            null()
        };
        if clip.is_null() {
            FillRect(hdc, prect, GetStockObject(DC_BRUSH));
        } else {
            FillRect(hdc, clip, GetStockObject(DC_BRUSH));
        }
    }
    S_OK
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

unsafe extern "system" fn task_dialog_callback(
    hwnd: HWND,
    msg: u32,
    _wparam: WPARAM,
    _lparam: LPARAM,
    lprefdata: isize,
) -> HRESULT {
    let msg = msg as TASKDIALOG_NOTIFICATIONS;
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

pub(crate) const TASK_DIALOG_CALLBACK: PFTASKDIALOGCALLBACK = Some(task_dialog_callback);

/// MISC: If in task dialog, set lparam to 1.
/// # Safety
/// `handle` should be valid.
pub unsafe fn children_refresh_dark_mode(handle: HWND, lparam: LPARAM) {
    unsafe extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        control_use_dark_mode(hwnd, lparam != 0);
        InvalidateRect(hwnd, null(), 1);
        EnumChildWindows(hwnd, Some(enum_callback), lparam);
        1
    }

    EnumChildWindows(handle, Some(enum_callback), lparam);
}

/// # Safety
/// `hwnd` should be valid.
pub unsafe fn control_use_dark_mode(hwnd: HWND, misc_task_dialog: bool) {
    let mut class = [0u16; MAX_CLASS_NAME as usize];
    GetClassNameW(hwnd, class.as_mut_ptr(), MAX_CLASS_NAME);
    let class = U16CStr::from_ptr_str(class.as_ptr());
    let subappname = if is_dark_mode_allowed_for_app() {
        if u16_string_eq_ignore_case(class, WC_COMBOBOXW) {
            w!("DarkMode_CFD")
        } else if u16_string_eq_ignore_case(class, WC_EDITW) {
            let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
            if style & ES_MULTILINE as isize != 0 {
                w!("DarkMode_Explorer")
            } else {
                w!("DarkMode_CFD")
            }
        } else if u16_string_eq_ignore_case(class, PROGRESS_CLASSW)
            || (u16_string_eq_ignore_case(class, WC_BUTTONW) && misc_task_dialog)
            || u16_string_starts_with_ignore_case(class, w!("WinioWindowVersion"))
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
        WM_CTLCOLORDLG => {
            if is_dark_mode_allowed_for_app() {
                return DLG_DARK_BACK.0 as _;
            }
        }
        _ => {}
    }
    DefSubclassProc(hwnd, umsg, wparam, lparam)
}
