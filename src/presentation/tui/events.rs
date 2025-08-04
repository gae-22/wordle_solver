use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

use super::mode::InteractionMode;

/// TUI events that can occur
#[derive(Debug, Clone)]
pub enum TuiEvent {
    /// Key press event
    Key(KeyEvent),
    /// Tick event for periodic updates
    Tick,
    /// Resize event
    Resize,
    /// Application quit event
    Quit,
}

impl From<KeyEvent> for TuiEvent {
    fn from(key_event: KeyEvent) -> Self {
        TuiEvent::Key(key_event)
    }
}

/// Key input actions that can be performed
#[derive(Debug, Clone, PartialEq)]
pub enum KeyAction {
    /// Add character to input
    AddChar(char),
    /// Delete character (backspace)
    DeleteChar,
    /// Move cursor left
    MoveCursorLeft,
    /// Move cursor right
    MoveCursorRight,
    /// Submit current input
    Submit,
    /// Clear current input
    Clear,
    /// Toggle help display
    ToggleHelp,
    /// Get first guess suggestion
    GetFirstGuess,
    /// Show solver statistics
    ShowStats,
    /// Reset the game
    Reset,
    /// Quit application
    Quit,
    /// Switch to input mode
    SwitchToInputMode,
    /// Switch to operation mode
    SwitchToOperationMode,
    /// Toggle between input and operation modes
    ToggleMode,
    /// No action
    None,
}

/// Event handler for processing TUI events
pub struct EventHandler {
    tick_rate: Duration,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        Self { tick_rate }
    }

    /// Process a key event and return the corresponding action
    ///
    /// # Arguments
    /// * `key_event` - The key event to process
    /// * `current_mode` - The current interaction mode
    /// * `is_typing` - Whether the user is currently typing (has partial input)
    pub fn process_key_event(
        &self,
        key_event: KeyEvent,
        current_mode: InteractionMode,
        is_typing: bool,
    ) -> KeyAction {
        match key_event {
            // Mode switching (always available)
            KeyEvent {
                code: KeyCode::Esc, ..
            } => KeyAction::ToggleMode,

            KeyEvent {
                code: KeyCode::Tab, ..
            } => KeyAction::ToggleMode,

            // Quit commands (always prioritized)
            KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => KeyAction::Quit,

            KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => KeyAction::Quit,

            // Mode-specific handling
            _ => match current_mode {
                InteractionMode::Input => self.process_input_mode_key(key_event, is_typing),
                InteractionMode::Operation => self.process_operation_mode_key(key_event),
            },
        }
    }

    /// Process key events in input mode
    fn process_input_mode_key(&self, key_event: KeyEvent, _is_typing: bool) -> KeyAction {
        match key_event {
            // Input actions
            KeyEvent {
                code: KeyCode::Enter,
                ..
            } => KeyAction::Submit,

            KeyEvent {
                code: KeyCode::Backspace,
                ..
            } => KeyAction::DeleteChar,

            KeyEvent {
                code: KeyCode::Left,
                ..
            } => KeyAction::MoveCursorLeft,

            KeyEvent {
                code: KeyCode::Right,
                ..
            } => KeyAction::MoveCursorRight,

            KeyEvent {
                code: KeyCode::Delete,
                ..
            } => KeyAction::Clear,

            // Character input (alphabetic and digits for feedback)
            KeyEvent {
                code: KeyCode::Char(c),
                modifiers: KeyModifiers::NONE,
                ..
            } if c.is_ascii_alphabetic() || c.is_ascii_digit() => KeyAction::AddChar(c),

            // Space for submit (alternative)
            KeyEvent {
                code: KeyCode::Char(' '),
                modifiers: KeyModifiers::NONE,
                ..
            } => KeyAction::Submit,

            _ => KeyAction::None,
        }
    }

    /// Process key events in operation mode
    fn process_operation_mode_key(&self, key_event: KeyEvent) -> KeyAction {
        match key_event {
            // Help
            KeyEvent {
                code: KeyCode::Char('h'),
                modifiers: KeyModifiers::NONE,
                ..
            } => KeyAction::ToggleHelp,

            // First guess
            KeyEvent {
                code: KeyCode::Char('f'),
                modifiers: KeyModifiers::NONE,
                ..
            } => KeyAction::GetFirstGuess,

            // Statistics
            KeyEvent {
                code: KeyCode::Char('s'),
                modifiers: KeyModifiers::NONE,
                ..
            } => KeyAction::ShowStats,

            // Reset
            KeyEvent {
                code: KeyCode::Char('r'),
                modifiers: KeyModifiers::NONE,
                ..
            } => KeyAction::Reset,

            // Clear
            KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::NONE,
                ..
            } => KeyAction::Clear,

            // Quit
            KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::NONE,
                ..
            } => KeyAction::Quit,

            _ => KeyAction::None,
        }
    }

    /// Get tick rate
    pub fn tick_rate(&self) -> Duration {
        self.tick_rate
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new(Duration::from_millis(250))
    }
}

/// Event loop for handling crossterm events
pub struct EventLoop {
    handler: EventHandler,
}

impl EventLoop {
    pub fn new(handler: EventHandler) -> Self {
        Self { handler }
    }

    /// Run the event loop and return the next event
    pub async fn next_event(&self) -> Result<TuiEvent> {
        use crossterm::event::{poll, read};

        if poll(self.handler.tick_rate())? {
            match read()? {
                crossterm::event::Event::Key(key_event) => Ok(TuiEvent::Key(key_event)),
                crossterm::event::Event::Resize(_, _) => Ok(TuiEvent::Resize),
                _ => Ok(TuiEvent::Tick),
            }
        } else {
            Ok(TuiEvent::Tick)
        }
    }

    /// Process an event and return the corresponding action
    ///
    /// # Arguments
    /// * `event` - The TUI event to process
    /// * `current_mode` - The current interaction mode
    /// * `is_typing` - Whether the user is currently typing (has partial input)
    pub fn process_event(
        &self,
        event: TuiEvent,
        current_mode: InteractionMode,
        is_typing: bool,
    ) -> KeyAction {
        match event {
            TuiEvent::Key(key_event) => {
                self.handler
                    .process_key_event(key_event, current_mode, is_typing)
            }
            TuiEvent::Quit => KeyAction::Quit,
            _ => KeyAction::None,
        }
    }
}

impl Default for EventLoop {
    fn default() -> Self {
        Self::new(EventHandler::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_actions_input_mode() {
        let handler = EventHandler::default();
        let mode = InteractionMode::Input;

        // Test character input in input mode
        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert_eq!(
            handler.process_key_event(key_event, mode, true),
            KeyAction::AddChar('a')
        );

        // Test digit input in input mode (for feedback)
        let key_event = KeyEvent::new(KeyCode::Char('1'), KeyModifiers::NONE);
        assert_eq!(
            handler.process_key_event(key_event, mode, false),
            KeyAction::AddChar('1')
        );

        // Test enter key
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(
            handler.process_key_event(key_event, mode, true),
            KeyAction::Submit
        );

        assert_eq!(
            handler.process_key_event(key_event, mode, false),
            KeyAction::Submit
        );

        // Test backspace
        let key_event = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        assert_eq!(
            handler.process_key_event(key_event, mode, false),
            KeyAction::DeleteChar
        );

        // Test delete
        let key_event = KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE);
        assert_eq!(
            handler.process_key_event(key_event, mode, false),
            KeyAction::Clear
        );

        // Test arrow keys
        let key_event = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
        assert_eq!(
            handler.process_key_event(key_event, mode, false),
            KeyAction::MoveCursorLeft
        );

        let key_event = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
        assert_eq!(
            handler.process_key_event(key_event, mode, false),
            KeyAction::MoveCursorRight
        );
    }

    #[test]
    fn test_key_actions_operation_mode() {
        let handler = EventHandler::default();
        let mode = InteractionMode::Operation;

        // Test help
        let key_event = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);
        assert_eq!(
            handler.process_key_event(key_event, mode, false),
            KeyAction::ToggleHelp
        );

        // Test first guess
        let key_event = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE);
        assert_eq!(
            handler.process_key_event(key_event, mode, false),
            KeyAction::GetFirstGuess
        );

        // Test stats
        let key_event = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE);
        assert_eq!(
            handler.process_key_event(key_event, mode, false),
            KeyAction::ShowStats
        );

        // Test reset
        let key_event = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE);
        assert_eq!(
            handler.process_key_event(key_event, mode, false),
            KeyAction::Reset
        );

        // Test clear
        let key_event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE);
        assert_eq!(
            handler.process_key_event(key_event, mode, false),
            KeyAction::Clear
        );

        // Test quit
        let key_event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        assert_eq!(
            handler.process_key_event(key_event, mode, false),
            KeyAction::Quit
        );
    }

    #[test]
    fn test_mode_switching() {
        let handler = EventHandler::default();

        // Test Esc key switches mode regardless of current mode
        let key_event = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(
            handler.process_key_event(key_event, InteractionMode::Input, false),
            KeyAction::ToggleMode
        );
        assert_eq!(
            handler.process_key_event(key_event, InteractionMode::Operation, false),
            KeyAction::ToggleMode
        );

        // Test Tab key switches mode regardless of current mode
        let key_event = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        assert_eq!(
            handler.process_key_event(key_event, InteractionMode::Input, false),
            KeyAction::ToggleMode
        );
        assert_eq!(
            handler.process_key_event(key_event, InteractionMode::Operation, false),
            KeyAction::ToggleMode
        );
    }

    #[test]
    fn test_global_quit_commands() {
        let handler = EventHandler::default();

        // Test Ctrl+Q quit (global)
        let key_event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
        assert_eq!(
            handler.process_key_event(key_event, InteractionMode::Input, false),
            KeyAction::Quit
        );
        assert_eq!(
            handler.process_key_event(key_event, InteractionMode::Operation, false),
            KeyAction::Quit
        );

        // Test Ctrl+C quit (global)
        let key_event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(
            handler.process_key_event(key_event, InteractionMode::Input, false),
            KeyAction::Quit
        );
        assert_eq!(
            handler.process_key_event(key_event, InteractionMode::Operation, false),
            KeyAction::Quit
        );
    }

    #[test]
    fn test_mode_specific_behavior() {
        let handler = EventHandler::default();

        // In input mode, 'h' should be added as character
        let key_event = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);
        assert_eq!(
            handler.process_key_event(key_event, InteractionMode::Input, false),
            KeyAction::AddChar('h')
        );

        // In operation mode, 'h' should toggle help
        assert_eq!(
            handler.process_key_event(key_event, InteractionMode::Operation, false),
            KeyAction::ToggleHelp
        );
    }
}
