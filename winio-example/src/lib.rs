use std::convert::Infallible;

use winio::prelude::*;

#[cfg(target_os = "android")]
mod android;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An error from the UI backend.
    #[error("UI error: {0}")]
    Ui(#[from] winio::Error),
    /// An error from [`winio_layout`].
    #[error("Layout error: {0}")]
    Layout(#[from] TaffyError),
    /// An IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl<E: Into<Error> + std::fmt::Display> From<LayoutError<E>> for Error {
    fn from(e: LayoutError<E>) -> Self {
        match e {
            LayoutError::Taffy(te) => Error::Layout(te),
            LayoutError::Child(ce) => ce.into(),
            _ => Error::Io(std::io::Error::other(e.to_string())),
        }
    }
}

impl From<Infallible> for Error {
    fn from(e: Infallible) -> Self {
        match e {}
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct MainModel {
    window: Child<Window>,
    link: Child<LinkLabel>,
    ulabel: Child<Label>,
    plabel: Child<Label>,
    uentry: Child<Edit>,
    pentry: Child<Edit>,
    pcheck: Child<CheckBox>,
    canvas: Child<Canvas>,
    combo: Child<ComboBox>,
    list: Child<ObservableVec<String>>,
    radios: Child<RadioButtonGroup>,
    rindex: usize,
    push_button: Child<Button>,
    pop_button: Child<Button>,
    show_button: Child<Button>,
    progress: Child<Progress>,
    mltext: Child<TextBox>,
}

#[derive(Debug)]
pub enum MainMessage {
    Noop,
    Close,
    Redraw,
    List(ObservableVecEvent<String>),
    Select,
    Push,
    Pop,
    Show,
    RSelect(usize),
    PasswordCheck,
}

impl Component for MainModel {
    type Error = Error;
    type Event = ();
    type Init<'a> = ();
    type Message = MainMessage;

    async fn init(_init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        init! {
            window: Window = (()) => {
                text: "Hello example",
            },
            canvas: Canvas = (&window),
            link: LinkLabel = (&window) => {
                text: "Source",
                uri: "https://github.com/compio-rs/winio",
            },
            ulabel: Label = (&window) => {
                text: "Username:",
                tooltip: "Your username",
                halign: HAlign::Right,
            },
            plabel: Label = (&window) => {
                text: "Password:",
                halign: HAlign::Right,
            },
            uentry: Edit = (&window) => {
                text: "AAA",
            },
            pentry: Edit = (&window) => {
                text: "123456",
                password: true,
            },
            pcheck: CheckBox = (&window) => {
                text: "Show",
                checked: false,
            },
            combo: ComboBox = (&window),
            list: ObservableVec<String> = ([]) => {
                // https://www.zhihu.com/question/23600507/answer/140640887
                items: [
                    "烫烫烫",
                    "昍昍昍",
                    "ﾌﾌﾌﾌﾌﾌ",
                    "쳌쳌쳌"
                ],
            },
            r1: RadioButton = (&window) => {
                text: "屯屯屯",
                checked: true,
            },
            r2: RadioButton = (&window) => {
                text: "锟斤拷",
            },
            r3: RadioButton = (&window) => {
                text: "╠╠╠"
            },
            radios: RadioButtonGroup = ([r1, r2, r3]),
            push_button: Button = (&window) => {
                text: "Push",
            },
            pop_button: Button = (&window) => {
                text: "Pop",
                tooltip: "Pop the last entry in the combo box."
            },
            show_button: Button = (&window) => {
                text: "Show",
                tooltip: "Show the current selection in the combo box.\nIf no selection, show \"No selection.\"",
            },
            progress: Progress = (&window) => {
                indeterminate: true,
            },
            mltext: TextBox = (&window) => {
                text: "This is an example of\nmulti-line text box.",
            },
        }

        window.show()?;

        Ok(Self {
            window,
            link,
            ulabel,
            plabel,
            uentry,
            pentry,
            pcheck,
            canvas,
            combo,
            list,
            radios,
            rindex: 0,
            push_button,
            pop_button,
            show_button,
            progress,
            mltext,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        start! {
            sender, default: MainMessage::Noop,
            self.window => {
                WindowEvent::Close => MainMessage::Close,
                WindowEvent::Resize => MainMessage::Redraw,
            },
            self.link => {},
            self.pcheck => {
                CheckBoxEvent::Click => MainMessage::PasswordCheck,
            },
            self.combo => {
                ComboBoxEvent::Select => MainMessage::Select,
            },
            self.push_button => {
                ButtonEvent::Click => MainMessage::Push,
            },
            self.pop_button => {
                ButtonEvent::Click => MainMessage::Pop,
            },
            self.show_button => {
                ButtonEvent::Click => MainMessage::Show,
            },
            self.list => {
                e => MainMessage::List(e),
            },
            self.radios => {
                RadioButtonGroupEvent::Click(i) => MainMessage::RSelect(i)
            },
        }
    }

    async fn update_children(&mut self) -> Result<bool> {
        update_children!(
            self.window,
            self.link,
            self.ulabel,
            self.plabel,
            self.uentry,
            self.pentry,
            self.pcheck,
            self.canvas,
            self.combo,
            self.list,
            self.radios,
            self.push_button,
            self.pop_button,
            self.show_button,
            self.progress,
            self.mltext,
        )
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            MainMessage::Noop => Ok(false),
            MainMessage::Close => {
                sender.output(());
                Ok(false)
            }
            MainMessage::Redraw => Ok(true),
            MainMessage::PasswordCheck => {
                self.pentry.set_password(!self.pcheck.is_checked()?)?;
                Ok(true)
            }
            MainMessage::List(e) => {
                self.pop_button.set_enabled(!self.list.is_empty())?;
                Ok(self
                    .combo
                    .emit(ComboBoxMessage::from_observable_vec_event(e))
                    .await?)
            }
            MainMessage::Select => Ok(true),
            MainMessage::Push => {
                self.list.push(self.radios[self.rindex].text()?);
                Ok(false)
            }
            MainMessage::Pop => {
                self.list.pop();
                Ok(false)
            }
            MainMessage::RSelect(i) => {
                self.rindex = i;
                let text = self.radios[self.rindex].text()?;
                self.push_button
                    .set_tooltip(format!("Push \"{text}\" to the back of the combo box."))?;
                Ok(false)
            }
            MainMessage::Show => {
                MessageBox::new()
                    .title("Show selected item")
                    .message(
                        self.combo
                            .selection()?
                            .and_then(|index| self.list.get(index))
                            .map(|s| s.as_str())
                            .unwrap_or("No selection."),
                    )
                    .buttons(MessageBoxButton::Ok)
                    .show(&self.window)
                    .await?;
                Ok(false)
            }
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) -> Result<()> {
        let csize = self.window.client_size()?;
        {
            let mut cred_panel = layout! {
                Grid::from_str("auto,1*,auto", "2*,auto,1*,auto,auto,2*").unwrap(),
                self.link   => { column: 0, row: 1, column_span: 3, halign: HAlign::Center, margin: Margin::new_all_same(4.0) },
                self.ulabel => { column: 0, row: 3, valign: VAlign::Center },
                self.uentry => { column: 1, row: 3, margin: Margin::new_all_same(4.0) },
                self.plabel => { column: 0, row: 4, valign: VAlign::Center },
                self.pentry => { column: 1, row: 4, margin: Margin::new_all_same(4.0) },
                self.pcheck => { column: 2, row: 4 },
            };

            let mut rgroup_panel = Grid::from_str("auto", "1*,auto,auto,auto,1*").unwrap();

            for (i, rb) in self.radios.iter_mut().enumerate() {
                rgroup_panel.push(rb).row(i + 1).finish();
            }

            let mut buttons_panel = layout! {
                StackPanel::new(Orient::Vertical),
                self.push_button => { margin: Margin::new_all_same(4.0) },
                self.pop_button  => { margin: Margin::new_all_same(4.0) },
                self.show_button => { margin: Margin::new_all_same(4.0) },
            };

            let mut root_panel = if csize.width < csize.height {
                layout! {
                    Grid::from_str("1*", "1*,1*,auto,auto,1*,1*,1*").unwrap(),
                    cred_panel    => { column: 0, row: 0 },
                    rgroup_panel  => { column: 0, row: 1, halign: HAlign::Center },
                    self.combo    => { column: 0, row: 2, halign: HAlign::Center },
                    self.progress => { column: 0, row: 3 },
                    self.canvas   => { column: 0, row: 4 },
                    self.mltext   => { column: 0, row: 5, margin: Margin::new_all_same(8.0) },
                    buttons_panel => { column: 0, row: 6 },
                }
            } else {
                layout! {
                    Grid::from_str("1*,1*,1*", "1*,auto,1*").unwrap(),
                    cred_panel    => { column: 1, row: 0 },
                    rgroup_panel  => { column: 2, row: 0, halign: HAlign::Center },
                    self.canvas   => { column: 0, row: 1, row_span: 2 },
                    self.combo    => { column: 1, row: 1, halign: HAlign::Center },
                    self.progress => { column: 2, row: 1 },
                    self.mltext   => { column: 1, row: 2, margin: Margin::new_all_same(8.0) },
                    buttons_panel => { column: 2, row: 2 },
                }
            };

            root_panel.set_size(csize)?;
        }

        let size = self.canvas.size()?;
        let is_dark = ColorTheme::current()? == ColorTheme::Dark;
        let back_color = if is_dark {
            Color::new(255, 255, 255, 255)
        } else {
            Color::new(0, 0, 0, 255)
        };
        let brush = SolidColorBrush::new(back_color);
        let pen = BrushPen::new(&brush, 1.0);
        let mut ctx = self.canvas.context()?;
        let cx = size.width / 2.0;
        let cy = size.height / 2.0;
        let r = cx.min(cy) - 2.0;
        ctx.draw_pie(
            &pen,
            Rect::new(Point::new(cx - r, cy - r), Size::new(r * 2.0, r * 2.0)),
            std::f64::consts::PI,
            std::f64::consts::PI * 2.0,
        )?;

        let brush2 = LinearGradientBrush::new(
            [
                GradientStop::new(Color::new(0x87, 0xCE, 0xEB, 0xFF), 0.0),
                GradientStop::new(back_color, 1.0),
            ],
            RelativePoint::zero(),
            RelativePoint::new(0.0, 1.0),
        );
        let pen2 = BrushPen::new(&brush2, 1.0);
        ctx.draw_round_rect(
            &pen2,
            Rect::new(
                Point::new(cx - r - 1.0, cy - r - 1.0),
                Size::new(r * 2.0 + 2.0, r * 1.618 + 2.0),
            ),
            Size::new(r / 10.0, r / 10.0),
        )?;
        let mut path = ctx.create_path_builder(Point::new(cx + r + 1.0 - r / 10.0, cy))?;
        path.add_arc(
            Point::new(cx, cy + r * 0.618 + 1.0),
            Size::new(r + 1.0 - r / 10.0, r * 0.382 / 2.0),
            0.0,
            std::f64::consts::PI,
            true,
        )?;
        path.add_line(Point::new(cx - r - 1.0 + r / 10.0, cy))?;
        let path = path.build(false)?;
        ctx.draw_path(&pen, &path)?;
        let brush3 = RadialGradientBrush::new(
            [
                GradientStop::new(Color::new(0xF5, 0xF5, 0xF5, 0xFF), 0.0),
                GradientStop::new(
                    Color::accent().unwrap_or(Color::new(0xFF, 0xC0, 0xCB, 0xFF)),
                    1.0,
                ),
            ],
            RelativePoint::new(0.5, 0.5),
            RelativePoint::new(0.2, 0.5),
            RelativeSize::new(0.5, 0.5),
        );
        let font = DrawingFontBuilder::new()
            .family("Arial")
            .size(r / 5.0)
            .halign(HAlign::Center)
            .valign(VAlign::Bottom)
            .build();
        ctx.draw_str(&brush3, font, Point::new(cx, cy), "Hello world!")?;

        ctx.close()?;

        Ok(())
    }
}
