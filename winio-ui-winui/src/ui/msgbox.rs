use std::{cell::RefCell, rc::Rc};

use send_wrapper::SendWrapper;
use windows::{
    Foundation::PropertyValue,
    UI::Text::FontWeight,
    Win32::Foundation::E_POINTER,
    core::{HSTRING, Interface, h},
};
use windows_sys::Win32::Foundation::HWND;
use winio_handle::AsWindow;
use winio_primitive::{MessageBoxButton, MessageBoxResponse, MessageBoxStyle};
use winui3::Microsoft::UI::{
    WindowId,
    Windowing::{AppWindow, OverlappedPresenter},
    Xaml::{
        Application,
        Controls::{
            BackgroundSizing, Button, ColumnDefinition, ContentDialog, ContentDialogButton, Grid,
            RowDefinition, StackPanel, TextBlock,
        },
        GridLength, GridUnitType, HorizontalAlignment,
        Media::Brush,
        RoutedEventHandler, Style, TextWrapping, Thickness, XamlRoot,
    },
};

use crate::{Error, ROOT_WINDOWS, Result};

struct ButtonMeta {
    flag: MessageBoxButton,
    label: &'static HSTRING,
    response: MessageBoxResponse,
}

const BUTTON_META: [ButtonMeta; 6] = [
    ButtonMeta {
        flag: MessageBoxButton::Ok,
        label: h!("OK"),
        response: MessageBoxResponse::Ok,
    },
    ButtonMeta {
        flag: MessageBoxButton::Yes,
        label: h!("Yes"),
        response: MessageBoxResponse::Yes,
    },
    ButtonMeta {
        flag: MessageBoxButton::No,
        label: h!("No"),
        response: MessageBoxResponse::No,
    },
    ButtonMeta {
        flag: MessageBoxButton::Cancel,
        label: h!("Cancel"),
        response: MessageBoxResponse::Cancel,
    },
    ButtonMeta {
        flag: MessageBoxButton::Retry,
        label: h!("Retry"),
        response: MessageBoxResponse::Retry,
    },
    ButtonMeta {
        flag: MessageBoxButton::Close,
        label: h!("Close"),
        response: MessageBoxResponse::Close,
    },
];

#[derive(Debug, Clone, Default)]
pub struct MessageBox {
    msg: HSTRING,
    title: HSTRING,
    instr: HSTRING,
    btns: MessageBoxButton,
    cbtns: Vec<CustomButton>,
}

impl MessageBox {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn show(
        self,
        parent: Option<impl AsWindow>,
    ) -> Result<impl Future<Output = Result<MessageBoxResponse>> + 'static> {
        let (hwnd, xaml_root) = if let Some(parent) = parent {
            let window = parent.as_window();
            let hwnd = window.handle()?;
            (hwnd, window.as_winui().Content()?.XamlRoot()?)
        } else {
            let xaml_root = ROOT_WINDOWS
                .with_borrow(|windows| windows.first().cloned())
                .ok_or_else(|| Error::from_hresult(E_POINTER))?
                .Content()?
                .XamlRoot()?;
            (std::ptr::null_mut(), xaml_root)
        };

        msgbox(
            hwnd, xaml_root, self.msg, self.title, self.instr, self.btns, self.cbtns,
        )
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
    mut btns: MessageBoxButton,
    cbtns: &[CustomButton],
) -> Vec<(&HSTRING, MessageBoxResponse)> {
    if cbtns.is_empty() && btns.is_empty() {
        btns = MessageBoxButton::Ok;
    }

    let n = BUTTON_META.iter().filter(|m| btns.contains(m.flag)).count();
    let mut out = Vec::with_capacity(n + cbtns.len());
    out.extend(
        cbtns
            .iter()
            .map(|btn| (&btn.text, MessageBoxResponse::Custom(btn.result))),
    );

    out.extend(
        BUTTON_META
            .iter()
            .filter(|m| btns.contains(m.flag))
            .map(|m| (m.label, m.response)),
    );

    out
}

fn lookup<T: Interface>(key: &HSTRING) -> Result<T> {
    let resources = Application::Current()?.Resources()?;
    let key_obj = PropertyValue::CreateString(key)?;
    resources.Lookup(&key_obj)?.cast()
}

fn build_button_grid(
    buttons: &[(&HSTRING, MessageBoxResponse)],
    dialog: &ContentDialog,
    result: &SendWrapper<Rc<RefCell<Option<MessageBoxResponse>>>>,
) -> Result<Grid> {
    let grid = Grid::new()?;
    grid.SetColumnSpacing(8.0)?;
    let cols = grid.ColumnDefinitions()?;
    let children = grid.Children()?;
    let n = buttons.len();
    let mut accent_style = (!buttons.is_empty())
        .then(|| lookup::<Style>(h!("AccentButtonStyle")).ok())
        .flatten();

    for _ in 0..n.max(2) {
        let btn_col = ColumnDefinition::new()?;
        btn_col.SetWidth(GridLength {
            Value: 1.0,
            GridUnitType: GridUnitType::Star,
        })?;
        cols.Append(&btn_col)?;
    }

    for (i, (label, response)) in buttons.iter().enumerate() {
        let col = if n == 1 { 1 } else { i as i32 };
        let btn = Button::new()?;
        let tb = TextBlock::new()?;
        tb.SetText(label)?;
        btn.SetContent(&tb)?;
        btn.SetHorizontalAlignment(HorizontalAlignment::Stretch)?;
        Grid::SetColumn(&btn, col)?;

        if matches!(response, MessageBoxResponse::Ok | MessageBoxResponse::Yes)
            && let Some(style) = accent_style.take()
        {
            btn.SetStyle(&style)?;
        }

        let result = result.clone();
        let dialog = dialog.clone();
        let resp = *response;
        btn.Click(&RoutedEventHandler::new(move |_, _| {
            *result.borrow_mut() = Some(resp);
            dialog.Hide()?;
            Ok(())
        }))?;

        children.Append(&btn)?;
    }

    Ok(grid)
}

fn build_content(
    instr: &HSTRING,
    msg: &HSTRING,
    buttons: &[(&HSTRING, MessageBoxResponse)],
    dialog: &ContentDialog,
    result: &SendWrapper<Rc<RefCell<Option<MessageBoxResponse>>>>,
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
        block.SetText(instr)?;
        block.SetFontSize(14.0)?;
        block.SetFontWeight(FontWeight { Weight: 600 })?;
        text_children.Append(&block)?;
    }

    if !msg.is_empty() {
        let block = TextBlock::new()?;
        block.SetText(msg)?;
        block.SetTextWrapping(TextWrapping::Wrap)?;
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

        bar.SetBackground(&lookup::<Brush>(h!("SolidBackgroundFillColorBaseBrush"))?)?;

        bar.SetBorderBrush(&lookup::<Brush>(h!("CardStrokeColorDefaultBrush"))?)?;
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

fn msgbox(
    hwnd: HWND,
    xaml_root: XamlRoot,
    msg: HSTRING,
    title: HSTRING,
    instr: HSTRING,
    btns: MessageBoxButton,
    cbtns: Vec<CustomButton>,
) -> Result<impl Future<Output = Result<MessageBoxResponse>> + 'static> {
    let all_buttons = collect_buttons(btns, &cbtns);

    let dialog = ContentDialog::new()?;
    dialog.SetXamlRoot(&xaml_root)?;
    dialog.SetTitle(&PropertyValue::CreateString(&title)?)?;
    dialog.SetDefaultButton(ContentDialogButton::None)?;

    let result = SendWrapper::new(Rc::new(RefCell::new(None)));
    let content = build_content(&instr, &msg, &all_buttons, &dialog, &result)?;
    dialog.SetContent(&content)?;

    struct EnableGuard {
        presenter: OverlappedPresenter,
        restored: bool,
    }

    impl EnableGuard {
        fn new(presenter: OverlappedPresenter) -> Self {
            Self {
                presenter,
                restored: false,
            }
        }

        fn restore_impl(&mut self) -> Result<()> {
            self.presenter.SetIsMaximizable(true)?;
            self.presenter.SetIsResizable(true)?;
            self.restored = true;
            Ok(())
        }

        fn restore(mut self) -> Result<()> {
            self.restore_impl()
        }
    }

    impl Drop for EnableGuard {
        fn drop(&mut self) {
            if !self.restored {
                self.restore_impl().ok();
            }
        }
    }

    let guard = if !hwnd.is_null() {
        let window = AppWindow::GetFromWindowId(WindowId { Value: hwnd as _ })?;
        let presenter = window.Presenter()?.cast::<OverlappedPresenter>()?;
        presenter.SetIsMaximizable(false)?;
        presenter.SetIsResizable(false)?;
        Some(EnableGuard::new(presenter))
    } else {
        None
    };

    let action = dialog.ShowAsync()?;

    Ok(async move {
        action.await?;

        if let Some(guard) = guard {
            guard.restore()?;
        }

        Ok(result
            .borrow_mut()
            .take()
            .unwrap_or(MessageBoxResponse::Cancel))
    })
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
