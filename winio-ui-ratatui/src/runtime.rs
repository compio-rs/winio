use std::{cell::RefCell, future::Future, task::Waker};

use super::Result;

pub struct App {
    terminal: RefCell<ratatui::DefaultTerminal>,
}

impl App {
    pub fn new() -> Result<Self> {
        let terminal = ratatui::try_init()?;
        Ok(Self {
            terminal: RefCell::new(terminal),
        })
    }

    pub fn set_app_id(&mut self, _app_id: &str) -> Result<()> {
        Ok(())
    }

    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        winio_pollable::block_on(future, Waker::noop().clone(), || {
            self.terminal
                .borrow_mut()
                .draw(|f| todo!())
                .expect("failed to draw frame");
            let event = crossterm::event::read().expect("failed to read event");
        })
    }
}

impl Drop for App {
    fn drop(&mut self) {
        ratatui::restore();
    }
}
