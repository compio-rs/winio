use std::{cell::Cell, mem::ManuallyDrop, ops::Deref, rc::Rc};

use inherit_methods_macro::inherit_methods;
use send_wrapper::SendWrapper;
use windows::Win32::{
    Foundation::{D2DERR_RECREATE_TARGET, E_POINTER},
    Graphics::{
        Direct2D::{
            Common::{D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_COLOR_F, D2D1_PIXEL_FORMAT},
            D2D1_BITMAP_OPTIONS_CANNOT_DRAW, D2D1_BITMAP_OPTIONS_TARGET, D2D1_BITMAP_PROPERTIES1,
            D2D1_DEVICE_CONTEXT_OPTIONS_NONE, ID2D1Bitmap1, ID2D1Device, ID2D1DeviceContext,
            ID2D1RenderTarget,
        },
        Direct3D::{
            D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL_9_1, D3D_FEATURE_LEVEL_9_2,
            D3D_FEATURE_LEVEL_9_3, D3D_FEATURE_LEVEL_10_0, D3D_FEATURE_LEVEL_10_1,
            D3D_FEATURE_LEVEL_11_0, D3D_FEATURE_LEVEL_11_1,
        },
        Direct3D11::{
            D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_SDK_VERSION, D3D11CreateDevice, ID3D11Device,
            ID3D11DeviceContext,
        },
        DirectWrite::IDWriteFactory,
        Dxgi::{
            Common::{DXGI_ALPHA_MODE_PREMULTIPLIED, DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC},
            DXGI_ERROR_DEVICE_REMOVED, DXGI_ERROR_DEVICE_RESET, DXGI_MATRIX_3X2_F,
            DXGI_SCALING_STRETCH, DXGI_SWAP_CHAIN_DESC1, DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,
            DXGI_USAGE_RENDER_TARGET_OUTPUT, IDXGIDevice1, IDXGIFactory2, IDXGISurface,
            IDXGISwapChain1, IDXGISwapChain2,
        },
    },
};
use windows_core::{BOOL, Interface};
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{ColorTheme, KeyCode, MouseButton, Point, Size, Vector};
pub use winio_ui_windows_common::{
    Brush, DrawingContext, DrawingImage, DrawingPath, DrawingPathBuilder, Pen,
};
use winio_ui_windows_common::{ContextOwner, d2d1_factory, dwrite_factory};
use winui3::Microsoft::UI::{
    Input::{PointerDeviceType, PointerPointProperties},
    Xaml::{Controls as MUXC, Input as MUXI, Media::DxInterop::ISwapChainPanelNative},
};

use crate::{
    Error, GlobalRuntime, Result, Widget, color_theme, get_root_window,
    platform::keyboard::Keyboard, widgets::Convertible,
};

#[derive(Debug)]
pub(crate) struct CanvasImpl {
    on_press: SendWrapper<Rc<Callback<MouseButton>>>,
    on_release: SendWrapper<Rc<Callback<MouseButton>>>,
    on_move: SendWrapper<Rc<Callback<Point>>>,
    on_wheel: SendWrapper<Rc<Callback<Vector>>>,
    keyboard: Keyboard,
    handle: Widget,
    panel: MUXC::SwapChainPanel,
}

#[inherit_methods(from = "self.handle")]
impl CanvasImpl {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let panel = MUXC::SwapChainPanel::new()?;
        let control = MUXC::UserControl::new()?;
        control.SetContent(&panel)?;
        let keyboard = Keyboard::new(&control.cast()?)?;

        let mouse_button_cache = SendWrapper::new(Rc::new(Cell::new(MouseButton::Other)));
        let on_press = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_press = on_press.clone();
            let mouse_button_cache = mouse_button_cache.clone();
            panel
                .PointerPressed(move |sender, args| {
                    let panel = sender.ok()?.cast::<MUXC::SwapChainPanel>()?;
                    let args = args.ok()?;
                    let mouse = mouse_button(&panel, args)?;
                    mouse_button_cache.set(mouse);
                    on_press.signal::<GlobalRuntime>(mouse);
                    Ok(())
                })?
                .forget();
        }
        let on_release = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_release = on_release.clone();
            let mouse_button_cache = mouse_button_cache.clone();
            panel
                .PointerReleased(move |_, _| {
                    let mouse = mouse_button_cache.get();
                    on_release.signal::<GlobalRuntime>(mouse);
                    mouse_button_cache.set(MouseButton::Other);
                    Ok(())
                })?
                .forget();
        }
        let on_move = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_move = on_move.clone();
            panel
                .PointerMoved(move |sender, args| {
                    let panel = sender.ok()?.cast::<MUXC::SwapChainPanel>()?;
                    let args = args.ok()?;
                    let point = args.GetCurrentPoint(&panel)?;
                    on_move.signal::<GlobalRuntime>(Point::from_native(point.Position()?));
                    Ok(())
                })?
                .forget();
        }
        let on_wheel = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_wheel = on_wheel.clone();
            panel
                .PointerWheelChanged(move |sender, args| {
                    let panel = sender.ok()?.cast::<MUXC::SwapChainPanel>()?;
                    let args = args.ok()?;
                    let point = args.GetCurrentPoint(&panel)?;
                    let props = point.Properties()?;
                    let delta = props.MouseWheelDelta()?;
                    let orient = props.Orientation()? / 180.0 * std::f32::consts::PI;
                    let deltay = orient.cos() as f64 * delta as f64;
                    let deltax = -orient.sin() as f64 * delta as f64;
                    on_wheel.signal::<GlobalRuntime>(Vector::new(deltax, deltay));
                    Ok(())
                })?
                .forget();
        }

        Ok(Self {
            on_press,
            on_release,
            on_move,
            on_wheel,
            keyboard,
            handle: Widget::new(parent, control.cast()?)?,
            panel,
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

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

    pub async fn wait_key_down(&self) -> KeyCode {
        self.keyboard.wait_key_down().await
    }

    pub async fn wait_key_up(&self) -> KeyCode {
        self.keyboard.wait_key_up().await
    }

    pub async fn wait_key_char(&self) -> char {
        self.keyboard.wait_key_char().await
    }
}

impl Deref for CanvasImpl {
    type Target = MUXC::SwapChainPanel;

    fn deref(&self) -> &Self::Target {
        &self.panel
    }
}

winio_handle::impl_as_widget!(CanvasImpl, handle);

fn mouse_button(
    panel: &MUXC::SwapChainPanel,
    args: &MUXI::PointerRoutedEventArgs,
) -> Result<MouseButton> {
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

#[inline]
fn is_lost(e: &Error) -> bool {
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
    pub fn new() -> Result<Self> {
        unsafe {
            let mut device = None;
            let mut context = None;
            D3D11CreateDevice(
                None,
                D3D_DRIVER_TYPE_HARDWARE,
                None,
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
            )?;
            let d3d11_device = device.ok_or(Error::from_hresult(E_POINTER))?;
            let dxdi_device = d3d11_device.cast::<IDXGIDevice1>()?;
            let d3d11_context = context.ok_or(Error::from_hresult(E_POINTER))?;
            let d2d1_device: ID2D1Device = d2d1_factory()?.CreateDevice(&dxdi_device)?.into();
            let d2d1_context = d2d1_device.CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)?;
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
            let adapter = dxdi_device.GetAdapter()?;
            let factory = adapter.GetParent::<IDXGIFactory2>()?;
            let swap_chain = factory.CreateSwapChainForComposition(&dxdi_device, &desc, None)?;
            dxdi_device.SetMaximumFrameLatency(1)?;
            Ok(Self {
                d3d11_device,
                d3d11_context,
                d2d1_device,
                d2d1_context,
                bitmap: None,
                swap_chain,
            })
        }
    }

    pub fn set_to_panel(&self, panel: &MUXC::SwapChainPanel) -> Result<()> {
        let native = panel.cast::<ISwapChainPanelNative>()?;
        unsafe {
            native.SetSwapChain(&self.swap_chain)?;
        }
        Ok(())
    }

    pub fn begin_draw(
        &mut self,
        panel: &MUXC::SwapChainPanel,
        size: Size,
        scalex: f32,
        scaley: f32,
    ) -> Result<()> {
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
                0,
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
            self.bitmap = Some(bitmap);
            context.BeginDraw();
            let has_backdrop = get_root_window(&panel.cast()?)
                .map(|w| w.SystemBackdrop().is_ok())
                .unwrap_or_default();
            let clear_color = if has_backdrop {
                None
            } else if matches!(color_theme()?, ColorTheme::Dark) {
                Some(D2D1_COLOR_F {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                })
            } else {
                Some(D2D1_COLOR_F {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                })
            };
            context.Clear(clear_color.as_ref().map(|c| c as *const _));
        }
        Ok(())
    }

    pub fn end_draw(&mut self) -> Result<()> {
        unsafe {
            self.d2d1_context.EndDraw(None, None).ok()?;
            self.swap_chain.Present(1, 0).ok()?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Canvas {
    handle: CanvasImpl,
    dwrite: IDWriteFactory,
    swap_chain: SwapChain,
}

#[inherit_methods(from = "self.handle")]
impl Canvas {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let handle = CanvasImpl::new(parent)?;
        let dwrite = dwrite_factory()?.clone();
        let swap_chain = SwapChain::new()?;
        swap_chain.set_to_panel(&handle)?;

        Ok(Self {
            handle,
            dwrite,
            swap_chain,
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn context(&mut self) -> Result<DrawingContext<'_>> {
        let size = self.size()?;
        let scalex = self.handle.CompositionScaleX()?;
        let scaley = self.handle.CompositionScaleY()?;
        loop {
            match self
                .swap_chain
                .begin_draw(&self.handle, size, scalex, scaley)
            {
                Ok(()) => break,
                Err(e) if is_lost(&e) => self.handle_lost()?,
                Err(e) => return Err(e),
            }
        }
        Ok(DrawingContext::new(
            d2d1_factory()?.clone().into(),
            self.dwrite.clone(),
            self.swap_chain.d2d1_context.clone().into(),
            Some(self),
        ))
    }

    fn handle_lost(&mut self) -> Result<()> {
        self.swap_chain = SwapChain::new()?;
        self.swap_chain.set_to_panel(&self.handle)?;
        Ok(())
    }

    pub async fn wait_mouse_down(&self) -> MouseButton {
        self.handle.wait_mouse_down().await
    }

    pub async fn wait_mouse_up(&self) -> MouseButton {
        self.handle.wait_mouse_up().await
    }

    pub async fn wait_mouse_move(&self) -> Point {
        self.handle.wait_mouse_move().await
    }

    pub async fn wait_mouse_wheel(&self) -> Vector {
        self.handle.wait_mouse_wheel().await
    }

    pub async fn wait_key_down(&self) -> KeyCode {
        self.handle.wait_key_down().await
    }

    pub async fn wait_key_up(&self) -> KeyCode {
        self.handle.wait_key_up().await
    }

    pub async fn wait_key_char(&self) -> char {
        self.handle.wait_key_char().await
    }
}

winio_handle::impl_as_widget!(Canvas, handle);

impl ContextOwner for Canvas {
    fn end_draw(&mut self, _target: &ID2D1RenderTarget) -> Result<()> {
        match self.swap_chain.end_draw() {
            Ok(()) => Ok(()),
            Err(e) if is_lost(&e) => self.handle_lost(),
            Err(e) => Err(e),
        }
    }
}
