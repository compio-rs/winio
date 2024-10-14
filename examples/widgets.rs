use taffy::{
    NodeId, TaffyTree,
    prelude::{auto, fr, length, line, percent, repeat},
};
use winio::{
    App, BrushPen, Button, ButtonEvent, Canvas, Child, Color, ColorTheme, ComboBox, ComboBoxEvent,
    ComboBoxMessage, Component, ComponentSender, Edit, HAlign, Label, Layoutable, MessageBox,
    MessageBoxButton, ObservableVec, ObservableVecEvent, PasswordEdit, Point, Rect, Size,
    SolidColorBrush, Window, WindowEvent,
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
    eheight: f64,
    canvas: Child<Canvas>,
    combo: Child<ComboBox>,
    list: Child<ObservableVec<String>>,
    index: Option<usize>,
    push_button: Child<Button>,
    pop_button: Child<Button>,
    show_button: Child<Button>,
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
        let eheight = uentry.size().height;
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

        Self {
            window,
            ulabel,
            plabel,
            uentry,
            pentry,
            eheight,
            canvas,
            combo,
            list,
            index: None,
            push_button,
            pop_button,
            show_button,
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
        let (
            ulabel_rect,
            uentry_rect,
            plabel_rect,
            pentry_rect,
            combo_rect,
            canvas_rect,
            b1_rect,
            b2_rect,
            b3_rect,
        ) = Layout::new(self.eheight).compute(csize);
        self.ulabel.set_rect(ulabel_rect);
        self.uentry.set_rect(uentry_rect);
        self.plabel.set_rect(plabel_rect);
        self.pentry.set_rect(pentry_rect);
        self.combo.set_rect(combo_rect);
        self.canvas.set_rect(canvas_rect);
        self.push_button.set_rect(b1_rect);
        self.pop_button.set_rect(b2_rect);
        self.show_button.set_rect(b3_rect);

        let size = self.canvas.size();
        let brush = SolidColorBrush::new(if self.is_dark {
            Color::new(255, 255, 255, 255)
        } else {
            Color::new(0, 0, 0, 255)
        });
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
    }
}

struct Layout {
    taffy: TaffyTree,
    cred: NodeId,
    ulabel: NodeId,
    uentry: NodeId,
    plabel: NodeId,
    pentry: NodeId,
    canvas: NodeId,
    combo: NodeId,
    buttons: NodeId,
    b1: NodeId,
    b2: NodeId,
    b3: NodeId,
    root: NodeId,
}

impl Layout {
    pub fn new(eheight: f64) -> Self {
        let mut taffy = TaffyTree::new();
        let combo = taffy
            .new_leaf(taffy::Style {
                size: taffy::Size {
                    width: auto(),
                    height: length(50.0),
                },
                grid_column: line(2),
                grid_row: line(2),
                margin: taffy::Rect {
                    left: percent(0.0),
                    right: percent(0.0),
                    top: auto(),
                    bottom: auto(),
                },
                ..Default::default()
            })
            .unwrap();
        let canvas = taffy
            .new_leaf(taffy::Style {
                size: auto(),
                grid_column: line(1),
                grid_row: line(3),
                ..Default::default()
            })
            .unwrap();

        let b1 = taffy
            .new_leaf(taffy::Style {
                size: taffy::Size {
                    width: auto(),
                    height: length(30.0),
                },
                margin: taffy::Rect {
                    left: length(0.0),
                    right: length(0.0),
                    top: length(4.0),
                    bottom: length(4.0),
                },
                ..Default::default()
            })
            .unwrap();
        let b2 = taffy
            .new_leaf(taffy::Style {
                size: taffy::Size {
                    width: auto(),
                    height: length(30.0),
                },
                margin: taffy::Rect {
                    left: length(0.0),
                    right: length(0.0),
                    top: length(4.0),
                    bottom: length(4.0),
                },
                ..Default::default()
            })
            .unwrap();
        let b3 = taffy
            .new_leaf(taffy::Style {
                size: taffy::Size {
                    width: auto(),
                    height: length(30.0),
                },
                margin: taffy::Rect {
                    left: length(0.0),
                    right: length(0.0),
                    top: length(4.0),
                    bottom: length(4.0),
                },
                ..Default::default()
            })
            .unwrap();
        let buttons = taffy
            .new_with_children(
                taffy::Style {
                    size: auto(),
                    grid_column: line(3),
                    grid_row: line(3),
                    flex_direction: taffy::FlexDirection::Column,
                    ..Default::default()
                },
                &[b1, b2, b3],
            )
            .unwrap();

        let ulabel = taffy
            .new_leaf(taffy::Style {
                size: auto(),
                grid_column: line(1),
                grid_row: line(2),
                ..Default::default()
            })
            .unwrap();
        let uentry = taffy
            .new_leaf(taffy::Style {
                size: taffy::Size {
                    width: auto(),
                    height: length(eheight as f32),
                },
                grid_column: line(2),
                grid_row: line(2),
                margin: taffy::Rect {
                    left: length(4.0),
                    right: length(0.0),
                    top: length(4.0),
                    bottom: length(4.0),
                },
                ..Default::default()
            })
            .unwrap();
        let plabel = taffy
            .new_leaf(taffy::Style {
                size: auto(),
                grid_column: line(1),
                grid_row: line(3),
                ..Default::default()
            })
            .unwrap();
        let pentry = taffy
            .new_leaf(taffy::Style {
                size: taffy::Size {
                    width: auto(),
                    height: length(eheight as f32),
                },
                grid_column: line(2),
                grid_row: line(3),
                margin: taffy::Rect {
                    left: length(4.0),
                    right: length(0.0),
                    top: length(4.0),
                    bottom: length(4.0),
                },
                ..Default::default()
            })
            .unwrap();

        let cred = taffy
            .new_with_children(
                taffy::Style {
                    display: taffy::Display::Grid,
                    size: auto(),
                    grid_column: line(2),
                    grid_row: line(1),
                    grid_template_columns: vec![length(100.0), fr(1.0)],
                    grid_template_rows: vec![fr(1.0), auto(), auto(), fr(1.0)],
                    ..Default::default()
                },
                &[ulabel, uentry, plabel, pentry],
            )
            .unwrap();

        let root = taffy
            .new_with_children(
                taffy::Style {
                    display: taffy::Display::Grid,
                    size: taffy::Size::from_percent(1.0, 1.0),
                    grid_template_columns: vec![repeat(3, vec![fr(1.0)])],
                    grid_template_rows: vec![fr(1.0), length(50.0), fr(1.0)],
                    ..Default::default()
                },
                &[cred, combo, canvas, buttons],
            )
            .unwrap();
        Self {
            taffy,
            cred,
            ulabel,
            uentry,
            plabel,
            pentry,
            canvas,
            combo,
            buttons,
            b1,
            b2,
            b3,
            root,
        }
    }

    pub fn compute(
        mut self,
        csize: Size,
    ) -> (Rect, Rect, Rect, Rect, Rect, Rect, Rect, Rect, Rect) {
        self.taffy
            .compute_layout(self.root, taffy::Size {
                width: taffy::AvailableSpace::Definite(csize.width as _),
                height: taffy::AvailableSpace::Definite(csize.height as _),
            })
            .unwrap();
        let combo_rect = self.taffy.layout(self.combo).unwrap();
        let canvas_rect = self.taffy.layout(self.canvas).unwrap();
        let buttons_rect = self.taffy.layout(self.buttons).unwrap();
        let b1_rect = self.taffy.layout(self.b1).unwrap();
        let b2_rect = self.taffy.layout(self.b2).unwrap();
        let b3_rect = self.taffy.layout(self.b3).unwrap();
        let cred_rect = self.taffy.layout(self.cred).unwrap();
        let ulabel_rect = self.taffy.layout(self.ulabel).unwrap();
        let uentry_rect = self.taffy.layout(self.uentry).unwrap();
        let plabel_rect = self.taffy.layout(self.plabel).unwrap();
        let pentry_rect = self.taffy.layout(self.pentry).unwrap();

        let buttons_rect = rect_t2e(buttons_rect);
        let cred_rect = rect_t2e(cred_rect);
        (
            offset(rect_t2e(ulabel_rect), cred_rect),
            offset(rect_t2e(uentry_rect), cred_rect),
            offset(rect_t2e(plabel_rect), cred_rect),
            offset(rect_t2e(pentry_rect), cred_rect),
            rect_t2e(combo_rect),
            rect_t2e(canvas_rect),
            offset(rect_t2e(b1_rect), buttons_rect),
            offset(rect_t2e(b2_rect), buttons_rect),
            offset(rect_t2e(b3_rect), buttons_rect),
        )
    }
}

fn rect_t2e(rect: &taffy::Layout) -> Rect {
    Rect::new(
        Point::new(rect.location.x as _, rect.location.y as _),
        Size::new(rect.size.width as _, rect.size.height as _),
    )
}

fn offset(mut a: Rect, offset: Rect) -> Rect {
    a.origin += offset.origin.to_vector();
    a
}
