use std::{cell::RefCell, marker::PhantomData, rc::Rc};

use image::DynamicImage;
use inherit_methods_macro::inherit_methods;
use send_wrapper::SendWrapper;
use windows::{
    Win32::Graphics::{
        Direct2D::ID2D1RenderTarget,
        DirectWrite::{DWRITE_FACTORY_TYPE_SHARED, DWriteCreateFactory, IDWriteFactory},
        Dxgi::{
            Common::{DXGI_ALPHA_MODE_PREMULTIPLIED, DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC},
            DXGI_PRESENT, DXGI_SCALING_STRETCH, DXGI_SWAP_CHAIN_DESC1,
            DXGI_SWAP_EFFECT_FLIP_DISCARD, DXGI_USAGE_RENDER_TARGET_OUTPUT, IDXGIDevice,
            IDXGIFactory2, IDXGISwapChain1,
        },
    },
    core::{BOOL, Interface},
};
use winio_handle::AsWindow;
use winio_primitive::{DrawingFont, MouseButton, Point, Rect, Size};
pub use winio_ui_windows_common::{Brush, DrawingImage, DrawingPath, DrawingPathBuilder, Pen};
use winui3::{
    ISwapChainPanelNative,
    Microsoft::UI::Xaml::{Controls as MUXC, RoutedEventHandler},
};

use crate::{RUNTIME, Widget};

#[derive(Debug)]
pub struct Canvas {
    dwrite: IDWriteFactory,
    handle: Widget,
    panel: MUXC::SwapChainPanel,
    swap_chain: SendWrapper<Rc<RefCell<Option<IDXGISwapChain1>>>>,
}

#[inherit_methods(from = "self.handle")]
impl Canvas {
    pub fn new(parent: impl AsWindow) -> Self {
        let dwrite = unsafe { DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED).unwrap() };
        let panel = MUXC::SwapChainPanel::new().unwrap();
        let native = SendWrapper::new(panel.cast::<ISwapChainPanelNative>().unwrap());
        let chain = SendWrapper::new(Rc::new(RefCell::new(None)));
        panel
            .Loaded(&RoutedEventHandler::new({
                let chain = chain.clone();
                move |_, _| {
                    let desc = DXGI_SWAP_CHAIN_DESC1 {
                        Width: 0,
                        Height: 0,
                        Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                        Stereo: BOOL(0),
                        SampleDesc: DXGI_SAMPLE_DESC {
                            Count: 1,
                            Quality: 0,
                        },
                        BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                        BufferCount: 2,
                        Scaling: DXGI_SCALING_STRETCH,
                        SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
                        AlphaMode: DXGI_ALPHA_MODE_PREMULTIPLIED,
                        Flags: 0,
                    };
                    unsafe {
                        let device =
                            RUNTIME.with(|runtime| runtime.d3d11_device().cast::<IDXGIDevice>())?;
                        let adapter = device.GetAdapter()?;
                        let factory = adapter.GetParent::<IDXGIFactory2>()?;
                        let swap_chain =
                            factory.CreateSwapChainForComposition(&device, &desc, None)?;
                        native.SetSwapChain(&swap_chain)?;
                        *chain.borrow_mut() = Some(swap_chain);
                    }
                    Ok(())
                }
            }))
            .unwrap();
        Self {
            dwrite,
            handle: Widget::new(parent, panel.cast().unwrap()),
            panel,
            swap_chain: chain,
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
        let context = RUNTIME.with(|runtime| runtime.d2d1_context().clone());
        unsafe {
            context.BeginDraw();
            context.Clear(None);
        }
        DrawingContext::new(&self.dwrite, &context, self.swap_chain.clone())
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
    swap_chain: SendWrapper<Rc<RefCell<Option<IDXGISwapChain1>>>>,
    _p: PhantomData<&'a mut Canvas>,
}

impl Drop for DrawingContext<'_> {
    fn drop(&mut self) {
        if let Some(chain) = self.swap_chain.borrow().as_ref() {
            unsafe {
                chain.Present(1, DXGI_PRESENT(0)).unwrap();
            }
        }
    }
}

#[inherit_methods(from = "self.ctx")]
impl DrawingContext<'_> {
    fn new(
        dwrite: &IDWriteFactory,
        target: &ID2D1RenderTarget,
        swap_chain: SendWrapper<Rc<RefCell<Option<IDXGISwapChain1>>>>,
    ) -> Self {
        RUNTIME.with(|runtime| Self {
            ctx: winio_ui_windows_common::DrawingContext::new(
                runtime.d2d1().clone(),
                dwrite.clone(),
                target.clone(),
            ),
            swap_chain,
            _p: PhantomData,
        })
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
