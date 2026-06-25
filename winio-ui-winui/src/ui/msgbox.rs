use std::borrow::Cow;

use windows::Foundation::PropertyValue;
use windows::UI::Text::FontWeight;
use windows::Win32::Foundation::{E_INVALIDARG, E_NOTIMPL};
use windows::core::HSTRING;
use winio_handle::AsWindow;
use winio_primitive::{MessageBoxButton, MessageBoxResponse, MessageBoxStyle};
use winio_ui_windows_common::CustomButton;
use winui3::Microsoft::UI::Xaml::{
    Controls::{ContentDialog, ContentDialogButton, ContentDialogResult, StackPanel, TextBlock},
    TextWrapping, Thickness, XamlRoot,
};

use crate::{Error, Result};

#[derive(Clone, Copy)]
enum Slot {
    Primary,
    Secondary,
    Close,
}

struct ButtonMeta {
    flag: MessageBoxButton,
    slot: Slot,
    label: &'static str,
    response: MessageBoxResponse,
}

const BUTTON_META: [ButtonMeta; 6] = [
    ButtonMeta {
        flag: MessageBoxButton::Yes,
        slot: Slot::Primary,
        label: "Yes",
        response: MessageBoxResponse::Yes,
    },
    ButtonMeta {
        flag: MessageBoxButton::Ok,
        slot: Slot::Primary,
        label: "OK",
        response: MessageBoxResponse::Ok,
    },
    ButtonMeta {
        flag: MessageBoxButton::Retry,
        slot: Slot::Primary,
        label: "Retry",
        response: MessageBoxResponse::Retry,
    },
    ButtonMeta {
        flag: MessageBoxButton::Cancel,
        slot: Slot::Close,
        label: "Cancel",
        response: MessageBoxResponse::Cancel,
    },
    ButtonMeta {
        flag: MessageBoxButton::No,
        slot: Slot::Secondary,
        label: "No",
        response: MessageBoxResponse::No,
    },
    ButtonMeta {
        flag: MessageBoxButton::Close,
        slot: Slot::Close,
        label: "Close",
        response: MessageBoxResponse::Close,
    },
];

struct ButtonProps {
    label: Cow<'static, str>,
    response: MessageBoxResponse,
}

struct ArrangedButtons {
    primary: Option<ButtonProps>,
    secondary: Option<ButtonProps>,
    close: Option<ButtonProps>,
}

#[derive(Debug, Clone)]
pub struct MessageBox {
    msg: String,
    title: String,
    instr: String,
    btns: MessageBoxButton,
    cbtns: Vec<CustomButton>,
}

impl Default for MessageBox {
    fn default() -> Self {
        Self {
            msg: String::new(),
            title: String::new(),
            instr: String::new(),
            btns: MessageBoxButton::empty(),
            cbtns: Vec::new(),
        }
    }
}

impl MessageBox {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn show(self, parent: Option<impl AsWindow>) -> Result<MessageBoxResponse> {
        let window = parent.map(|p| p.as_window().as_winui().clone());
        let xaml_root = window
            .as_ref()
            .and_then(|w| w.Content().ok())
            .and_then(|c| c.XamlRoot().ok())
            .ok_or_else(|| Error::from_hresult(E_NOTIMPL))?;

        msgbox(
            &xaml_root,
            &self.msg,
            &self.title,
            &self.instr,
            self.btns,
            &self.cbtns,
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

    pub fn style(&mut self, _style: MessageBoxStyle) {}

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

/// Build the three ContentDialog button slots from standard + custom buttons.
///
/// Rules:
/// - Cancel/Close always occupies the Close slot so that Esc returns Cancel.
/// - Custom buttons are limited to Primary/Secondary; the Close slot is
///   reserved for Cancel or Close.
/// - When multiple standard buttons prefer the same slot, the higher-priority
///   one (earlier in `BUTTON_META`) wins and the displaced one tries the
///   next slot in fallback order.
/// - If no buttons at all, a default OK is placed in Primary.
fn arrange_buttons(btns: MessageBoxButton, cbtns: &[CustomButton]) -> Result<ArrangedButtons> {
    let supported = BUTTON_META
        .iter()
        .fold(MessageBoxButton::empty(), |acc, m| acc | m.flag);
    if !supported.contains(btns) {
        return Err(Error::from_hresult(E_NOTIMPL));
    }

    let std_count = BUTTON_META.iter().filter(|m| btns.contains(m.flag)).count();
    if std_count + cbtns.len() > 3 {
        return Err(Error::from_hresult(E_INVALIDARG));
    }

    let has_cancel = btns.contains(MessageBoxButton::Cancel);
    let has_close = btns.contains(MessageBoxButton::Close);
    if std_count + cbtns.len() == 3 && !has_cancel && !has_close {
        return Err(Error::from_hresult(E_INVALIDARG));
    }

    let std_non_dismiss_count = BUTTON_META
        .iter()
        .filter(|m| {
            btns.contains(m.flag)
                && m.response != MessageBoxResponse::Cancel
                && m.response != MessageBoxResponse::Close
        })
        .count();
    let available_custom_slots = 2usize.saturating_sub(std_non_dismiss_count);
    if cbtns.len() > available_custom_slots {
        return Err(Error::from_hresult(E_INVALIDARG));
    }

    if std_count == 0 && cbtns.is_empty() {
        return Ok(ArrangedButtons {
            primary: Some(ButtonProps {
                label: Cow::Borrowed("OK"),
                response: MessageBoxResponse::Ok,
            }),
            secondary: None,
            close: None,
        });
    }

    let mut primary = None;
    let mut secondary = None;
    let mut close = None;

    for meta in BUTTON_META.iter().filter(|m| btns.contains(m.flag)) {
        let props = ButtonProps {
            label: Cow::Borrowed(meta.label),
            response: meta.response,
        };
        let targets = match meta.slot {
            Slot::Primary => [&mut primary, &mut secondary, &mut close],
            Slot::Secondary => [&mut secondary, &mut primary, &mut close],
            Slot::Close => [&mut close, &mut primary, &mut secondary],
        };
        if let Some(target) = targets.into_iter().find(|t| t.is_none()) {
            *target = Some(props);
        }
    }

    for btn in cbtns {
        let props = ButtonProps {
            label: Cow::Owned(btn.text.to_string_lossy()),
            response: MessageBoxResponse::Custom(btn.result),
        };
        if primary.is_none() {
            primary = Some(props);
        } else if secondary.is_none() {
            secondary = Some(props);
        }
    }

    Ok(ArrangedButtons {
        primary,
        secondary,
        close,
    })
}

fn build_panel(instr: &str, msg: &str) -> Result<StackPanel> {
    let content = StackPanel::new()?;

    if !instr.is_empty() {
        let block = TextBlock::new()?;
        block.SetText(&HSTRING::from(instr))?;
        block.SetFontSize(14.0)?;
        block.SetFontWeight(FontWeight { Weight: 600 })?;
        block.SetMargin(Thickness {
            Left: 0.0,
            Top: 0.0,
            Right: 0.0,
            Bottom: 12.0,
        })?;
        content.Children()?.Append(&block)?;
    }

    if !msg.is_empty() {
        let block = TextBlock::new()?;
        block.SetText(&HSTRING::from(msg))?;
        block.SetTextWrapping(TextWrapping::Wrap)?;
        block.SetMargin(Thickness {
            Left: 0.0,
            Top: 0.0,
            Right: 0.0,
            Bottom: 12.0,
        })?;
        content.Children()?.Append(&block)?;
    }

    Ok(content)
}

async fn msgbox(
    xaml_root: &XamlRoot,
    msg: &str,
    title: &str,
    instr: &str,
    btns: MessageBoxButton,
    cbtns: &[CustomButton],
) -> Result<MessageBoxResponse> {
    let buttons = arrange_buttons(btns, cbtns)?;

    let dialog = ContentDialog::new()?;
    dialog.SetXamlRoot(xaml_root)?;

    let title_ref = PropertyValue::CreateString(&HSTRING::from(title))?;
    dialog.SetTitle(&title_ref)?;

    if !instr.is_empty() || !msg.is_empty() {
        dialog.SetContent(&build_panel(instr, msg)?)?;
    }

    let default_btn = buttons
        .primary
        .as_ref()
        .map_or(ContentDialogButton::None, |b| match b.response {
            MessageBoxResponse::Ok | MessageBoxResponse::Yes | MessageBoxResponse::Retry => {
                ContentDialogButton::Primary
            }
            _ => ContentDialogButton::None,
        });
    dialog.SetDefaultButton(default_btn)?;

    if let Some(ButtonProps { ref label, .. }) = buttons.primary {
        dialog.SetPrimaryButtonText(&HSTRING::from(label.as_ref()))?;
    }
    if let Some(ButtonProps { ref label, .. }) = buttons.secondary {
        dialog.SetSecondaryButtonText(&HSTRING::from(label.as_ref()))?;
    }
    if let Some(ButtonProps { ref label, .. }) = buttons.close {
        dialog.SetCloseButtonText(&HSTRING::from(label.as_ref()))?;
    }

    let show_result = dialog.ShowAsync()?.await?;

    let response = match show_result {
        ContentDialogResult::Primary => buttons.primary.map(|b| b.response),
        ContentDialogResult::Secondary => buttons.secondary.map(|b| b.response),
        _ => buttons.close.map(|b| b.response),
    };

    Ok(response.unwrap_or(MessageBoxResponse::Cancel))
}
