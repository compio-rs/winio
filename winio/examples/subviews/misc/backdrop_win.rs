use inherit_methods_macro::inherit_methods;
use tuplex::IntoArray;
use winio::prelude::*;

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

impl Component for BackdropChooser {
    type Event = BackdropChooserEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = BackdropChooserMessage;

    fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Self {
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
        Self {
            view,
            r_none,
            r_acrylic,
            r_mica,
            r_mica_alt,
        }
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

    async fn update_children(&mut self) -> bool {
        futures_util::join!(
            self.view.update(),
            self.r_none.update(),
            self.r_acrylic.update(),
            self.r_mica.update(),
            self.r_mica_alt.update(),
        )
        .into_array()
        .into_iter()
        .any(|b| b)
    }

    async fn update(&mut self, message: Self::Message, sender: &ComponentSender<Self>) -> bool {
        match message {
            BackdropChooserMessage::Noop => false,
            BackdropChooserMessage::RSelect(i) => {
                let backdrop = match i {
                    0 => Backdrop::None,
                    1 => Backdrop::Acrylic,
                    2 => Backdrop::Mica,
                    3 => Backdrop::MicaAlt,
                    _ => unreachable!(),
                };
                sender.output(BackdropChooserEvent::ChooseBackdrop(backdrop));
                false
            }
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {
        let csize = self.view.size();

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

        grid.set_size(csize);
    }

    fn render_children(&mut self) {
        self.view.render();
        self.r_none.render();
        self.r_acrylic.render();
        self.r_mica.render();
        self.r_mica_alt.render();
    }
}

#[inherit_methods(from = "self.view")]
impl Layoutable for BackdropChooser {
    fn loc(&self) -> Point;

    fn set_loc(&mut self, p: Point);

    fn size(&self) -> Size;

    fn set_size(&mut self, s: Size);

    fn preferred_size(&self) -> Size {
        let mut width = 0.0f64;
        let mut height = 0.0f64;
        for rb in [
            &self.r_none,
            &self.r_acrylic,
            &self.r_mica,
            &self.r_mica_alt,
        ] {
            let ps = rb.preferred_size();
            width = width.max(ps.width);
            height += ps.height;
        }
        Size::new(width, height)
    }

    fn min_size(&self) -> Size {
        self.preferred_size()
    }
}
