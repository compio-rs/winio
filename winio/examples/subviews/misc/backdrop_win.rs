use tuplex::IntoArray;
use winio::prelude::*;

use crate::{Error, Result};

pub struct BackdropChooser {
    view: Child<View>,
    r_none: Child<RadioButton>,
    r_acrylic: Child<RadioButton>,
    r_mica: Child<RadioButton>,
    r_mica_alt: Child<RadioButton>,
}

#[derive(Debug)]
pub enum BackdropChooserEvent {
    ChooseBackdrop(Backdrop),
}

#[derive(Debug)]
pub enum BackdropChooserMessage {
    Noop,
    RSelect(usize),
}

impl Failable for BackdropChooser {
    type Error = Error;
}

impl Component for BackdropChooser {
    type Event = BackdropChooserEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = BackdropChooserMessage;

    fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        init! {
            view: View = (&init),
            r_none: RadioButton = (&view) => {
                text: "Default",
                checked: true,
            },
            r_acrylic: RadioButton = (&view) => {
                text: "Acrylic"
            },
            r_mica: RadioButton = (&view) => {
                text: "Mica"
            },
            r_mica_alt: RadioButton = (&view) => {
                text: "Mica Alt"
            },
        }
        Ok(Self {
            view,
            r_none,
            r_acrylic,
            r_mica,
            r_mica_alt,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        let mut group = RadioButtonGroup::new([
            &mut *self.r_none,
            &mut *self.r_acrylic,
            &mut *self.r_mica,
            &mut *self.r_mica_alt,
        ]);
        start! {
            sender, default: BackdropChooserMessage::Noop,
            group => {
                |i| Some(BackdropChooserMessage::RSelect(i))
            }
        }
    }

    async fn update_children(&mut self) -> Result<bool> {
        Ok(futures_util::try_join!(
            self.view.update(),
            self.r_none.update(),
            self.r_acrylic.update(),
            self.r_mica.update(),
            self.r_mica_alt.update(),
        )?
        .into_array()
        .into_iter()
        .any(|b| b))
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            BackdropChooserMessage::Noop => {}
            BackdropChooserMessage::RSelect(i) => {
                let backdrop = match i {
                    0 => Backdrop::None,
                    1 => Backdrop::Acrylic,
                    2 => Backdrop::Mica,
                    3 => Backdrop::MicaAlt,
                    _ => unreachable!(),
                };
                sender.output(BackdropChooserEvent::ChooseBackdrop(backdrop));
            }
        }
        Ok(false)
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) -> Result<()> {
        let csize = self.view.size()?;

        let mut panel = layout! {
            StackPanel::new(Orient::Vertical),
            self.r_none,
            self.r_acrylic,
            self.r_mica,
            self.r_mica_alt,
        };

        let mut grid = layout! {
            Grid::from_str("1*,auto,1*", "auto").unwrap(),
            panel => { column: 1, row: 0 }
        };

        grid.set_size(csize)?;
        Ok(())
    }

    fn render_children(&mut self) -> Result<()> {
        self.view.render()?;
        self.r_none.render()?;
        self.r_acrylic.render()?;
        self.r_mica.render()?;
        self.r_mica_alt.render()?;
        Ok(())
    }
}

impl Layoutable for BackdropChooser {
    fn loc(&self) -> Result<Point> {
        Ok(self.view.loc()?)
    }

    fn set_loc(&mut self, p: Point) -> Result<()> {
        self.view.set_loc(p)?;
        Ok(())
    }

    fn size(&self) -> Result<Size> {
        Ok(self.view.size()?)
    }

    fn set_size(&mut self, s: Size) -> Result<()> {
        self.view.set_size(s)?;
        Ok(())
    }

    fn preferred_size(&self) -> Result<Size> {
        let mut width = 0.0f64;
        let mut height = 0.0f64;
        for rb in [
            &self.r_none,
            &self.r_acrylic,
            &self.r_mica,
            &self.r_mica_alt,
        ] {
            let ps = rb.preferred_size()?;
            width = width.max(ps.width);
            height += ps.height;
        }
        Ok(Size::new(width, height))
    }

    fn min_size(&self) -> Result<Size> {
        self.preferred_size()
    }
}
