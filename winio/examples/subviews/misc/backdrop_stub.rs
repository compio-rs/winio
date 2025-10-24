use winio::prelude::*;

pub struct BackdropChooser {}

#[derive(Debug)]
pub enum BackdropChooserEvent {}

#[derive(Debug)]
pub enum BackdropChooserMessage {}

impl Component for BackdropChooser {
    type Event = BackdropChooserEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = BackdropChooserMessage;

    fn init(_init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Self {
        Self {}
    }

    async fn start(&mut self, _sender: &ComponentSender<Self>) -> ! {
        std::future::pending().await
    }

    async fn update(&mut self, _message: Self::Message, _sender: &ComponentSender<Self>) -> bool {
        false
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {}
}

impl Layoutable for BackdropChooser {
    fn loc(&self) -> Point {
        Point::zero()
    }

    fn set_loc(&mut self, _p: Point) {}

    fn size(&self) -> Size {
        Size::zero()
    }

    fn set_size(&mut self, _s: Size) {}
}
