use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

use crate::presentation::tui::{
    components::{
        centered_rect, render_feedback_help, render_feedback_input, render_footer, render_help,
        render_history, render_input, render_logs, render_mode_indicator, render_progress,
        render_remaining_words, render_stats, render_status, render_suggestion, render_title,
    },
    feedback::FeedbackInputManager,
    state::TuiState,
};

/// Main layout manager for the TUI
pub struct LayoutManager;

impl LayoutManager {
    /// Render the main application layout
    pub fn render_main_layout(
        frame: &mut Frame,
        state: &TuiState,
        feedback_manager: &FeedbackInputManager,
    ) {
        let size = frame.size();

        // Create main vertical layout
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(3), // Mode indicator
                Constraint::Length(3), // Input or Feedback Input
                Constraint::Length(3), // Suggestion
                Constraint::Min(8),    // Main content area
                Constraint::Length(3), // Status/Progress
                Constraint::Length(1), // Footer
            ])
            .split(size);

        // Render title
        render_title(frame, main_chunks[0]);

        // Render mode indicator
        render_mode_indicator(frame, main_chunks[1], state);

        // Render input area (normal input or feedback input)
        if feedback_manager.is_in_feedback_mode() {
            if let Some(current_guess) = feedback_manager.get_current_guess() {
                render_feedback_input(
                    frame,
                    main_chunks[2],
                    current_guess,
                    feedback_manager.get_feedback_input(),
                    feedback_manager.get_feedback_cursor(),
                );
            }
        } else {
            render_input(frame, main_chunks[2], state);
        }

        // Render suggestion
        render_suggestion(frame, main_chunks[3], state);

        // Split main content area
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40), // Left panel
                Constraint::Percentage(35), // Middle panel
                Constraint::Percentage(25), // Right panel
            ])
            .split(main_chunks[4]);

        // Left panel: History
        render_history(frame, content_chunks[0], state);

        // Middle panel: Stats and remaining words OR feedback help
        if feedback_manager.is_in_feedback_mode() {
            render_feedback_help(frame, content_chunks[1]);
        } else {
            let middle_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(50), // Stats
                    Constraint::Percentage(50), // Remaining words
                ])
                .split(content_chunks[1]);

            render_stats(frame, middle_chunks[0], state);
            render_remaining_words(frame, middle_chunks[1], state);
        }

        // Right panel: Logs
        render_logs(frame, content_chunks[2], state);

        // Bottom panel: Status or Progress
        if state.status_message.is_some() {
            render_status(frame, main_chunks[5], state);
        } else {
            render_progress(frame, main_chunks[5], state);
        }

        // Footer command bar
        render_footer(frame, main_chunks[6], state);

        // Render help overlay if needed
        if state.should_show_help() {
            let help_area = centered_rect(80, 70, size);
            render_help(frame, help_area, state);
        }
    }

    /// Render a compact layout for smaller terminals
    pub fn render_compact_layout(
        frame: &mut Frame,
        state: &TuiState,
        feedback_manager: &FeedbackInputManager,
    ) {
        let size = frame.size();

        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(3), // Input + Suggestion combined
                Constraint::Min(6),    // Content
                Constraint::Length(2), // Status
                Constraint::Length(1), // Footer
            ])
            .split(size);

        // Render title
        render_title(frame, main_chunks[0]);

        // Split input line
        let input_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50), // Input
                Constraint::Percentage(50), // Suggestion
            ])
            .split(main_chunks[1]);

        // Render input area (normal input or feedback input)
        if feedback_manager.is_in_feedback_mode() {
            if let Some(current_guess) = feedback_manager.get_current_guess() {
                render_feedback_input(
                    frame,
                    input_chunks[0],
                    current_guess,
                    feedback_manager.get_feedback_input(),
                    feedback_manager.get_feedback_cursor(),
                );
            }
        } else {
            render_input(frame, input_chunks[0], state);
        }

        render_suggestion(frame, input_chunks[1], state);

        // Split content area
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60), // History
                Constraint::Percentage(40), // Stats or feedback help
            ])
            .split(main_chunks[2]);

        render_history(frame, content_chunks[0], state);

        if feedback_manager.is_in_feedback_mode() {
            render_feedback_help(frame, content_chunks[1]);
        } else {
            render_stats(frame, content_chunks[1], state);
        }

        // Status
        if state.status_message.is_some() {
            render_status(frame, main_chunks[3], state);
        } else {
            render_progress(frame, main_chunks[3], state);
        }

        // Footer
        render_footer(frame, main_chunks[4], state);

        // Help overlay
        if state.should_show_help() {
            let help_area = centered_rect(90, 80, size);
            render_help(frame, help_area, state);
        }
    }

    /// Render a mobile-friendly layout for very small terminals
    pub fn render_mobile_layout(
        frame: &mut Frame,
        state: &TuiState,
        feedback_manager: &FeedbackInputManager,
    ) {
        let size = frame.size();

        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Title (compact)
                Constraint::Length(3), // Input
                Constraint::Min(4),    // Content (scrollable)
                Constraint::Length(1), // Status
                Constraint::Length(1), // Footer
            ])
            .split(size);

        // Compact title
        render_title(frame, main_chunks[0]);

        // Input
        if feedback_manager.is_in_feedback_mode() {
            if let Some(current_guess) = feedback_manager.get_current_guess() {
                render_feedback_input(
                    frame,
                    main_chunks[1],
                    current_guess,
                    feedback_manager.get_feedback_input(),
                    feedback_manager.get_feedback_cursor(),
                );
            }
        } else {
            render_input(frame, main_chunks[1], state);
        }

        // Just show the most important content
        if feedback_manager.is_in_feedback_mode() {
            render_feedback_help(frame, main_chunks[2]);
        } else if !state.guess_history.is_empty() {
            render_history(frame, main_chunks[2], state);
        } else {
            render_stats(frame, main_chunks[2], state);
        }

        // Simple status
        if state.status_message.is_some() {
            render_status(frame, main_chunks[3], state);
        }

        // Footer (minimal)
        render_footer(frame, main_chunks[4], state);

        // Full-screen help
        if state.should_show_help() {
            render_help(frame, size, state);
        }
    }

    /// Determine which layout to use based on terminal size
    pub fn render_adaptive_layout(
        frame: &mut Frame,
        state: &TuiState,
        feedback_manager: &FeedbackInputManager,
    ) {
        let size = frame.size();

        match (size.width, size.height) {
            // Very small terminal (mobile-like)
            (w, h) if w < 80 || h < 20 => {
                Self::render_mobile_layout(frame, state, feedback_manager)
            }
            // Small terminal
            (w, h) if w < 120 || h < 30 => {
                Self::render_compact_layout(frame, state, feedback_manager)
            }
            // Full-size terminal
            _ => Self::render_main_layout(frame, state, feedback_manager),
        }
    }

    /// Get minimum required terminal size
    pub fn min_size() -> (u16, u16) {
        (40, 10)
    }

    /// Check if terminal size is adequate
    pub fn is_size_adequate(width: u16, height: u16) -> bool {
        let (min_width, min_height) = Self::min_size();
        width >= min_width && height >= min_height
    }

    /// Get recommended terminal size
    pub fn recommended_size() -> (u16, u16) {
        (120, 35)
    }
}

/// Layout presets for different screen configurations
pub enum LayoutPreset {
    /// Full desktop layout with all panels
    Desktop,
    /// Laptop layout with reduced panels
    Laptop,
    /// Tablet layout with stacked panels
    Tablet,
    /// Mobile layout with minimal UI
    Mobile,
}

impl LayoutPreset {
    /// Get the appropriate preset based on terminal size
    pub fn from_size(width: u16, height: u16) -> Self {
        match (width, height) {
            (w, h) if w >= 140 && h >= 40 => LayoutPreset::Desktop,
            (w, h) if w >= 100 && h >= 25 => LayoutPreset::Laptop,
            (w, h) if w >= 80 && h >= 20 => LayoutPreset::Tablet,
            _ => LayoutPreset::Mobile,
        }
    }

    /// Render the layout using this preset
    pub fn render(
        &self,
        frame: &mut Frame,
        state: &TuiState,
        feedback_manager: &FeedbackInputManager,
    ) {
        match self {
            LayoutPreset::Desktop => {
                LayoutManager::render_main_layout(frame, state, feedback_manager)
            }
            LayoutPreset::Laptop => {
                LayoutManager::render_main_layout(frame, state, feedback_manager)
            }
            LayoutPreset::Tablet => {
                LayoutManager::render_compact_layout(frame, state, feedback_manager)
            }
            LayoutPreset::Mobile => {
                LayoutManager::render_mobile_layout(frame, state, feedback_manager)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_adequacy() {
        assert!(LayoutManager::is_size_adequate(80, 24));
        assert!(!LayoutManager::is_size_adequate(30, 10));
        assert!(!LayoutManager::is_size_adequate(80, 5));
    }

    #[test]
    fn test_layout_preset_selection() {
        assert!(matches!(
            LayoutPreset::from_size(150, 50),
            LayoutPreset::Desktop
        ));
        assert!(matches!(
            LayoutPreset::from_size(100, 30),
            LayoutPreset::Laptop
        ));
        assert!(matches!(
            LayoutPreset::from_size(80, 20),
            LayoutPreset::Tablet
        ));
        assert!(matches!(
            LayoutPreset::from_size(40, 15),
            LayoutPreset::Mobile
        ));
    }
}
