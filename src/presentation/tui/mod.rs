//! TUI (Terminal User Interface) presentation layer
//!
//! This module provides a modern, responsive terminal interface for the Wordle solver
//! using ratatui and crossterm for cross-platform compatibility.

pub mod app;
pub mod components;
pub mod events;
pub mod feedback;
pub mod layout;
pub mod mode;
pub mod state;

#[cfg(test)]
mod state_tests;

// Re-export main components
pub use app::{TuiApp, run_tui};
pub use events::{EventHandler, EventLoop, KeyAction, TuiEvent};
pub use feedback::FeedbackInputManager;
pub use layout::{LayoutManager, LayoutPreset};
pub use mode::InteractionMode;
pub use state::{
    GameStats, GuessHistoryEntry, LogLevel, LogMessage, MessageType, StatusMessage, TuiState,
};
