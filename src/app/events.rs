use crossterm::event::{Event, KeyEvent};
use std::io;

/// Event types for the application
#[derive(Debug, Clone)]
pub enum AppEvent {
    Key(KeyEvent),
    Tick,
}

/// Event handler for the application
pub struct EventHandler {
    // Future extension for event handling
}

impl EventHandler {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn next(&mut self) -> Result<AppEvent, io::Error> {
        match crossterm::event::read()? {
            Event::Key(key_event) => Ok(AppEvent::Key(key_event)),
            _ => Ok(AppEvent::Tick),
        }
    }
}
