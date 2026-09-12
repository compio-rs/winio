use std::{cell::RefCell, collections::VecDeque};

use objc2_foundation::NSString;
use winio_callback::Callback;
use winio_pollable::GlobalRuntime;
use winio_primitive::KeyCode;

use crate::from_nsstring;

fn is_function_key(c: char) -> bool {
    ('\u{f700}'..='\u{f8ff}').contains(&c)
}

/// Convert a single character key symbol to its uppercase key code.
///
/// Empty or multi-character strings, control characters, Apple function-key
/// symbols and uppercase mappings containing multiple characters are
/// unidentified.
pub fn character_key(text: &str) -> KeyCode {
    let mut chars = text.chars();
    let Some(c) = chars.next() else {
        return KeyCode::Unidentified;
    };
    if chars.next().is_some() {
        return KeyCode::Unidentified;
    }
    let mut upper = c.to_uppercase();
    let c = upper.next().unwrap();
    if c.is_control() || is_function_key(c) || upper.next().is_some() {
        return KeyCode::Unidentified;
    }
    KeyCode::Char(c)
}

/// Buffers native keyboard text and yields one Unicode character at a time.
#[derive(Debug, Default)]
pub struct KeyCharCallback {
    pending: RefCell<VecDeque<char>>,
    ready: Callback,
}

impl KeyCharCallback {
    /// Enqueue keyboard text, filtering out Apple's function-key symbols.
    pub fn signal(&self, text: &NSString) {
        let text = from_nsstring(text);
        let mut chars = text.chars().filter(|c| !is_function_key(*c)).peekable();
        if chars.peek().is_some() {
            self.pending.borrow_mut().extend(chars);
            self.ready.signal::<GlobalRuntime>(());
        }
    }

    /// Wait for the next character. Cancelling a wait preserves queued text.
    pub async fn wait(&self) -> char {
        loop {
            if let Some(c) = self.pending.borrow_mut().pop_front() {
                return c;
            }
            self.ready.wait().await;
        }
    }
}
