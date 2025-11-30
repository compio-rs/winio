use inherit_methods_macro::inherit_methods;
use winio::prelude::{Error as SysError, Result as SysResult, *};

use crate::{Error, Result};

pub struct BackdropChooser {
    view: Child<View>,
    radios: Child<RadioButtonGroup>,
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

impl Component for BackdropChooser {
    type Error = Error;
    type Event = BackdropChooserEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = BackdropChooserMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
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
            radios: RadioButtonGroup = ([r_none, r_acrylic, r_mica, r_mica_alt]),
        }
        Ok(Self { view, radios })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        start! {
            sender, default: BackdropChooserMessage::Noop,
            self.radios => {
                RadioButtonGroupEvent::Click(i) => BackdropChooserMessage::RSelect(i)
            }
        }
    }

    async fn update_children(&mut self) -> Result<bool> {
        update_children!(self.view, self.radios)
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

        let mut panel = StackPanel::new(Orient::Vertical);
        for r in &mut self.radios[..] {
            panel.push(r).finish();
        }

        let mut grid = layout! {
            Grid::from_str("1*,auto,1*", "auto").unwrap(),
            panel => { column: 1, row: 0 }
        };

        grid.set_size(csize)?;
        Ok(())
    }

    fn render_children(&mut self) -> Result<()> {
        self.view.render()?;
        self.radios.render()?;
        Ok(())
    }
}

impl Failable for BackdropChooser {
    type Error = SysError;
}

#[inherit_methods(from = "self.view")]
impl Layoutable for BackdropChooser {
    fn loc(&self) -> SysResult<Point>;

    fn set_loc(&mut self, p: Point) -> SysResult<()>;

    fn size(&self) -> SysResult<Size>;

    fn set_size(&mut self, s: Size) -> SysResult<()>;

    fn preferred_size(&self) -> SysResult<Size> {
        let mut width = 0.0f64;
        let mut height = 0.0f64;
        for rb in &self.radios[..] {
            let ps = rb.preferred_size()?;
            width = width.max(ps.width);
            height += ps.height;
        }
        Ok(Size::new(width, height))
    }

    fn min_size(&self) -> SysResult<Size> {
        self.preferred_size()
    }
}
