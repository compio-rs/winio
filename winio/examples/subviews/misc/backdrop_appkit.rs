use inherit_methods_macro::inherit_methods;
use winio::prelude::*;

pub struct BackdropChooser {
    combo: Child<ComboBox>,
}

#[derive(Debug)]
pub enum BackdropChooserEvent {
    ChooseVibrancy(Option<Vibrancy>),
}

#[derive(Debug)]
pub enum BackdropChooserMessage {
    Noop,
    Select,
}

impl Component for BackdropChooser {
    type Event = BackdropChooserEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = BackdropChooserMessage;

    fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Self {
        init! {
            combo: ComboBox = (&init) => {
                items: [
                    "None",
                    "Appearance Based",
                    "Light",
                    "Dark",
                    "Titlebar",
                    "Selection",
                    "Menu",
                    "Popover",
                    "Sidebar",
                    "Medium Light",
                    "Ultra Dark",
                    "Header View",
                    "Sheet",
                    "Window Background",
                    "HUD Window",
                    "Full Screen UI",
                    "Tooltip",
                    "Content Background",
                    "Under Window Background",
                    "Under Page Background",
                ],
            }
        }
        Self { combo }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        start! {
            sender, default: BackdropChooserMessage::Noop,
            self.combo => {
                ComboBoxEvent::Select => BackdropChooserMessage::Select,
            }
        }
    }

    #[allow(deprecated)]
    async fn update(&mut self, message: Self::Message, sender: &ComponentSender<Self>) -> bool {
        match message {
            BackdropChooserMessage::Noop => false,
            BackdropChooserMessage::Select => {
                let vibrancy = match self.combo.selection() {
                    Some(0) => None,
                    Some(index) => Some(match index {
                        1 => Vibrancy::AppearanceBased,
                        2 => Vibrancy::Light,
                        3 => Vibrancy::Dark,
                        4 => Vibrancy::Titlebar,
                        5 => Vibrancy::Selection,
                        6 => Vibrancy::Menu,
                        7 => Vibrancy::Popover,
                        8 => Vibrancy::Sidebar,
                        9 => Vibrancy::MediumLight,
                        10 => Vibrancy::UltraDark,
                        11 => Vibrancy::HeaderView,
                        12 => Vibrancy::Sheet,
                        13 => Vibrancy::WindowBackground,
                        14 => Vibrancy::HudWindow,
                        15 => Vibrancy::FullScreenUI,
                        16 => Vibrancy::Tooltip,
                        17 => Vibrancy::ContentBackground,
                        18 => Vibrancy::UnderWindowBackground,
                        19 => Vibrancy::UnderPageBackground,
                        _ => unreachable!(),
                    }),
                    _ => None,
                };
                sender.output(BackdropChooserEvent::ChooseVibrancy(vibrancy));
                true
            }
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {}
}

#[inherit_methods(from = "self.combo")]
impl Layoutable for BackdropChooser {
    fn loc(&self) -> Point;

    fn set_loc(&mut self, _p: Point);

    fn size(&self) -> Size;

    fn set_size(&mut self, _s: Size);

    fn preferred_size(&self) -> Size;

    fn min_size(&self) -> Size;
}
