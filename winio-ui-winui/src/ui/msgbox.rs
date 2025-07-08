use windows::core::HSTRING;
use winio_handle::AsWindow;
use winio_primitive::{MessageBoxButton, MessageBoxResponse, MessageBoxStyle};

async fn msgbox_custom(
    parent: Option<impl AsWindow>,
    msg: HSTRING,
    title: HSTRING,
    instr: HSTRING,
    style: MessageBoxStyle,
    btns: MessageBoxButton,
    cbtns: Vec<CustomButton>,
) -> MessageBoxResponse {
    let parent = parent.map(|p| p.as_window().as_winui().clone());

    todo!()
}

#[derive(Debug, Clone)]
pub struct MessageBox {
    msg: HSTRING,
    title: HSTRING,
    instr: HSTRING,
    style: MessageBoxStyle,
    btns: MessageBoxButton,
    cbtns: Vec<CustomButton>,
}

impl Default for MessageBox {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageBox {
    pub fn new() -> Self {
        Self {
            msg: HSTRING::new(),
            title: HSTRING::new(),
            instr: HSTRING::new(),
            style: MessageBoxStyle::None,
            btns: MessageBoxButton::empty(),
            cbtns: vec![],
        }
    }

    pub async fn show(self, parent: Option<impl AsWindow>) -> MessageBoxResponse {
        msgbox_custom(
            parent, self.msg, self.title, self.instr, self.style, self.btns, self.cbtns,
        )
        .await
    }

    pub fn message(&mut self, msg: &str) {
        self.msg = HSTRING::from(msg);
    }

    pub fn title(&mut self, title: &str) {
        self.title = HSTRING::from(title);
    }

    pub fn instruction(&mut self, instr: &str) {
        self.instr = HSTRING::from(instr);
    }

    pub fn style(&mut self, style: MessageBoxStyle) {
        self.style = style;
    }

    pub fn buttons(&mut self, btns: MessageBoxButton) {
        self.btns = btns;
    }

    pub fn custom_button(&mut self, btn: CustomButton) {
        self.cbtns.push(btn);
    }

    pub fn custom_buttons(&mut self, btn: impl IntoIterator<Item = CustomButton>) {
        self.cbtns.extend(btn);
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CustomButton {
    pub result: u16,
    pub text: HSTRING,
}

impl CustomButton {
    pub fn new(result: u16, text: &str) -> Self {
        Self {
            result,
            text: HSTRING::from(text),
        }
    }
}
