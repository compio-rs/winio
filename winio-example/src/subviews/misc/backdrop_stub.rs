use winio::prelude::{Error as SysError, Result as SysResult, *};

use crate::{Error, Result};

pub struct BackdropChooser {}

#[derive(Debug)]
pub enum BackdropChooserEvent {}

#[derive(Debug)]
pub enum BackdropChooserMessage {}

impl Component for BackdropChooser {
    type Error = Error;
    type Event = BackdropChooserEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = BackdropChooserMessage;

    async fn init(_init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        Ok(Self {})
    }
}

impl Failable for BackdropChooser {
    type Error = SysError;
}

impl Layoutable for BackdropChooser {
    fn loc(&self) -> SysResult<Point> {
        Ok(Point::zero())
    }

    fn set_loc(&mut self, _p: Point) -> SysResult<()> {
        Ok(())
    }

    fn size(&self) -> SysResult<Size> {
        Ok(Size::zero())
    }

    fn set_size(&mut self, _s: Size) -> SysResult<()> {
        Ok(())
    }
}
