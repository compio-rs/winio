use std::sync::{Arc, Mutex};

use windows::Foundation::PropertyValue;
use windows::core::{HSTRING, Interface};
use winio_handle::AsWindow;
use winio_primitive::{MessageBoxButton, MessageBoxResponse, MessageBoxStyle};
use winio_ui_windows_common::CustomButton;
use winui3::Microsoft::UI::Xaml::Style;
use winui3::Microsoft::UI::Xaml::{
    Application,
    Controls::{
        BackgroundSizing, Button as WinUIButton, ColumnDefinition, ContentDialog,
        ContentDialogButton, Grid, RowDefinition, StackPanel, TextBlock,
    },
    GridLength, GridUnitType, HorizontalAlignment,
    Media::Brush,
    RoutedEventHandler, Thickness, XamlRoot,
};

use crate::{Error, Result};

struct ButtonMeta {
    flag: MessageBoxButton,
    label: &'static str,
    response: MessageBoxResponse,
}

const BUTTON_META: [ButtonMeta; 6] = [
    ButtonMeta {
        flag: MessageBoxButton::Ok,
        label: "OK",
        response: MessageBoxResponse::Ok,
    },
    ButtonMeta {
        flag: MessageBoxButton::Yes,
        label: "Yes",
        response: MessageBoxResponse::Yes,
    },
    ButtonMeta {
        flag: MessageBoxButton::No,
        label: "No",
        response: MessageBoxResponse::No,
    },
    ButtonMeta {
        flag: MessageBoxButton::Cancel,
        label: "Cancel",
        response: MessageBoxResponse::Cancel,
    },
    ButtonMeta {
        flag: MessageBoxButton::Retry,
        label: "Retry",
        response: MessageBoxResponse::Retry,
    },
    ButtonMeta {
        flag: MessageBoxButton::Close,
        label: "Close",
        response: MessageBoxResponse::Close,
    },
];

#[derive(Debug, Clone, Default)]
pub struct MessageBox {
    msg: String,
    title: String,
    instr: String,
    btns: MessageBoxButton,
    cbtns: Vec<CustomButton>,
}

impl MessageBox {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn show(self, parent: Option<impl AsWindow>) -> Result<MessageBoxResponse> {
        let xaml_root = parent
            .ok_or_else(|| Error::from_hresult(windows::Win32::Foundation::E_INVALIDARG))?
            .as_window()
            .as_winui()
            .Content()?
            .XamlRoot()?;

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
        self.msg = msg.to_owned();
    }

    pub fn title(&mut self, title: &str) {
        self.title = title.to_owned();
    }

    pub fn instruction(&mut self, instr: &str) {
        self.instr = instr.to_owned();
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

fn collect_buttons(
    btns: MessageBoxButton,
    cbtns: &[CustomButton],
) -> Vec<(HSTRING, MessageBoxResponse)> {
    let n = BUTTON_META.iter().filter(|m| btns.contains(m.flag)).count();
    let mut out = Vec::with_capacity(n + cbtns.len());
    out.extend(cbtns.iter().map(|btn| {
        (
            HSTRING::from_wide(btn.text.as_slice_with_nul()),
            MessageBoxResponse::Custom(btn.result),
        )
    }));

    out.extend(
        BUTTON_META
            .iter()
            .filter(|m| btns.contains(m.flag))
            .map(|m| (HSTRING::from(m.label), m.response)),
    );

    if out.is_empty() {
        out.push((HSTRING::from("OK"), MessageBoxResponse::Ok));
    }

    out
}

fn lookup<T: Interface>(key: &str) -> Result<T> {
    let resources = Application::Current()?.Resources()?;
    let key_obj = PropertyValue::CreateString(&HSTRING::from(key))?;
    Ok(resources.Lookup(&key_obj)?.cast()?)
}

fn build_button_grid(
    buttons: &[(HSTRING, MessageBoxResponse)],
    dialog: &ContentDialog,
    result: &Arc<Mutex<Option<MessageBoxResponse>>>,
) -> Result<Grid> {
    let grid = Grid::new()?;
    grid.SetColumnSpacing(8.0)?;
    let cols = grid.ColumnDefinitions()?;
    let children = grid.Children()?;
    let n = buttons.len();
    let accent_style = (!buttons.is_empty())
        .then(|| lookup::<Style>("AccentButtonStyle").ok())
        .flatten();

    for _ in 0..if n == 1 { 2 } else { n } {
        let btn_col = ColumnDefinition::new()?;
        btn_col.SetWidth(GridLength {
            Value: 1.0,
            GridUnitType: GridUnitType::Star,
        })?;
        cols.Append(&btn_col)?;
    }

    for (i, (label, response)) in buttons.iter().enumerate() {
        let col = if n == 1 { 1 } else { i as i32 };
        let btn = WinUIButton::new()?;
        let tb = TextBlock::new()?;
        tb.SetText(label)?;
        btn.SetContent(&tb)?;
        btn.SetHorizontalAlignment(HorizontalAlignment::Stretch)?;
        Grid::SetColumn(&btn, col)?;

        if i == 0
            && response != &MessageBoxResponse::Cancel
            && response != &MessageBoxResponse::Close
            && let Some(ref style) = accent_style
        {
            btn.SetStyle(style)?;
        }

        let result = Arc::clone(result);
        let dialog = dialog.clone();
        let resp = *response;
        btn.Click(&RoutedEventHandler::new(move |_, _| {
            if let Ok(mut r) = result.lock() {
                *r = Some(resp);
            }
            dialog.Hide()?;
            Ok(())
        }))?;

        children.Append(&btn)?;
    }

    Ok(grid)
}

fn build_content(
    instr: &str,
    msg: &str,
    buttons: &[(HSTRING, MessageBoxResponse)],
    dialog: &ContentDialog,
    result: &Arc<Mutex<Option<MessageBoxResponse>>>,
) -> Result<Grid> {
    let content = Grid::new()?;
    let content_rows = content.RowDefinitions()?;
    let content_children = content.Children()?;

    let row0 = RowDefinition::new()?;
    row0.SetHeight(GridLength {
        Value: 1.0,
        GridUnitType: GridUnitType::Star,
    })?;
    content_rows.Append(&row0)?;

    let text_panel = StackPanel::new()?;
    text_panel.SetPadding(Thickness {
        Left: 0.0,
        Top: 0.0,
        Right: 0.0,
        Bottom: 24.0,
    })?;
    let text_children = text_panel.Children()?;

    if !instr.is_empty() {
        let block = TextBlock::new()?;
        block.SetText(&HSTRING::from(instr))?;
        block.SetFontSize(14.0)?;
        block.SetFontWeight(windows::UI::Text::FontWeight { Weight: 600 })?;
        text_children.Append(&block)?;
    }

    if !msg.is_empty() {
        let block = TextBlock::new()?;
        block.SetText(&HSTRING::from(msg))?;
        block.SetTextWrapping(winui3::Microsoft::UI::Xaml::TextWrapping::Wrap)?;
        text_children.Append(&block)?;
    }

    Grid::SetRow(&text_panel, 0)?;
    content_children.Append(&text_panel)?;

    if !buttons.is_empty() {
        let row1 = RowDefinition::new()?;
        row1.SetHeight(GridLength {
            Value: 0.0,
            GridUnitType: GridUnitType::Auto,
        })?;
        content_rows.Append(&row1)?;

        let bar = Grid::new()?;
        bar.SetMargin(Thickness {
            Left: -24.0,
            Top: 0.0,
            Right: -24.0,
            Bottom: -24.0,
        })?;
        bar.SetBackgroundSizing(BackgroundSizing::OuterBorderEdge)?;

        bar.SetBackground(&lookup::<Brush>("SolidBackgroundFillColorBaseBrush")?)?;

        bar.SetBorderBrush(&lookup::<Brush>("CardStrokeColorDefaultBrush")?)?;
        bar.SetBorderThickness(Thickness {
            Left: 0.0,
            Top: 1.0,
            Right: 0.0,
            Bottom: 0.0,
        })?;

        let btn_grid = build_button_grid(buttons, dialog, result)?;
        btn_grid.SetMargin(Thickness {
            Left: 24.0,
            Top: 24.0,
            Right: 24.0,
            Bottom: 24.0,
        })?;
        bar.Children()?.Append(&btn_grid)?;

        Grid::SetRow(&bar, 1)?;
        content_children.Append(&bar)?;
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
    let all_buttons = collect_buttons(btns, cbtns);

    let dialog = ContentDialog::new()?;
    dialog.SetXamlRoot(xaml_root)?;
    dialog.SetTitle(&PropertyValue::CreateString(&HSTRING::from(title))?)?;
    dialog.SetDefaultButton(ContentDialogButton::None)?;

    let result = Arc::new(Mutex::new(None::<MessageBoxResponse>));
    let content = build_content(instr, msg, &all_buttons, &dialog, &result)?;
    dialog.SetContent(&content)?;

    dialog.ShowAsync()?.await?;

    Ok(result
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .take()
        .unwrap_or(MessageBoxResponse::Cancel))
}
