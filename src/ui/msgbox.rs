use std::{
    borrow::Cow,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign},
};

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum MessageBoxStyle {
    #[default]
    None,
    Info,
    Warning,
    Error,
}

#[repr(i32)]
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum MessageBoxButton {
    #[default]
    Ok     = 1 << 0,
    Yes    = 1 << 1,
    No     = 1 << 2,
    Cancel = 1 << 3,
    Retry  = 1 << 4,
    Close  = 1 << 5,
}

impl MessageBoxButton {
    pub fn contains(&self, v: Self) -> bool {
        *self & v == v
    }
}

impl BitOr for MessageBoxButton {
    type Output = MessageBoxButton;

    fn bitor(self, rhs: Self) -> Self::Output {
        unsafe { std::mem::transmute(self as i32 | rhs as i32) }
    }
}

impl BitOrAssign for MessageBoxButton {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

impl BitAnd for MessageBoxButton {
    type Output = MessageBoxButton;

    fn bitand(self, rhs: Self) -> Self::Output {
        unsafe { std::mem::transmute(self as i32 & rhs as i32) }
    }
}

impl BitAndAssign for MessageBoxButton {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MessageBoxResponse {
    Cancel,
    No,
    Ok,
    Retry,
    Yes,
    Close,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CustomButton {
    pub result: i32,
    pub text: Cow<'static, str>,
}

impl CustomButton {
    pub fn new<S: Into<Cow<'static, str>>>(result: i32, text: S) -> Self {
        Self {
            result,
            text: text.into(),
        }
    }
}
