use std::{io, ptr::null, rc::Rc};

use windows::{
    core::ComInterface,
    Win32::Graphics::{
        Direct2D::{
            Common::{D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_COLOR_F, D2D1_PIXEL_FORMAT, D2D_SIZE_U},
            D2D1CreateFactory, ID2D1Factory, ID2D1HwndRenderTarget, ID2D1RenderTarget,
            D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1_FEATURE_LEVEL_DEFAULT,
            D2D1_HWND_RENDER_TARGET_PROPERTIES, D2D1_PRESENT_OPTIONS_NONE,
            D2D1_RENDER_TARGET_PROPERTIES, D2D1_RENDER_TARGET_TYPE_DEFAULT,
            D2D1_RENDER_TARGET_USAGE_NONE,
        },
        DirectWrite::{DWriteCreateFactory, IDWriteFactory, DWRITE_FACTORY_TYPE_SHARED},
        Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM,
    },
};
use windows_sys::Win32::{
    Foundation::HWND,
    Graphics::Gdi::InvalidateRect,
    System::SystemServices::SS_OWNERDRAW,
    UI::{
        Controls::{DRAWITEMSTRUCT, WC_STATICW},
        WindowsAndMessaging::{WM_DRAWITEM, WS_CHILD, WS_VISIBLE},
    },
};

use crate::{
    drawing::{Color, Point, Size},
    syscall_bool,
    window::{AsRawWindow, Widget},
};

#[derive(Debug, Clone)]
pub struct Canvas {
    handle: Rc<Widget>,
    d2d: ID2D1Factory,
    dwrite: IDWriteFactory,
    target: ID2D1HwndRenderTarget,
}

impl Canvas {
    pub fn new(parent: &impl AsRawWindow) -> io::Result<Self> {
        let handle = Widget::new(
            WC_STATICW,
            WS_CHILD | WS_VISIBLE | SS_OWNERDRAW,
            0,
            parent.as_raw_window(),
        )?;
        let d2d: ID2D1Factory =
            unsafe { D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)? };
        let dwrite = unsafe { DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)? };
        let target = unsafe {
            d2d.CreateHwndRenderTarget(
                &D2D1_RENDER_TARGET_PROPERTIES {
                    r#type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
                    pixelFormat: D2D1_PIXEL_FORMAT {
                        format: DXGI_FORMAT_B8G8R8A8_UNORM,
                        alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                    },
                    dpiX: 0.0,
                    dpiY: 0.0,
                    usage: D2D1_RENDER_TARGET_USAGE_NONE,
                    minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
                },
                &D2D1_HWND_RENDER_TARGET_PROPERTIES {
                    hwnd: windows::Win32::Foundation::HWND(handle.as_raw_window()),
                    pixelSize: D2D_SIZE_U::default(),
                    presentOptions: D2D1_PRESENT_OPTIONS_NONE,
                },
            )?
        };
        Ok(Self {
            handle: Rc::new(handle),
            d2d,
            dwrite,
            target,
        })
    }

    pub fn loc(&self) -> io::Result<Point> {
        self.handle.loc()
    }

    pub fn set_loc(&self, p: Point) -> io::Result<()> {
        self.handle.set_loc(p)
    }

    pub fn size(&self) -> io::Result<Size> {
        self.handle.size()
    }

    pub fn set_size(&self, v: Size) -> io::Result<()> {
        self.handle.set_size(v)
    }

    pub fn redraw(&self) -> io::Result<()> {
        syscall_bool(unsafe { InvalidateRect(self.as_raw_window(), null(), 0) })?;
        Ok(())
    }

    pub async fn wait_redraw(&self) {
        loop {
            let msg = self.handle.wait_parent(WM_DRAWITEM).await;
            let ds = unsafe { &mut *(msg.lParam as *mut DRAWITEMSTRUCT) };
            if ds.hwndItem == self.as_raw_window() {
                break;
            }
        }
    }

    pub fn context(&self) -> io::Result<DrawingContext> {
        unsafe {
            let dpi = self.handle.dpi();
            let size = self.handle.size_l2d(self.handle.size()?);
            self.target.Resize(&D2D_SIZE_U {
                width: size.0 as u32,
                height: size.1 as u32,
            })?;
            self.target.BeginDraw();
            self.target
                .Clear(Some(&color_f(Color::new(255, 255, 255, 255))));
            self.target.SetDpi(dpi as f32, dpi as f32);
            let ctx = DrawingContext {
                target: self.target.clone().cast()?,
                d2d: self.d2d.clone(),
                dwrite: self.dwrite.clone(),
            };
            Ok(ctx)
        }
    }
}

impl AsRawWindow for Canvas {
    fn as_raw_window(&self) -> HWND {
        self.handle.as_raw_window()
    }
}

#[inline]
fn color_f(c: Color) -> D2D1_COLOR_F {
    D2D1_COLOR_F {
        r: c.r as f32 / 255.0,
        g: c.g as f32 / 255.0,
        b: c.b as f32 / 255.0,
        a: c.a as f32 / 255.0,
    }
}

pub struct DrawingContext {
    target: ID2D1RenderTarget,
    d2d: ID2D1Factory,
    dwrite: IDWriteFactory,
}

impl Drop for DrawingContext {
    fn drop(&mut self) {
        unsafe { self.target.EndDraw(None, None) }.unwrap();
    }
}
