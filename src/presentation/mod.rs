//! Presentation layer for user interfaces
//!
//! This module contains all user interface implementations including
//! TUI (Terminal User Interface) and potential future GUI implementations.

pub mod tui;

// Re-export TUI components for easy access
pub use tui::{run_tui, TuiApp};
