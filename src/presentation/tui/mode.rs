//! Input mode management for TUI
//!
//! This module handles the switching between input mode and operation mode,
//! providing a clear separation of concerns for user interactions.

/// The current interaction mode of the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionMode {
    /// Input mode: User can type words and provide feedback
    Input,
    /// Operation mode: User can perform commands like reset, help, quit
    Operation,
}

impl Default for InteractionMode {
    fn default() -> Self {
        Self::Input
    }
}

impl InteractionMode {
    /// Toggle between input and operation modes
    pub fn toggle(self) -> Self {
        match self {
            Self::Input => Self::Operation,
            Self::Operation => Self::Input,
        }
    }

    /// Check if currently in input mode
    pub fn is_input(self) -> bool {
        matches!(self, Self::Input)
    }

    /// Check if currently in operation mode
    pub fn is_operation(self) -> bool {
        matches!(self, Self::Operation)
    }

    /// Get the mode name as string for display
    pub fn name(self) -> &'static str {
        match self {
            Self::Input => "INPUT",
            Self::Operation => "OPERATION",
        }
    }

    /// Get the mode description for help display
    pub fn description(self) -> &'static str {
        match self {
            Self::Input => "Type words and provide feedback",
            Self::Operation => "Perform operations (reset, help, quit, etc.)",
        }
    }

    /// Get available commands for the current mode
    pub fn available_commands(self) -> Vec<(&'static str, &'static str)> {
        match self {
            Self::Input => vec![
                ("a-z", "Type characters"),
                ("0-2", "Provide feedback (0=absent, 1=present, 2=correct)"),
                ("Enter", "Submit guess/feedback"),
                ("Backspace", "Delete character"),
                ("Delete", "Clear input"),
                ("Left/Right", "Move cursor"),
                ("Esc/Tab", "Switch to operation mode"),
            ],
            Self::Operation => vec![
                ("h", "Toggle help"),
                ("f", "Get first guess"),
                ("s", "Show statistics"),
                ("r", "Reset game"),
                ("c", "Clear input"),
                ("q", "Quit application"),
                ("Esc/Tab", "Switch to input mode"),
            ],
        }
    }
}
