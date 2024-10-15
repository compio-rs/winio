use winio::{
    App, BrushPen, Button, ButtonEvent, Canvas, Child, Color, ColorTheme, ComboBox, ComboBoxEvent,
    ComboBoxMessage, Component, ComponentSender, DrawingFontBuilder, Edit, GradientStop, GridPanel,
    HAlign, Label, Layoutable, LinearGradientBrush, Margin, MessageBox, MessageBoxButton,
    ObservableVec, ObservableVecEvent, Orient, PasswordEdit, Point, Progress, RadialGradientBrush,
    Rect, RelativePoint, RelativeSize, Size, SolidColorBrush, StackPanel, VAlign, Window,
    WindowEvent,
};

fn main() {
    #[cfg(feature = "enable_log")]
    tracing_subscriber::fmt()
        .with_max_level(compio_log::Level::INFO)
        .init();

    App::new().run::<MainModel>((), &());
}

struct MainModel {
    window: Child<Window>,
    ulabel: Child<Label>,
    plabel: Child<Label>,
    uentry: Child<Edit>,
    pentry: Child<PasswordEdit>,
    canvas: Child<Canvas>,
    combo: Child<ComboBox>,
    list: Child<ObservableVec<String>>,
    index: Option<usize>,
    push_button: Child<Button>,
    pop_button: Child<Button>,
    show_button: Child<Button>,
    progress: Child<Progress>,
    is_dark: bool,
}

#[derive(Debug)]
enum MainMessage {
    Close,
    Redraw,
    List(ObservableVecEvent<String>),
    Select,
    Push,
    Pop,
    Show,
}

impl Component for MainModel {
    type Event = ();
    type Init = ();
    type Message = MainMessage;
    type Root = ();

    fn init(_counter: Self::Init, _root: &Self::Root, sender: &ComponentSender<Self>) -> Self {
        let mut window = Child::<Window>::init((), &());
        let canvas = Child::<Canvas>::init((), &window);

        window.set_text("Widgets example");
        window.set_size(Size::new(800.0, 600.0));

        let mut ulabel = Child::<Label>::init((), &window);
        ulabel.set_text("Username:");
        ulabel.set_halign(HAlign::Right);
        let mut plabel = Child::<Label>::init((), &window);
        plabel.set_text("Password:");
        plabel.set_halign(HAlign::Right);

        let mut uentry = Child::<Edit>::init((), &window);
        uentry.set_text("AAA");
        let mut pentry = Child::<PasswordEdit>::init((), &window);
        pentry.set_text("123456");

        let combo = Child::<ComboBox>::init((), &window);

        let mut list = Child::<ObservableVec<String>>::init(vec![], &());
        list.push("锟斤拷".into());
        list.push("烫烫烫".into());
        list.push("フフフ".into());

        sender.post(MainMessage::Redraw);

        let mut push_button = Child::<Button>::init((), &window);
        push_button.set_text("Push");
        let mut pop_button = Child::<Button>::init((), &window);
        pop_button.set_text("Pop");
        let mut show_button = Child::<Button>::init((), &window);
        show_button.set_text("Show");

        let mut progress = Child::<Progress>::init((), &window);
        progress.set_indeterminate(true);

        Self {
            window,
            ulabel,
            plabel,
            uentry,
            pentry,
            canvas,
            combo,
            list,
            index: None,
            push_button,
            pop_button,
            show_button,
            progress,
            is_dark: ColorTheme::current() == ColorTheme::Dark,
        }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) {
        let fut_window = self.window.start(sender, |e| match e {
            WindowEvent::Close => Some(MainMessage::Close),
            WindowEvent::Move | WindowEvent::Resize => Some(MainMessage::Redraw),
            _ => None,
        });
        let fut_combo = self.combo.start(sender, |e| match e {
            ComboBoxEvent::Select => Some(MainMessage::Select),
            _ => None,
        });
        let fut_list = self.list.start(sender, |e| Some(MainMessage::List(e)));
        let fut_push = self.push_button.start(sender, |e| match e {
            ButtonEvent::Click => Some(MainMessage::Push),
            _ => None,
        });
        let fut_pop = self.pop_button.start(sender, |e| match e {
            ButtonEvent::Click => Some(MainMessage::Pop),
            _ => None,
        });
        let fut_show = self.show_button.start(sender, |e| match e {
            ButtonEvent::Click => Some(MainMessage::Show),
            _ => None,
        });
        futures_util::join!(fut_window, fut_combo, fut_list, fut_push, fut_pop, fut_show);
    }

    async fn update(&mut self, message: Self::Message, sender: &ComponentSender<Self>) -> bool {
        futures_util::future::join(self.window.update(), self.canvas.update()).await;
        match message {
            MainMessage::Close => {
                sender.output(());
                false
            }
            MainMessage::Redraw => true,
            MainMessage::List(e) => {
                self.combo
                    .emit(ComboBoxMessage::from_observable_vec_event(e))
                    .await
            }
            MainMessage::Select => {
                self.index = self.combo.selection();
                false
            }
            MainMessage::Push => {
                self.list.push("屯屯屯".into());
                false
            }
            MainMessage::Pop => {
                self.list.pop();
                false
            }
            MainMessage::Show => {
                MessageBox::new()
                    .title("Show selected item")
                    .message(if let Some(index) = self.index {
                        self.list[index].as_str()
                    } else {
                        "No selection."
                    })
                    .buttons(MessageBoxButton::Ok)
                    .show(Some(&*self.window))
                    .await;
                false
            }
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {
        self.window.render();
        self.canvas.render();

        let csize = self.window.client_size();
        {
            let mut root_panel = GridPanel::from_str("1*,1*,1*", "1*,auto,1*").unwrap();
            let mut cred_panel = GridPanel::from_str("auto,1*", "1*,auto,auto,1*").unwrap();
            cred_panel
                .push(&mut self.ulabel)
                .valign(VAlign::Center)
                .column(0)
                .row(1)
                .finish();
            cred_panel
                .push(&mut self.uentry)
                .margin(Margin::new_all_same(4.0))
                .column(1)
                .row(1)
                .finish();
            cred_panel
                .push(&mut self.plabel)
                .valign(VAlign::Center)
                .column(0)
                .row(2)
                .finish();
            cred_panel
                .push(&mut self.pentry)
                .margin(Margin::new_all_same(4.0))
                .column(1)
                .row(2)
                .finish();
            root_panel.push(&mut cred_panel).column(1).row(0).finish();

            root_panel
                .push(&mut self.combo)
                .halign(HAlign::Center)
                .column(1)
                .row(1)
                .finish();
            root_panel
                .push(&mut self.progress)
                .column(2)
                .row(1)
                .finish();

            root_panel.push(&mut self.canvas).column(0).row(2).finish();

            let mut buttons_panel = StackPanel::new(Orient::Vertical);
            buttons_panel
                .push(&mut self.push_button)
                .margin(Margin::new_all_same(4.0))
                .finish();
            buttons_panel
                .push(&mut self.pop_button)
                .margin(Margin::new_all_same(4.0))
                .finish();
            buttons_panel
                .push(&mut self.show_button)
                .margin(Margin::new_all_same(4.0))
                .finish();
            root_panel
                .push(&mut buttons_panel)
                .column(2)
                .row(2)
                .finish();

            root_panel.set_size(csize);
        }

        let size = self.canvas.size();
        let back_color = if self.is_dark {
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
        ctx.draw_arc(
            &pen,
            Rect::new(Point::new(cx - r, cy - r), Size::new(r * 2.0, r * 2.0)),
            std::f64::consts::PI,
            std::f64::consts::PI * 2.0,
        );
        ctx.draw_line(&pen, Point::new(cx - r, cy), Point::new(cx + r, cy));

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
        ctx.draw_arc(
            &pen,
            Rect::new(
                Point::new(
                    cx - r - 1.0 + r / 10.0,
                    cy + r * 0.618 + 1.0 - r * 0.382 / 2.0,
                ),
                Size::new(r * 2.0 - r / 5.0, r * 0.382),
            ),
            0.0,
            std::f64::consts::PI,
        );
        let brush3 = RadialGradientBrush::new(
            [
                GradientStop::new(Color::new(0xF5, 0xF5, 0xF5, 0xFF), 0.0),
                GradientStop::new(Color::new(0xFF, 0xC0, 0xCB, 0xFF), 1.0),
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
