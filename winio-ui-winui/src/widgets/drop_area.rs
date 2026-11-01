use std::{fmt::Debug, rc::Rc};

use inherit_methods_macro::inherit_methods;
use send_wrapper::SendWrapper;
use windows::{
    ApplicationModel::DataTransfer::{DataPackageOperation, DataPackageView},
    UI::Color,
};
use windows_core::Interface;
use winio_callback::Callback;
use winio_handle::{AsContainer, BorrowedContainer};
use winio_primitive::{Point, Size};
use winui3::Microsoft::UI::Xaml::{self as MUX, Controls as MUXC, Media as MUXM};

use crate::{Data, GlobalRuntime, Result, Widget};

pub struct DropArea {
    on_drop: SendWrapper<Rc<Callback<DataPackageView>>>,
    handle: Widget,
    canvas: MUXC::Canvas,
}

#[inherit_methods(from = "self.handle")]
impl DropArea {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let canvas = MUXC::Canvas::new()?;
        let panel = canvas.cast::<MUXC::Panel>()?;
        let brush = MUXM::SolidColorBrush::new()?;
        brush.SetColor(Color {
            A: 0,
            R: 0,
            G: 0,
            B: 0,
        })?;
        panel.SetBackground(&brush)?;

        let element = canvas.cast::<MUX::FrameworkElement>()?;
        element.SetAllowDrop(true)?;

        let on_drop = SendWrapper::new(Rc::new(Callback::new()));

        element
            .DragOver(move |_, args| {
                let args = args.ok()?;
                args.SetAcceptedOperation(DataPackageOperation::Copy)?;
                args.SetHandled(true)?;
                Ok(())
            })?
            .forget();
        element
            .DragEnter(move |_, args| {
                let args = args.ok()?;
                args.SetAcceptedOperation(DataPackageOperation::Copy)?;
                args.SetHandled(true)?;
                Ok(())
            })?
            .forget();
        element
            .Drop({
                let on_drop = on_drop.clone();
                move |_, args| {
                    let args = args.ok()?;
                    args.SetAcceptedOperation(DataPackageOperation::Copy)?;
                    args.SetHandled(true)?;
                    let view = args.DataView()?;
                    on_drop.signal::<GlobalRuntime>(view);
                    Ok(())
                }
            })?
            .forget();

        Ok(Self {
            handle: Widget::new(parent, canvas.cast()?)?,
            canvas,
            on_drop,
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, visible: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, point: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, size: Size) -> Result<()>;

    pub async fn wait_drop(&self) -> Data {
        loop {
            let view = self.on_drop.wait().await;
            if let Ok(Some(data)) = Data::from_data_package_view(&view).await {
                return data;
            }
        }
    }
}

impl Debug for DropArea {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DropArea").finish_non_exhaustive()
    }
}

winio_handle::impl_as_widget!(DropArea, handle);

impl AsContainer for DropArea {
    fn as_container(&self) -> BorrowedContainer<'_> {
        BorrowedContainer::winui(&self.canvas)
    }
}
