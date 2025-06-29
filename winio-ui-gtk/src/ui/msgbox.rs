use std::sync::LazyLock;

use gtk4::glib::{GString, dgettext};
use winio_handle::{AsRawWindow, AsWindow};
use winio_primitive::{MessageBoxButton, MessageBoxResponse, MessageBoxStyle};

struct PredefButtons {
    ok: GString,
    yes: GString,
    no: GString,
    cancel: GString,
    retry: GString,
    close: GString,
}

impl PredefButtons {
    pub fn new() -> Self {
        const DOMAIN: Option<&str> = Some("gtk40");
        Self {
            ok: dgettext(DOMAIN, "_Ok"),
            yes: dgettext(DOMAIN, "_Yes"),
            no: dgettext(DOMAIN, "_No"),
            cancel: dgettext(DOMAIN, "_Cancel"),
            retry: dgettext(DOMAIN, "_Retry"),
            close: dgettext(DOMAIN, "_Close"),
        }
    }
}

static PREDEF_BUTTONS: LazyLock<PredefButtons> = LazyLock::new(PredefButtons::new);

async fn msgbox_custom(
    parent: Option<impl AsWindow>,
    msg: String,
    _title: String,
    instr: String,
    _style: MessageBoxStyle,
    btns: MessageBoxButton,
    cbtns: Vec<CustomButton>,
) -> MessageBoxResponse {
    let predef = &*PREDEF_BUTTONS;
    let mut buttons = Vec::<&str>::new();
    let mut results = vec![];
    if btns.contains(MessageBoxButton::Ok) {
        buttons.push(&predef.ok);
        results.push(MessageBoxResponse::Ok);
    }
    if btns.contains(MessageBoxButton::Yes) {
        buttons.push(&predef.yes);
        results.push(MessageBoxResponse::Yes);
    }
    if btns.contains(MessageBoxButton::No) {
        buttons.push(&predef.no);
        results.push(MessageBoxResponse::No);
    }
    if btns.contains(MessageBoxButton::Cancel) {
        buttons.push(&predef.cancel);
        results.push(MessageBoxResponse::Cancel);
    }
    if btns.contains(MessageBoxButton::Retry) {
        buttons.push(&predef.retry);
        results.push(MessageBoxResponse::Retry);
    }
    if btns.contains(MessageBoxButton::Close) {
        buttons.push(&predef.close);
        results.push(MessageBoxResponse::Close);
    }
    for b in &cbtns {
        buttons.push(&b.text);
        results.push(MessageBoxResponse::Custom(b.result))
    }

    let mut builder = gtk4::AlertDialog::builder().modal(true).buttons(buttons);
    if instr.is_empty() {
        builder = builder.message(msg);
    } else {
        builder = builder.message(instr).detail(msg)
    }
    let dialog = builder.build();

    let res = dialog
        .choose_future(parent.map(|w| w.as_window().as_raw_window()).as_ref())
        .await
        .ok();

    res.map(|res| results[res as usize])
        .unwrap_or(MessageBoxResponse::Cancel)
}

#[derive(Debug, Clone)]
pub struct MessageBox {
    msg: String,
    title: String,
    instr: String,
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
            msg: String::new(),
            title: String::new(),
            instr: String::new(),
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
        self.msg = msg.to_string();
    }

    pub fn title(&mut self, title: &str) {
        self.title = title.to_string();
    }

    pub fn instruction(&mut self, instr: &str) {
        self.instr = instr.to_string();
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
    pub text: String,
}

impl CustomButton {
    pub fn new(result: u16, text: &str) -> Self {
        Self {
            result,
            text: text.to_string(),
        }
    }
}
