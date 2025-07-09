use std::{marker::PhantomData, mem::ManuallyDrop, ptr::null_mut};

use image::DynamicImage;
use inherit_methods_macro::inherit_methods;
use windows::{
    Win32::{
        Foundation::HMODULE,
        Graphics::{
            Direct2D::{
                Common::{D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_PIXEL_FORMAT},
                D2D1_BITMAP_OPTIONS_CANNOT_DRAW, D2D1_BITMAP_OPTIONS_TARGET,
                D2D1_BITMAP_PROPERTIES1, D2D1_DEVICE_CONTEXT_OPTIONS_NONE,
                D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1_TEXT_ANTIALIAS_MODE_CLEARTYPE,
                D2D1CreateFactory, ID2D1Bitmap1, ID2D1Device, ID2D1DeviceContext, ID2D1Factory,
                ID2D1Factory2, ID2D1RenderTarget,
            },
            Direct3D::{
                D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL_9_1, D3D_FEATURE_LEVEL_9_2,
                D3D_FEATURE_LEVEL_9_3, D3D_FEATURE_LEVEL_10_0, D3D_FEATURE_LEVEL_10_1,
                D3D_FEATURE_LEVEL_11_0, D3D_FEATURE_LEVEL_11_1,
            },
            Direct3D11::{
                D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_SDK_VERSION, D3D11CreateDevice,
                ID3D11Device, ID3D11DeviceContext,
            },
            DirectWrite::{DWRITE_FACTORY_TYPE_SHARED, DWriteCreateFactory, IDWriteFactory},
            Dxgi::{
                Common::{
                    DXGI_ALPHA_MODE_PREMULTIPLIED, DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC,
                },
                DXGI_MATRIX_3X2_F, DXGI_PRESENT, DXGI_SCALING_STRETCH, DXGI_SWAP_CHAIN_DESC1,
                DXGI_SWAP_CHAIN_FLAG, DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,
                DXGI_USAGE_RENDER_TARGET_OUTPUT, IDXGIDevice1, IDXGIFactory2, IDXGISurface,
                IDXGISwapChain1, IDXGISwapChain2,
            },
        },
    },
    core::{BOOL, Interface},
};
use windows_sys::Win32::UI::HiDpi::GetDpiForWindow;
use winio_handle::AsWindow;
use winio_primitive::{DrawingFont, MouseButton, Point, Rect, Size};
pub use winio_ui_windows_common::{Brush, DrawingImage, DrawingPath, DrawingPathBuilder, Pen};
use winui3::{
    ISwapChainPanelNative,
    Microsoft::UI::{Windowing::AppWindow, Xaml::Controls as MUXC},
};

use crate::Widget;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Canvas {
    handle: Widget,
    panel: MUXC::SwapChainPanel,
    app_window: AppWindow,
    dwrite: IDWriteFactory,
    d3d11_device: ID3D11Device,
    d3d11_context: ID3D11DeviceContext,
    d2d1: ID2D1Factory2,
    d2d1_device: ID2D1Device,
    d2d1_context: ID2D1DeviceContext,
    bitmap: Option<ID2D1Bitmap1>,
    swap_chain: IDXGISwapChain1,
}

#[inherit_methods(from = "self.handle")]
impl Canvas {
    pub fn new(parent: impl AsWindow) -> Self {
        let dwrite = unsafe { DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED).unwrap() };
        unsafe {
            let mut device = None;
            let mut context = None;
            D3D11CreateDevice(
                None,
                D3D_DRIVER_TYPE_HARDWARE,
                HMODULE(null_mut()),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                Some(&[
                    D3D_FEATURE_LEVEL_11_1,
                    D3D_FEATURE_LEVEL_11_0,
                    D3D_FEATURE_LEVEL_10_1,
                    D3D_FEATURE_LEVEL_10_0,
                    D3D_FEATURE_LEVEL_9_3,
                    D3D_FEATURE_LEVEL_9_2,
                    D3D_FEATURE_LEVEL_9_1,
                ]),
                D3D11_SDK_VERSION,
                Some(&mut device),
                None,
                Some(&mut context),
            )
            .unwrap();
            let d3d11_device = device.unwrap();
            let dxdi_device = d3d11_device.cast::<IDXGIDevice1>().unwrap();
            let d3d11_context = context.unwrap();
            let d2d1: ID2D1Factory2 =
                D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None).unwrap();
            let d2d1_device: ID2D1Device = d2d1.CreateDevice(&dxdi_device).unwrap().into();
            let d2d1_context = d2d1_device
                .CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)
                .unwrap();
            let panel = MUXC::SwapChainPanel::new().unwrap();
            let native = panel.cast::<ISwapChainPanelNative>().unwrap();
            let desc = DXGI_SWAP_CHAIN_DESC1 {
                Width: 100,
                Height: 100,
                Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                Stereo: BOOL(0),
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                BufferCount: 2,
                Scaling: DXGI_SCALING_STRETCH,
                SwapEffect: DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,
                AlphaMode: DXGI_ALPHA_MODE_PREMULTIPLIED,
                Flags: 0,
            };
            let adapter = dxdi_device.GetAdapter().unwrap();
            let factory = adapter.GetParent::<IDXGIFactory2>().unwrap();
            let swap_chain = factory
                .CreateSwapChainForComposition(&dxdi_device, &desc, None)
                .unwrap();
            dxdi_device.SetMaximumFrameLatency(1).unwrap();
            native.SetSwapChain(&swap_chain).unwrap();
            let app_window = parent.as_window().as_winui().AppWindow().unwrap();
            Self {
                handle: Widget::new(parent, panel.cast().unwrap()),
                panel,
                app_window,
                dwrite,
                d3d11_device,
                d3d11_context,
                d2d1,
                d2d1_device,
                d2d1_context,
                bitmap: None,
                swap_chain,
            }
        }
    }

    fn dpi(&self) -> u32 {
        if let Ok(id) = self.app_window.Id() {
            unsafe { GetDpiForWindow(id.Value as _) }
        } else {
            96
        }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn preferred_size(&self) -> Size;

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);

    pub fn context(&mut self) -> DrawingContext<'_> {
        let context = &self.d2d1_context;
        unsafe {
            context.SetTarget(None);
            self.bitmap = None;
            self.d3d11_context.OMSetRenderTargets(None, None);
            self.d3d11_context.Flush();
            let size = self.size();
            let dpi = self.dpi() as f32;
            let scalex = self.panel.CompositionScaleX().unwrap();
            let scaley = self.panel.CompositionScaleY().unwrap();
            self.swap_chain
                .ResizeBuffers(
                    2,
                    (size.width as f32 * dpi / 96.0 * scalex) as _,
                    (size.height as f32 * dpi / 96.0 * scaley) as _,
                    DXGI_FORMAT_B8G8R8A8_UNORM,
                    DXGI_SWAP_CHAIN_FLAG(0),
                )
                .unwrap();
            let matrix = DXGI_MATRIX_3X2_F {
                _11: 1.0 / scalex / (dpi / 96.0),
                _22: 1.0 / scaley / (dpi / 96.0),
                ..Default::default()
            };
            self.swap_chain
                .cast::<IDXGISwapChain2>()
                .unwrap()
                .SetMatrixTransform(&matrix)
                .unwrap();
            let buffer: IDXGISurface = self.swap_chain.GetBuffer(0).unwrap();
            let props = D2D1_BITMAP_PROPERTIES1 {
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: DXGI_FORMAT_B8G8R8A8_UNORM,
                    alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                },
                dpiX: dpi * scalex,
                dpiY: dpi * scaley,
                bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET | D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
                colorContext: ManuallyDrop::new(None),
            };
            let bitmap = context
                .CreateBitmapFromDxgiSurface(&buffer, Some(&props))
                .unwrap();
            context.SetTarget(&bitmap);
            context.SetDpi(dpi * scalex, dpi * scaley);
            context.SetTextAntialiasMode(D2D1_TEXT_ANTIALIAS_MODE_CLEARTYPE);
            self.bitmap = Some(bitmap);
            context.BeginDraw();
            context.Clear(None);
        }
        DrawingContext::new(&self.dwrite, &self.d2d1, context, &self.swap_chain)
    }

    pub async fn wait_mouse_down(&self) -> MouseButton {
        std::future::pending().await
    }

    pub async fn wait_mouse_up(&self) -> MouseButton {
        std::future::pending().await
    }

    pub async fn wait_mouse_move(&self) -> Point {
        std::future::pending().await
    }
}

pub struct DrawingContext<'a> {
    ctx: winio_ui_windows_common::DrawingContext,
    swap_chain: IDXGISwapChain1,
    _p: PhantomData<&'a mut Canvas>,
}

impl Drop for DrawingContext<'_> {
    fn drop(&mut self) {
        unsafe {
            self.ctx.render_target().EndDraw(None, None).unwrap();
            self.swap_chain.Present(1, DXGI_PRESENT(0)).unwrap();
        }
    }
}

#[inherit_methods(from = "self.ctx")]
impl DrawingContext<'_> {
    fn new(
        dwrite: &IDWriteFactory,
        d2d1: &ID2D1Factory,
        target: &ID2D1RenderTarget,
        swap_chain: &IDXGISwapChain1,
    ) -> Self {
        Self {
            ctx: winio_ui_windows_common::DrawingContext::new(
                d2d1.clone(),
                dwrite.clone(),
                target.clone(),
            ),
            swap_chain: swap_chain.clone(),
            _p: PhantomData,
        }
    }

    pub fn draw_path(&mut self, pen: impl Pen, path: &DrawingPath);

    pub fn fill_path(&mut self, brush: impl Brush, path: &DrawingPath);

    pub fn draw_arc(&mut self, pen: impl Pen, rect: Rect, start: f64, end: f64);

    pub fn draw_pie(&mut self, pen: impl Pen, rect: Rect, start: f64, end: f64);

    pub fn fill_pie(&mut self, brush: impl Brush, rect: Rect, start: f64, end: f64);

    pub fn draw_ellipse(&mut self, pen: impl Pen, rect: Rect);

    pub fn fill_ellipse(&mut self, brush: impl Brush, rect: Rect);

    pub fn draw_line(&mut self, pen: impl Pen, start: Point, end: Point);

    pub fn draw_rect(&mut self, pen: impl Pen, rect: Rect);

    pub fn fill_rect(&mut self, brush: impl Brush, rect: Rect);

    pub fn draw_round_rect(&mut self, pen: impl Pen, rect: Rect, round: Size);

    pub fn fill_round_rect(&mut self, brush: impl Brush, rect: Rect, round: Size);

    pub fn draw_str(&mut self, brush: impl Brush, font: DrawingFont, pos: Point, text: &str);

    pub fn create_image(&self, image: DynamicImage) -> DrawingImage;

    pub fn draw_image(&mut self, image: &DrawingImage, rect: Rect, clip: Option<Rect>);

    pub fn create_path_builder(&self, start: Point) -> DrawingPathBuilder;
}
