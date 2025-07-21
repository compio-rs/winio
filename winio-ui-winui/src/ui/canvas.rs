use std::{cell::Cell, mem::ManuallyDrop, ptr::null_mut, rc::Rc};

use image::DynamicImage;
use inherit_methods_macro::inherit_methods;
use send_wrapper::SendWrapper;
use windows::{
    Win32::{
        Foundation::{D2DERR_RECREATE_TARGET, HMODULE},
        Graphics::{
            Direct2D::{
                Common::{D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_COLOR_F, D2D1_PIXEL_FORMAT},
                D2D1_BITMAP_OPTIONS_CANNOT_DRAW, D2D1_BITMAP_OPTIONS_TARGET,
                D2D1_BITMAP_PROPERTIES1, D2D1_DEVICE_CONTEXT_OPTIONS_NONE,
                D2D1_TEXT_ANTIALIAS_MODE_CLEARTYPE, ID2D1Bitmap1, ID2D1Device, ID2D1DeviceContext,
                ID2D1Factory2,
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
                DXGI_ERROR_DEVICE_REMOVED, DXGI_ERROR_DEVICE_RESET, DXGI_MATRIX_3X2_F,
                DXGI_PRESENT, DXGI_SCALING_STRETCH, DXGI_SWAP_CHAIN_DESC1, DXGI_SWAP_CHAIN_FLAG,
                DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL, DXGI_USAGE_RENDER_TARGET_OUTPUT, IDXGIDevice1,
                IDXGIFactory2, IDXGISurface, IDXGISwapChain1, IDXGISwapChain2,
            },
        },
    },
    core::{BOOL, Interface, Result},
};
use winio_callback::Callback;
use winio_handle::AsWindow;
use winio_primitive::{DrawingFont, MouseButton, Point, Rect, Size, Vector};
use winio_ui_windows_common::is_dark_mode_allowed_for_app;
pub use winio_ui_windows_common::{Brush, DrawingImage, DrawingPath, DrawingPathBuilder, Pen};
use winui3::{
    ISwapChainPanelNative,
    Microsoft::UI::{
        Input::{PointerDeviceType, PointerPointProperties},
        Xaml::{
            Controls::{self as MUXC, SwapChainPanel},
            Input::{PointerEventHandler, PointerRoutedEventArgs},
        },
    },
};

use crate::{GlobalRuntime, RUNTIME, Widget, ui::Convertible};

#[inline]
fn d2d1<T>(f: impl FnOnce(&ID2D1Factory2) -> T) -> T {
    RUNTIME.with(|runtime| f(runtime.d2d1()))
}

#[inline]
fn is_lost(e: &windows::core::Error) -> bool {
    matches!(
        e.code(),
        D2DERR_RECREATE_TARGET | DXGI_ERROR_DEVICE_REMOVED | DXGI_ERROR_DEVICE_RESET
    )
}

#[derive(Debug)]
#[allow(dead_code)]
struct SwapChain {
    d3d11_device: ID3D11Device,
    d3d11_context: ID3D11DeviceContext,
    d2d1_device: ID2D1Device,
    d2d1_context: ID2D1DeviceContext,
    bitmap: Option<ID2D1Bitmap1>,
    swap_chain: IDXGISwapChain1,
}

impl SwapChain {
    pub fn new() -> Self {
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
            let d2d1_device: ID2D1Device =
                d2d1(|d2d1| d2d1.CreateDevice(&dxdi_device).unwrap().into());
            let d2d1_context = d2d1_device
                .CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)
                .unwrap();
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
            Self {
                d3d11_device,
                d3d11_context,
                d2d1_device,
                d2d1_context,
                bitmap: None,
                swap_chain,
            }
        }
    }

    pub fn set_to_panel(&self, panel: &SwapChainPanel) {
        let native = panel.cast::<ISwapChainPanelNative>().unwrap();
        unsafe {
            native.SetSwapChain(&self.swap_chain).unwrap();
        }
    }

    pub fn begin_draw(&mut self, size: Size, scalex: f32, scaley: f32) -> Result<()> {
        const DPI: f32 = 96.0;

        let context = &self.d2d1_context;
        unsafe {
            context.SetTarget(None);
            self.bitmap = None;
            self.d3d11_context.OMSetRenderTargets(None, None);
            self.d3d11_context.Flush();
            self.swap_chain.ResizeBuffers(
                2,
                (size.width as f32 * scalex).max(1.0) as _,
                (size.height as f32 * scaley).max(1.0) as _,
                DXGI_FORMAT_B8G8R8A8_UNORM,
                DXGI_SWAP_CHAIN_FLAG(0),
            )?;
            let matrix = DXGI_MATRIX_3X2_F {
                _11: 1.0 / scalex,
                _22: 1.0 / scaley,
                ..Default::default()
            };
            self.swap_chain
                .cast::<IDXGISwapChain2>()?
                .SetMatrixTransform(&matrix)?;
            let buffer: IDXGISurface = self.swap_chain.GetBuffer(0)?;
            let props = D2D1_BITMAP_PROPERTIES1 {
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: DXGI_FORMAT_B8G8R8A8_UNORM,
                    alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                },
                dpiX: DPI * scalex,
                dpiY: DPI * scaley,
                bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET | D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
                colorContext: ManuallyDrop::new(None),
            };
            let bitmap = context.CreateBitmapFromDxgiSurface(&buffer, Some(&props))?;
            context.SetTarget(&bitmap);
            context.SetDpi(DPI * scalex, DPI * scaley);
            context.SetTextAntialiasMode(D2D1_TEXT_ANTIALIAS_MODE_CLEARTYPE);
            self.bitmap = Some(bitmap);
            context.BeginDraw();
            context.Clear(Some(&if is_dark_mode_allowed_for_app() {
                D2D1_COLOR_F {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                }
            } else {
                D2D1_COLOR_F {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                }
            }));
        }
        Ok(())
    }

    pub fn end_draw(&mut self) -> Result<()> {
        unsafe {
            self.d2d1_context.EndDraw(None, None)?;
            self.swap_chain.Present(1, DXGI_PRESENT(0)).ok()?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Canvas {
    on_press: SendWrapper<Rc<Callback<MouseButton>>>,
    on_release: SendWrapper<Rc<Callback<MouseButton>>>,
    on_move: SendWrapper<Rc<Callback<Point>>>,
    on_wheel: SendWrapper<Rc<Callback<Vector>>>,
    handle: Widget,
    panel: MUXC::SwapChainPanel,
    dwrite: IDWriteFactory,
    swap_chain: SwapChain,
}

#[inherit_methods(from = "self.handle")]
impl Canvas {
    pub fn new(parent: impl AsWindow) -> Self {
        let dwrite = unsafe { DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED).unwrap() };
        let panel = MUXC::SwapChainPanel::new().unwrap();
        let swap_chain = SwapChain::new();
        swap_chain.set_to_panel(&panel);

        let mouse_button_cache = SendWrapper::new(Rc::new(Cell::new(MouseButton::Other)));
        let on_press = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_press = on_press.clone();
            let mouse_button_cache = mouse_button_cache.clone();
            panel
                .PointerPressed(&PointerEventHandler::new(move |sender, args| {
                    if let Some(args) = args.as_ref() {
                        if let Some(panel) = sender
                            .as_ref()
                            .and_then(|sender| sender.cast::<SwapChainPanel>().ok())
                        {
                            let mouse = mouse_button(&panel, args)?;
                            mouse_button_cache.set(mouse);
                            on_press.signal::<GlobalRuntime>(mouse);
                        }
                    }
                    Ok(())
                }))
                .unwrap();
        }
        let on_release = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_release = on_release.clone();
            let mouse_button_cache = mouse_button_cache.clone();
            panel
                .PointerReleased(&PointerEventHandler::new(move |_, _| {
                    let mouse = mouse_button_cache.get();
                    on_release.signal::<GlobalRuntime>(mouse);
                    mouse_button_cache.set(MouseButton::Other);
                    Ok(())
                }))
                .unwrap();
        }
        let on_move = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_move = on_move.clone();
            panel
                .PointerMoved(&PointerEventHandler::new(move |sender, args| {
                    if let Some(args) = args.as_ref() {
                        if let Some(panel) = sender
                            .as_ref()
                            .and_then(|sender| sender.cast::<SwapChainPanel>().ok())
                        {
                            let point = args.GetCurrentPoint(&panel)?;
                            on_move.signal::<GlobalRuntime>(Point::from_native(point.Position()?));
                        }
                    }
                    Ok(())
                }))
                .unwrap();
        }
        let on_wheel = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_wheel = on_wheel.clone();
            panel
                .PointerWheelChanged(&PointerEventHandler::new(move |sender, args| {
                    if let Some(args) = args.as_ref() {
                        if let Some(panel) = sender
                            .as_ref()
                            .and_then(|sender| sender.cast::<SwapChainPanel>().ok())
                        {
                            let point = args.GetCurrentPoint(&panel)?;
                            let props = point.Properties()?;
                            let delta = props.MouseWheelDelta()?;
                            let orient = props.Orientation()? / 180.0 * std::f32::consts::PI;
                            let deltay = orient.cos() as f64 * delta as f64;
                            let deltax = -orient.sin() as f64 * delta as f64;
                            on_wheel.signal::<GlobalRuntime>(Vector::new(deltax, deltay));
                        }
                    }
                    Ok(())
                }))
                .unwrap();
        }

        Self {
            on_press,
            on_release,
            on_move,
            on_wheel,
            handle: Widget::new(parent, panel.cast().unwrap()),
            panel,
            dwrite,
            swap_chain,
        }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn preferred_size(&self) -> Size {
        Size::zero()
    }

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);

    pub fn context(&mut self) -> DrawingContext<'_> {
        let size = self.size();
        let scalex = self.panel.CompositionScaleX().unwrap();
        let scaley = self.panel.CompositionScaleY().unwrap();
        loop {
            match self.swap_chain.begin_draw(size, scalex, scaley) {
                Ok(()) => break,
                Err(e) if is_lost(&e) => self.handle_lost(),
                Err(e) => panic!("{e:}"),
            }
        }
        DrawingContext::new(self)
    }

    fn handle_lost(&mut self) {
        self.swap_chain = SwapChain::new();
        self.swap_chain.set_to_panel(&self.panel);
    }

    pub async fn wait_mouse_down(&self) -> MouseButton {
        self.on_press.wait().await
    }

    pub async fn wait_mouse_up(&self) -> MouseButton {
        self.on_release.wait().await
    }

    pub async fn wait_mouse_move(&self) -> Point {
        self.on_move.wait().await
    }

    pub async fn wait_mouse_wheel(&self) -> Vector {
        self.on_wheel.wait().await
    }
}

winio_handle::impl_as_widget!(Canvas, handle);

fn mouse_button(panel: &SwapChainPanel, args: &PointerRoutedEventArgs) -> Result<MouseButton> {
    let pointer = args.Pointer()?;
    if pointer.PointerDeviceType() == Ok(PointerDeviceType::Mouse) {
        let pt = args.GetCurrentPoint(panel)?;
        let props = pt.Properties()?;
        mouse_button_from_point(&props)
    } else {
        Ok(MouseButton::Other)
    }
}

fn mouse_button_from_point(props: &PointerPointProperties) -> Result<MouseButton> {
    let res = if props.IsLeftButtonPressed()? {
        MouseButton::Left
    } else if props.IsRightButtonPressed()? {
        MouseButton::Right
    } else if props.IsMiddleButtonPressed()? {
        MouseButton::Middle
    } else {
        MouseButton::Other
    };
    Ok(res)
}

pub struct DrawingContext<'a> {
    ctx: winio_ui_windows_common::DrawingContext,
    canvas: &'a mut Canvas,
}

impl Drop for DrawingContext<'_> {
    fn drop(&mut self) {
        match self.canvas.swap_chain.end_draw() {
            Ok(()) => {}
            Err(e) if is_lost(&e) => self.canvas.handle_lost(),
            Err(e) => panic!("{e:?}"),
        }
    }
}

impl<'a> DrawingContext<'a> {
    fn new(canvas: &'a mut Canvas) -> Self {
        Self {
            ctx: winio_ui_windows_common::DrawingContext::new(
                d2d1(|d2d1| d2d1.clone().into()),
                canvas.dwrite.clone(),
                canvas.swap_chain.d2d1_context.clone().into(),
            ),
            canvas,
        }
    }
}

#[inherit_methods(from = "self.ctx")]
impl DrawingContext<'_> {
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
