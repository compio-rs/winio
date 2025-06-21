use winio::{
    App, BrushPen, Button, ButtonEvent, Canvas, CanvasEvent, CheckBox, CheckBoxEvent, Child, Color,
    ColorTheme, ComboBox, ComboBoxEvent, ComboBoxMessage, Component, ComponentSender,
    DrawingFontBuilder, Edit, Enable, GradientStop, Grid, HAlign, Label, Layoutable,
    LinearGradientBrush, Margin, MessageBox, MessageBoxButton, ObservableVec, ObservableVecEvent,
    Orient, Point, Progress, RadialGradientBrush, RadioButton, RadioButtonGroup, Rect,
    RelativePoint, RelativeSize, Size, SolidColorBrush, StackPanel, TextBox, VAlign, Visible,
    Window, WindowEvent, accent_color, init, layout, start,
};

fn main() {
    #[cfg(feature = "enable_log")]
    tracing_subscriber::fmt()
        .with_max_level(compio_log::Level::INFO)
        .init();

    App::new().run::<MainModel>(());
}

struct MainModel {
    window: Child<Window>,
    ulabel: Child<Label>,
    plabel: Child<Label>,
    uentry: Child<Edit>,
    pentry: Child<Edit>,
    pcheck: Child<CheckBox>,
    canvas: Child<Canvas>,
    combo: Child<ComboBox>,
    list: Child<ObservableVec<String>>,
    index: Option<usize>,
    r1: Child<RadioButton>,
    r2: Child<RadioButton>,
    r3: Child<RadioButton>,
    rindex: usize,
    push_button: Child<Button>,
    pop_button: Child<Button>,
    show_button: Child<Button>,
    progress: Child<Progress>,
    mltext: Child<TextBox>,
}

#[derive(Debug)]
enum MainMessage {
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
    type Event = ();
    type Init<'a> = ();
    type Message = MainMessage;

    fn init(_init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Self {
        init! {
            window: Window = (()) => {
                text: "Widgets example",
                size: Size::new(800.0, 600.0),
            },
            canvas: Canvas = (&window),
            ulabel: Label = (&window) => {
                text: "Username:",
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
            list: ObservableVec<String> = (vec![]) => {
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
            push_button: Button = (&window) => {
                text: "Push",
            },
            pop_button: Button = (&window) => {
                text: "Pop",
            },
            show_button: Button = (&window) => {
                text: "Show",
            },
            progress: Progress = (&window) => {
                indeterminate: true,
            },
            mltext: TextBox = (&window) => {
                text: "This is an example of\nmulti-line text box.",
            },
        }

        window.show();

        Self {
            window,
            ulabel,
            plabel,
            uentry,
            pentry,
            pcheck,
            canvas,
            combo,
            list,
            index: None,
            r1,
            r2,
            r3,
            rindex: 0,
            push_button,
            pop_button,
            show_button,
            progress,
            mltext,
        }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) {
        let mut radio_group = RadioButtonGroup::new([&mut *self.r1, &mut self.r2, &mut self.r3]);
        start! {
            sender, default: MainMessage::Noop,
            self.window => {
                WindowEvent::Close => MainMessage::Close,
                WindowEvent::Resize => MainMessage::Redraw,
            },
            self.pcheck => {
                CheckBoxEvent::Click => MainMessage::PasswordCheck,
            },
            self.combo => {
                ComboBoxEvent::Select => MainMessage::Select,
            },
            self.canvas => {
                CanvasEvent::Redraw => MainMessage::Redraw,
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
            radio_group => {
                |i| Some(MainMessage::RSelect(i))
            }
        }
    }

    async fn update(&mut self, message: Self::Message, sender: &ComponentSender<Self>) -> bool {
        futures_util::future::join(self.window.update(), self.canvas.update()).await;
        match message {
            MainMessage::Noop => false,
            MainMessage::Close => {
                sender.output(());
                false
            }
            MainMessage::Redraw => true,
            MainMessage::PasswordCheck => {
                self.pentry.set_password(!self.pcheck.is_checked());
                true
            }
            MainMessage::List(e) => {
                self.pop_button.set_enabled(!self.list.is_empty());
                self.combo
                    .emit(ComboBoxMessage::from_observable_vec_event(e))
                    .await
            }
            MainMessage::Select => {
                self.index = self.combo.selection();
                false
            }
            MainMessage::Push => {
                self.list.push(
                    match self.rindex {
                        0 => &self.r1,
                        1 => &self.r2,
                        2 => &self.r3,
                        _ => unreachable!(),
                    }
                    .text(),
                );
                false
            }
            MainMessage::Pop => {
                self.list.pop();
                false
            }
            MainMessage::RSelect(i) => {
                self.rindex = i;
                false
            }
            MainMessage::Show => {
                MessageBox::new()
                    .title("Show selected item")
                    .message(
                        self.index
                            .and_then(|index| self.list.get(index))
                            .map(|s| s.as_str())
                            .unwrap_or("No selection."),
                    )
                    .buttons(MessageBoxButton::Ok)
                    .show(&self.window)
                    .await;
                false
            }
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {
        let csize = self.window.client_size();
        {
            let mut cred_panel = layout! {
                Grid::from_str("auto,1*,auto", "1*,auto,auto,1*").unwrap(),
                self.ulabel => { column: 0, row: 1, valign: VAlign::Center },
                self.uentry => { column: 1, row: 1, margin: Margin::new_all_same(4.0) },
                self.plabel => { column: 0, row: 2, valign: VAlign::Center },
                self.pentry => { column: 1, row: 2, margin: Margin::new_all_same(4.0) },
                self.pcheck => { column: 2, row: 2 },
            };

            let mut rgroup_panel = layout! {
                Grid::from_str("auto", "1*,auto,auto,auto,1*").unwrap(),
                self.r1 => { row: 1 },
                self.r2 => { row: 2 },
                self.r3 => { row: 3 },
            };

            let mut buttons_panel = layout! {
                StackPanel::new(Orient::Vertical),
                self.push_button => { margin: Margin::new_all_same(4.0) },
                self.pop_button  => { margin: Margin::new_all_same(4.0) },
                self.show_button => { margin: Margin::new_all_same(4.0) },
            };

            let mut root_panel = layout! {
                Grid::from_str("1*,1*,1*", "1*,auto,1*").unwrap(),
                cred_panel    => { column: 1, row: 0 },
                rgroup_panel  => { column: 2, row: 0, halign: HAlign::Center },
                self.canvas   => { column: 0, row: 1, row_span: 2 },
                self.combo    => { column: 1, row: 1, halign: HAlign::Center },
                self.progress => { column: 2, row: 1 },
                self.mltext   => { column: 1, row: 2, margin: Margin::new_all_same(8.0) },
                buttons_panel => { column: 2, row: 2 },
            };

            root_panel.set_size(csize);
        }

        let size = self.canvas.size();
        let is_dark = ColorTheme::current() == ColorTheme::Dark;
        let back_color = if is_dark {
            Color::new(255, 255, 255, 255)
        } else {
            Color::new(0, 0, 0, 255)
        };
        let brush = SolidColorBrush::new(back_color);
        let pen = BrushPen::new(&brush, 1.0);
        let mut ctx = self.canvas.context();
        let cx = size.width / 2.0;
        let cy = size.height / 2.0;
        let r = cx.min(cy) - 2.0;
        ctx.draw_pie(
            &pen,
            Rect::new(Point::new(cx - r, cy - r), Size::new(r * 2.0, r * 2.0)),
            std::f64::consts::PI,
            std::f64::consts::PI * 2.0,
        );

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
        );
        let mut path = ctx.create_path_builder(Point::new(cx + r + 1.0 - r / 10.0, cy));
        path.add_arc(
            Point::new(cx, cy + r * 0.618 + 1.0),
            Size::new(r + 1.0 - r / 10.0, r * 0.382 / 2.0),
            0.0,
            std::f64::consts::PI,
            true,
        );
        path.add_line(Point::new(cx - r - 1.0 + r / 10.0, cy));
        let path = path.build(false);
        ctx.draw_path(&pen, &path);
        let brush3 = RadialGradientBrush::new(
            [
                GradientStop::new(Color::new(0xF5, 0xF5, 0xF5, 0xFF), 0.0),
                GradientStop::new(
                    accent_color().unwrap_or(Color::new(0xFF, 0xC0, 0xCB, 0xFF)),
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
        ctx.draw_str(&brush3, font, Point::new(cx, cy), "Hello world!");
    }
}
