#[cfg(feature = "once_cell_try")]
use std::sync::OnceLock;
use std::{
    collections::BTreeMap,
    mem::MaybeUninit,
    ptr::{null, null_mut},
    sync::{LazyLock, Mutex, Once},
};

use compio_log::error;
#[cfg(not(feature = "once_cell_try"))]
use once_cell::sync::OnceCell as OnceLock;
use slim_detours_sys::{DETOUR_INLINE_HOOK, SlimDetoursInlineHooks};
use sync_unsafe_cell::SyncUnsafeCell;
use windows_sys::{
    Win32::{
        Foundation::{
            COLORREF, E_INVALIDARG, HWND, LPARAM, LRESULT, RECT, S_OK, SIZE, SetLastError, WPARAM,
        },
        Graphics::{
            Dwm::DwmSetWindowAttribute,
            Gdi::{
                CreateSolidBrush, DC_BRUSH, DRAW_TEXT_FORMAT, DT_CENTER, DT_HIDEPREFIX,
                DT_SINGLELINE, DT_VCENTER, FillRect, FrameRect, GetStockObject, GetWindowDC, HDC,
                InvalidateRect, MapWindowPoints, OffsetRect, ReleaseDC, SetDCBrushColor,
                TRANSPARENT,
            },
        },
        System::SystemServices::MAX_CLASS_NAME,
        UI::{
            Accessibility::{HCF_HIGHCONTRASTON, HIGHCONTRASTW},
            Controls::{
                BP_CHECKBOX, BP_GROUPBOX, BP_RADIOBUTTON, CDDS_ITEMPREPAINT, CDDS_PREPAINT,
                CDIS_CHECKED, CDIS_DISABLED, CDIS_HOT, CDIS_SELECTED, CDRF_NOTIFYITEMDRAW,
                CloseThemeData, DRAWITEMSTRUCT, DTBG_CLIPRECT, DTBGOPTS, DTT_TEXTCOLOR, DTTOPTS,
                DrawThemeBackground, DrawThemeBackgroundEx, DrawThemeParentBackground,
                DrawThemeText, DrawThemeTextEx, GBS_DISABLED, GetThemeColor, HTHEME, MBI_DISABLED,
                MBI_DISABLEDHOT, MBI_DISABLEDPUSHED, MBI_HOT, MBI_NORMAL, MBI_PUSHED, MENU_BARITEM,
                NM_CUSTOMDRAW, NMHDR, NMTBCUSTOMDRAW, ODS_DISABLED, ODS_GRAYED, ODS_INACTIVE,
                ODS_NOACCEL, ODS_SELECTED, OPEN_THEME_DATA_FLAGS, OpenThemeData, OpenThemeDataEx,
                PBS_DISABLED, PFTASKDIALOGCALLBACK, PP_TRANSPARENTBAR, PROGRESS_CLASSW,
                SetWindowTheme, TABP_AEROWIZARDBODY, TABP_BODY, TABP_PANE, TABP_TABITEM,
                TABP_TABITEMBOTHEDGE, TABP_TABITEMLEFTEDGE, TABP_TABITEMRIGHTEDGE, TABP_TOPTABITEM,
                TABP_TOPTABITEMBOTHEDGE, TABP_TOPTABITEMLEFTEDGE, TABP_TOPTABITEMRIGHTEDGE,
                TASKDIALOG_NOTIFICATIONS, TBCDRF_HILITEHOTTRACK, TBCDRF_NOBACKGROUND,
                TBCDRF_USECDCOLORS, TDLG_MAININSTRUCTIONPANE, TDLG_PRIMARYPANEL,
                TDLG_SECONDARYPANEL, TDN_CREATED, TDN_DIALOG_CONSTRUCTED, TIS_DISABLED,
                TIS_FOCUSED, TIS_HOT, TIS_NORMAL, TIS_SELECTED, TMT_TEXTCOLOR, TOOLBARCLASSNAMEW,
                VSCLASS_MENU, WC_BUTTONW, WC_COMBOBOXW, WC_EDITW, WC_TABCONTROLW,
            },
            HiDpi::OpenThemeDataForDpi,
            Shell::{DefSubclassProc, SetWindowSubclass},
            WindowsAndMessaging::{
                DefWindowProcW, ES_MULTILINE, EnumChildWindows, GWL_STYLE, GetClassNameW,
                GetClientRect, GetMenuBarInfo, GetMenuItemInfoW, GetWindowLongPtrW, GetWindowRect,
                HMENU, MENUBARINFO, MENUITEMINFOW, MIIM_STRING, OBJID_MENU, SPI_GETHIGHCONTRAST,
                SystemParametersInfoW, WM_CTLCOLORDLG, WM_NCACTIVATE, WM_NCPAINT, WM_NOTIFY,
                WM_SETTINGCHANGE,
            },
        },
    },
    core::{BOOL, HRESULT, PCWSTR},
    w,
};

use super::{PreferredAppMode, WHITE, WinBrush, u16_string_eq_ignore_case};
use crate::{Error, Result, darkmode::u16_string_starts_with_ignore_case, get_nt_build, syscall};

#[link(name = "uxtheme", kind = "raw-dylib")]
unsafe extern "system" {
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
            error!("SystemParametersInfoW: {:?}", Error::from_thread());
            return false;
        }
        ((hc.dwFlags & HCF_HIGHCONTRASTON) == 0) && ShouldAppsUseDarkMode()
    }
}

const DWMWA_USE_IMMERSIVE_DARK_MODE: u32 = 0x13;
const DWMWA_USE_IMMERSIVE_DARK_MODE_V2: u32 = 0x14;

/// # Safety
/// `h_wnd` should be valid.
pub unsafe fn window_use_dark_mode(h_wnd: HWND) -> Result<()> {
    unsafe {
        let set_dark_mode = is_dark_mode_allowed_for_app();
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
                return windows_core::HRESULT(hr).ok();
            }
        }
        FlushMenuThemes();
        Ok(())
    }
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

fn detour_attach() -> Result<()> {
    unsafe {
        let mut hooks = detour_hooks();
        let res = SlimDetoursInlineHooks(1, hooks.len() as _, hooks.as_mut_ptr());
        windows_core::HRESULT(res).ok()
    }
}

static DETOUR_GUARD: Once = Once::new();

pub fn init_dark() {
    DETOUR_GUARD.call_once(|| {
        if let Err(_e) = detour_attach() {
            error!("Failed to hook uxtheme.dll: {_e:?}");
        }
    });
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
    unsafe {
        let ty = if u16_string_starts_with_ignore_case(pszclasslist, w!("Button")) {
            Some(ThemeType::Button)
        } else if u16_string_starts_with_ignore_case(pszclasslist, w!("TaskDialog")) {
            Some(ThemeType::TaskDialog)
        } else if u16_string_eq_ignore_case(pszclasslist, w!("Tab")) {
            Some(ThemeType::Tab)
        } else if u16_string_eq_ignore_case(pszclasslist, w!("Progress"))
            || u16_string_eq_ignore_case(pszclasslist, w!("Indeterminate::Progress"))
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
}

unsafe extern "system" fn dark_open_theme_data(hwnd: HWND, pszclasslist: PCWSTR) -> HTHEME {
    unsafe {
        let htheme = (*TRUE_OPEN_THEME_DATA.get())(hwnd, pszclasslist);
        if htheme == 0 {
            return htheme;
        }
        on_theme_open(pszclasslist, htheme);
        htheme
    }
}

unsafe extern "system" fn dark_open_theme_data_ex(
    hwnd: HWND,
    pszclasslist: PCWSTR,
    dwflags: OPEN_THEME_DATA_FLAGS,
) -> HTHEME {
    unsafe {
        let htheme = (*TRUE_OPEN_THEME_DATA_EX.get())(hwnd, pszclasslist, dwflags);
        if htheme == 0 {
            return htheme;
        }
        on_theme_open(pszclasslist, htheme);
        htheme
    }
}

unsafe extern "system" fn dark_open_theme_data_for_dpi(
    hwnd: HWND,
    pszclasslist: PCWSTR,
    dpi: u32,
) -> HTHEME {
    unsafe {
        let htheme = (*TRUE_OPEN_THEME_DATA_FOR_DPI.get())(hwnd, pszclasslist, dpi);
        if htheme == 0 {
            return htheme;
        }
        on_theme_open(pszclasslist, htheme);
        htheme
    }
}

unsafe extern "system" fn dark_close_theme_data(htheme: HTHEME) -> HRESULT {
    let mut map = HTHEME_MAP.lock().unwrap();
    if let Some((count, _)) = map.get_mut(&htheme) {
        *count -= 1;
        if *count == 0 {
            map.remove(&htheme);
        }
    }
    unsafe { (*TRUE_CLOSE_THEME_DATA.get())(htheme) }
}

unsafe extern "system" fn dark_get_theme_color(
    htheme: HTHEME,
    ipartid: i32,
    istateid: i32,
    ipropid: i32,
    pcolor: *mut COLORREF,
) -> HRESULT {
    let res = unsafe { (*TRUE_GET_THEME_COLOR.get())(htheme, ipartid, istateid, ipropid, pcolor) };

    if is_dark_mode_allowed_for_app()
        && let Some((_, ThemeType::TaskDialog)) = HTHEME_MAP.lock().unwrap().get(&htheme)
        && ipropid == TMT_TEXTCOLOR as _
        && let Some(pcolor) = unsafe { pcolor.as_mut() }
    {
        if ipartid == TDLG_MAININSTRUCTIONPANE {
            *pcolor = increase(*pcolor, 150);
        } else {
            *pcolor = WHITE;
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
        && let Some((_, ty)) = HTHEME_MAP.lock().unwrap().get(&htheme)
        && ((ty == &ThemeType::Button
            && ((ipartid == BP_GROUPBOX && istateid != GBS_DISABLED)
                || ((ipartid == BP_CHECKBOX || ipartid == BP_RADIOBUTTON)
                    && istateid != PBS_DISABLED)))
            || (ty == &ThemeType::Tab))
    {
        let mut options: DTTOPTS = unsafe { std::mem::zeroed() };
        options.dwSize = size_of::<DTTOPTS>() as _;
        options.dwFlags = DTT_TEXTCOLOR;
        options.crText = WHITE;
        return unsafe {
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
        };
    }
    unsafe {
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
    let options = DTBGOPTS {
        dwSize: size_of::<DTBGOPTS>() as _,
        dwFlags: if pcliprect.is_null() {
            0
        } else {
            DTBG_CLIPRECT
        },
        rcClip: unsafe { pcliprect.as_ref() }.cloned().unwrap_or_default(),
    };
    unsafe { dark_draw_theme_background_ex(htheme, hdc, ipartid, istateid, prect, &options) }
}

unsafe extern "system" fn dark_draw_theme_background_ex(
    htheme: HTHEME,
    hdc: HDC,
    ipartid: i32,
    istateid: i32,
    prect: *const RECT,
    poptions: *const DTBGOPTS,
) -> HRESULT {
    if is_dark_mode_allowed_for_app()
        && let Some((_, ty)) = HTHEME_MAP.lock().unwrap().get(&htheme)
    {
        match ty {
            ThemeType::Progress if ipartid == PP_TRANSPARENTBAR => {
                unsafe { FillRect(hdc, prect, DLG_GRAY_BACK.0) };
                return S_OK;
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
                        unsafe { FrameRect(hdc, prect, DLG_GRAY_BACK.0) };
                        return res;
                    }
                }
                _ => {}
            },
            ThemeType::TaskDialog => {
                match ipartid {
                    TDLG_PRIMARYPANEL => {
                        unsafe { FillRect(hdc, prect, DLG_GRAY_BACK.0) };
                        return S_OK;
                    }
                    TDLG_SECONDARYPANEL => {
                        unsafe { FillRect(hdc, prect, DLG_DARK_BACK.0) };
                        return S_OK;
                    }
                    _ => {}
                };
            }
            _ => {}
        }
    }
    unsafe {
        (*TRUE_DRAW_THEME_BACKGROUND_EX.get())(htheme, hdc, ipartid, istateid, prect, poptions)
    }
}

unsafe extern "system" fn dark_draw_theme_parent_background(
    hwnd: HWND,
    hdc: HDC,
    prect: *const RECT,
) -> HRESULT {
    unsafe {
        if is_dark_mode_allowed_for_app() {
            let mut class_name = [0u16; MAX_CLASS_NAME as usize];
            let res = GetClassNameW(hwnd, class_name.as_mut_ptr(), MAX_CLASS_NAME);
            if res != 0 {
                if u16_string_eq_ignore_case(class_name.as_ptr(), WC_TABCONTROLW) {
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
            } else {
                error!("GetClassNameW: {:?}", Error::from_thread());
            }
        }
        (*TRUE_DRAW_THEME_PARENT_BACKGROUND.get())(hwnd, hdc, prect)
    }
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

static TOOLBAR_CHECKED_BACK: LazyLock<WinBrush> =
    LazyLock::new(|| WinBrush(unsafe { CreateSolidBrush(TOOLBAR_CHECKED_COLOR) }));

static TOOLBAR_HOT_BACK: LazyLock<WinBrush> =
    LazyLock::new(|| WinBrush(unsafe { CreateSolidBrush(TOOLBAR_HOT_COLOR) }));

const TOOLBAR_TEXT: COLORREF = 0x00FFFFFF;
const TOOLBAR_DISABLED_TEXT: COLORREF = 0x00808080;
const TOOLBAR_CHECKED_COLOR: COLORREF = 0x002A2A2A;
const TOOLBAR_HOT_COLOR: COLORREF = 0x00343434;

fn increase(c: COLORREF, inc: u32) -> COLORREF {
    let r = c & 0xFF;
    let r = (r + inc).min(0xFF);
    let g = (c >> 8) & 0xFF;
    let g = (g + inc).min(0xFF);
    let b = (c >> 16) & 0xFF;
    let b = (b + inc).min(0xFF);
    r + (g << 8) + (b << 16)
}

/// MISC: If in task dialog, set lparam to 1.
/// # Safety
/// `handle` should be valid.
pub unsafe fn children_refresh_dark_mode(handle: HWND, lparam: LPARAM) {
    unsafe extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        unsafe {
            if control_use_dark_mode(hwnd, lparam != 0).is_err() {
                error!("control_use_dark_mode: {:?}", Error::from_thread());
                return 0;
            }
            if InvalidateRect(hwnd, null(), 1) == 0 {
                error!("InvalidateRect: {:?}", Error::from_thread());
                return 0;
            }
            EnumChildWindows(hwnd, Some(enum_callback), lparam);
            1
        }
    }

    unsafe {
        EnumChildWindows(handle, Some(enum_callback), lparam);
    }
}

/// # Safety
/// `hwnd` should be valid.
pub unsafe fn control_use_dark_mode(hwnd: HWND, misc_task_dialog: bool) -> Result<()> {
    unsafe {
        let mut class = [0u16; MAX_CLASS_NAME as usize];
        let res = GetClassNameW(hwnd, class.as_mut_ptr(), MAX_CLASS_NAME);
        if res == 0 {
            return Err(Error::from_thread());
        }
        let subappname = if is_dark_mode_allowed_for_app() {
            if u16_string_eq_ignore_case(class.as_ptr(), WC_COMBOBOXW) {
                w!("DarkMode_CFD")
            } else if u16_string_eq_ignore_case(class.as_ptr(), WC_EDITW) {
                let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
                if style as i32 & ES_MULTILINE != 0 {
                    w!("DarkMode_Explorer")
                } else {
                    w!("DarkMode_CFD")
                }
            } else if u16_string_eq_ignore_case(class.as_ptr(), PROGRESS_CLASSW)
                || (u16_string_eq_ignore_case(class.as_ptr(), WC_BUTTONW) && misc_task_dialog)
            {
                null()
            } else {
                w!("DarkMode_Explorer")
            }
        } else if u16_string_eq_ignore_case(class.as_ptr(), WC_BUTTONW) && misc_task_dialog {
            w!("DarkMode_Explorer")
        } else {
            null()
        };
        let res = SetWindowTheme(hwnd, subappname, null());
        windows_core::HRESULT(res).ok()
    }
}

fn task_dialog_refresh(hwnd: HWND) -> Result<()> {
    unsafe {
        window_use_dark_mode(hwnd)?;
        children_refresh_dark_mode(hwnd, 0);
    }
    Ok(())
}

pub(crate) const TASK_DIALOG_CALLBACK: PFTASKDIALOGCALLBACK = Some(task_dialog_callback);

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
            if let Err(_e) = task_dialog_refresh(hwnd) {
                error!("task_dialog_refresh: {_e:?}");
            }
        }
        _ => {}
    }
    if msg == TDN_CREATED {
        unsafe { SetWindowSubclass(hwnd, Some(task_dialog_subclass), hwnd as _, lprefdata as _) };
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
            if let Err(_e) = task_dialog_refresh(hwnd) {
                error!("task_dialog_refresh: {_e:?}");
            }
        }
        WM_CTLCOLORDLG if is_dark_mode_allowed_for_app() => {
            return DLG_DARK_BACK.0 as _;
        }
        _ => {}
    }
    unsafe { DefSubclassProc(hwnd, umsg, wparam, lparam) }
}

const WM_UAHDRAWMENU: u32 = 0x0091;
const WM_UAHDRAWMENUITEM: u32 = 0x0092;

/// # Safety
/// `handle` should be valid.
pub unsafe fn handle_wnd_proc(
    handle: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> Option<LRESULT> {
    if !is_dark_mode_allowed_for_app() {
        return None;
    }
    match msg {
        WM_NOTIFY => unsafe { toolbar_custom_draw(lparam) },
        WM_NCPAINT | WM_NCACTIVATE => {
            let ret = unsafe { DefWindowProcW(handle, msg, wparam, lparam) };
            if let Err(e) = unsafe { paint_nc_menu(handle) } {
                error!("WM_NCPAINT: {e:?}");
            }
            Some(ret)
        }
        WM_UAHDRAWMENU => {
            let menu = unsafe { &*(lparam as *const UAHMENU) };
            if let Err(e) = unsafe { paint_menu_bar(handle, menu.hdc) } {
                error!("WM_UAHDRAWMENU: {e:?}");
            }
            Some(0)
        }
        WM_UAHDRAWMENUITEM => {
            let udmi = unsafe { &*(lparam as *const UAHDRAWMENUITEM) };
            if let Err(e) = unsafe { paint_menu_item(handle, udmi) } {
                error!("WM_UAHDRAWMENUITEM: {e:?}");
            }
            Some(0)
        }
        _ => None,
    }
}

/// Paint toolbar items with dark colors.
///
/// # Safety
/// `lparam` must be the `lParam` of a `WM_NOTIFY` message.
unsafe fn toolbar_custom_draw(lparam: LPARAM) -> Option<LRESULT> {
    let header = unsafe { &*(lparam as *const NMHDR) };
    if header.code != NM_CUSTOMDRAW {
        return None;
    }

    let mut class = [0u16; MAX_CLASS_NAME as usize];
    if unsafe { GetClassNameW(header.hwndFrom, class.as_mut_ptr(), MAX_CLASS_NAME) } == 0 {
        return None;
    }
    if !unsafe { u16_string_eq_ignore_case(class.as_ptr(), TOOLBARCLASSNAMEW) } {
        return None;
    }

    let nm = unsafe { &mut *(lparam as *mut NMTBCUSTOMDRAW) };
    match nm.nmcd.dwDrawStage {
        CDDS_PREPAINT => Some(CDRF_NOTIFYITEMDRAW as LRESULT),
        CDDS_ITEMPREPAINT => {
            let state = nm.nmcd.uItemState;
            let disabled = state & CDIS_DISABLED != 0;
            let checked = state & (CDIS_CHECKED | CDIS_SELECTED) != 0;
            let hot = state & CDIS_HOT != 0;

            nm.clrText = if disabled {
                TOOLBAR_DISABLED_TEXT
            } else {
                TOOLBAR_TEXT
            };
            nm.clrTextHighlight = nm.clrText;
            nm.nStringBkMode = TRANSPARENT as _;
            nm.nHLStringBkMode = TRANSPARENT as _;

            let brush = if checked {
                Some(TOOLBAR_CHECKED_BACK.0)
            } else if hot {
                Some(TOOLBAR_HOT_BACK.0)
            } else {
                None
            };

            if let Some(brush) = brush {
                nm.clrBtnFace = if checked {
                    TOOLBAR_CHECKED_COLOR
                } else {
                    TOOLBAR_HOT_COLOR
                };
                nm.clrBtnHighlight = nm.clrBtnFace;
                nm.clrHighlightHotTrack = nm.clrBtnFace;
                unsafe { FillRect(nm.nmcd.hdc, &nm.nmcd.rc, brush) };
            }

            let mut flags = TBCDRF_NOBACKGROUND | TBCDRF_USECDCOLORS;
            if hot {
                flags |= TBCDRF_HILITEHOTTRACK;
            }
            Some(flags as LRESULT)
        }
        _ => None,
    }
}

#[repr(C)]
#[allow(clippy::upper_case_acronyms)]
struct UAHMENU {
    hmenu: HMENU,
    hdc: HDC,
    dwflags: u32,
}

unsafe fn paint_menu_bar(hwnd: HWND, hdc: HDC) -> Result<()> {
    let mut mbi: MENUBARINFO = unsafe { std::mem::zeroed() };
    mbi.cbSize = size_of::<MENUBARINFO>() as _;
    syscall!(BOOL, GetMenuBarInfo(hwnd, OBJID_MENU, 0, &mut mbi))?;

    let mut rc_window = MaybeUninit::uninit();
    syscall!(BOOL, GetWindowRect(hwnd, rc_window.as_mut_ptr()))?;
    let rc_window = unsafe { rc_window.assume_init() };
    let mut rcbar = mbi.rcBar;
    syscall!(
        BOOL,
        OffsetRect(&mut rcbar, -rc_window.left, -rc_window.top)
    )?;

    syscall!(BOOL, FillRect(hdc, &rcbar, DLG_GRAY_BACK.0))?;
    Ok(())
}

#[repr(C)]
#[allow(clippy::upper_case_acronyms)]
struct UAHMENUITEMMETRICS {
    rgsizebar: [SIZE; 2],
    rgsizepopup: [SIZE; 4],
}

#[repr(C)]
#[allow(clippy::upper_case_acronyms)]
struct UAHMENUPOPUPMETRICS {
    rgcx: [i32; 4],
    dummy: i32,
}

#[repr(C)]
#[allow(clippy::upper_case_acronyms)]
struct UAHMENUITEM {
    position: u32,
    umim: UAHMENUITEMMETRICS,
    umpm: UAHMENUPOPUPMETRICS,
}

#[repr(C)]
#[allow(clippy::upper_case_acronyms)]
struct UAHDRAWMENUITEM {
    dis: DRAWITEMSTRUCT,
    um: UAHMENU,
    umi: UAHMENUITEM,
}

struct Theme(HTHEME);

impl Theme {
    pub fn new(hwnd: HWND, pszclasslist: PCWSTR) -> Result<Self> {
        let htheme = unsafe { OpenThemeData(hwnd, pszclasslist) };
        if htheme == 0 {
            return Err(Error::from_thread());
        }
        Ok(Self(htheme))
    }
}

impl Drop for Theme {
    fn drop(&mut self) {
        unsafe { CloseThemeData(self.0) };
    }
}

static VSCLASS_MENU_THEME: OnceLock<Theme> = OnceLock::new();

unsafe fn paint_menu_item(hwnd: HWND, udmi: &UAHDRAWMENUITEM) -> Result<()> {
    let theme = VSCLASS_MENU_THEME.get_or_try_init(|| Theme::new(hwnd, VSCLASS_MENU))?;

    let mut mii: MENUITEMINFOW = unsafe { std::mem::zeroed() };
    mii.cbSize = size_of::<MENUITEMINFOW>() as _;
    mii.fMask = MIIM_STRING;
    syscall!(
        BOOL,
        GetMenuItemInfoW(udmi.um.hmenu, udmi.umi.position, 1, &mut mii)
    )?;

    let len = mii.cch as usize;
    let mut buf = Vec::with_capacity(len + 1);
    mii.dwTypeData = buf.as_mut_ptr();
    mii.cch += 1;
    syscall!(
        BOOL,
        GetMenuItemInfoW(udmi.um.hmenu, udmi.umi.position, 1, &mut mii)
    )?;
    unsafe {
        buf.set_len(len + 1);
    }

    const fn has_flag(flags: u32, flag: u32) -> bool {
        (flags & flag) == flag
    }

    let mut flags = DT_CENTER | DT_SINGLELINE | DT_VCENTER;

    let mut text_state_id = MBI_NORMAL;
    let mut bkg_state_id = MBI_NORMAL;
    if has_flag(udmi.dis.itemState, ODS_SELECTED) {
        text_state_id = MBI_PUSHED;
        bkg_state_id = MBI_PUSHED;
    } else if has_flag(udmi.dis.itemState, ODS_GRAYED)
        || has_flag(udmi.dis.itemState, ODS_DISABLED)
        || has_flag(udmi.dis.itemState, ODS_INACTIVE)
    {
        text_state_id = MBI_DISABLED;
        bkg_state_id = MBI_DISABLED;
    }

    if has_flag(udmi.dis.itemState, ODS_NOACCEL) {
        flags |= DT_HIDEPREFIX;
    }

    match bkg_state_id {
        MBI_NORMAL | MBI_DISABLED => unsafe {
            FillRect(udmi.um.hdc, &udmi.dis.rcItem, DLG_GRAY_BACK.0);
        },
        MBI_HOT | MBI_DISABLEDHOT | MBI_PUSHED | MBI_DISABLEDPUSHED => unsafe {
            FillRect(udmi.um.hdc, &udmi.dis.rcItem, DLG_DARK_BACK.0);
        },
        _ => unsafe {
            DrawThemeBackground(
                theme.0,
                udmi.um.hdc,
                MENU_BARITEM,
                bkg_state_id,
                &udmi.dis.rcItem,
                null(),
            );
        },
    }

    let mut dttopts: DTTOPTS = unsafe { std::mem::zeroed() };
    dttopts.dwSize = size_of::<DTTOPTS>() as _;
    dttopts.dwFlags = DTT_TEXTCOLOR;
    match text_state_id {
        MBI_NORMAL | MBI_HOT | MBI_PUSHED => {
            dttopts.crText = WHITE;
        }
        MBI_DISABLED | MBI_DISABLEDHOT | MBI_DISABLEDPUSHED => {
            dttopts.crText = increase(WHITE, 150);
        }
        _ => {}
    }

    let mut rcitem = udmi.dis.rcItem;
    unsafe {
        DrawThemeTextEx(
            theme.0,
            udmi.um.hdc,
            MENU_BARITEM,
            text_state_id,
            mii.dwTypeData,
            mii.cch as i32,
            flags,
            &mut rcitem,
            &dttopts,
        );
    }

    Ok(())
}

unsafe fn paint_nc_menu(hwnd: HWND) -> Result<()> {
    let mut mbi: MENUBARINFO = unsafe { std::mem::zeroed() };
    mbi.cbSize = size_of::<MENUBARINFO>() as _;
    let res = unsafe { GetMenuBarInfo(hwnd, OBJID_MENU, 0, &mut mbi) };
    if res == 0 {
        return Ok(());
    }
    let mut rc_client = MaybeUninit::uninit();
    syscall!(BOOL, GetClientRect(hwnd, rc_client.as_mut_ptr()))?;
    unsafe { SetLastError(0) };
    syscall!(
        BOOL,
        MapWindowPoints(hwnd, null_mut(), rc_client.as_mut_ptr().cast(), 2)
    )?;

    let mut rc_window = MaybeUninit::uninit();
    syscall!(BOOL, GetWindowRect(hwnd, rc_window.as_mut_ptr()))?;
    let rc_window = unsafe { rc_window.assume_init() };
    syscall!(
        BOOL,
        OffsetRect(rc_client.as_mut_ptr(), -rc_window.left, -rc_window.top)
    )?;

    let mut rc_client = unsafe { rc_client.assume_init() };
    rc_client.bottom = rc_client.top;
    rc_client.top -= 1;

    let hdc = unsafe { GetWindowDC(hwnd) };
    if hdc.is_null() {
        return Err(Error::from_thread());
    }

    struct DC(HDC, HWND);

    impl Drop for DC {
        fn drop(&mut self) {
            unsafe { ReleaseDC(self.1, self.0) };
        }
    }

    let hdc = DC(hdc, hwnd);

    syscall!(BOOL, FillRect(hdc.0, &rc_client, DLG_GRAY_BACK.0))?;
    Ok(())
}
