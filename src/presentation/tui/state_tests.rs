#[cfg(test)]
mod tests {
    use crate::presentation::tui::{TuiState, InteractionMode};

    #[test]
    fn test_help_visibility_in_input_mode() {
        let state = TuiState::new();

        // Initially in INPUT mode, help should be hidden
        assert_eq!(state.interaction_mode(), InteractionMode::Input);
        assert!(!state.should_show_help());
        assert!(!state.show_help);
    }

    #[test]
    fn test_help_visibility_in_operation_mode() {
        let mut state = TuiState::new();

        // Switch to OPERATION mode
        state.switch_to_operation_mode();

        assert_eq!(state.interaction_mode(), InteractionMode::Operation);
        assert!(state.should_show_help());
        assert!(state.show_help);
    }

    #[test]
    fn test_mode_toggle_updates_help() {
        let mut state = TuiState::new();

        // Start in INPUT mode (help hidden)
        assert!(!state.should_show_help());

        // Toggle to OPERATION mode (help shown)
        state.toggle_interaction_mode();
        assert!(state.should_show_help());

        // Toggle back to INPUT mode (help hidden)
        state.toggle_interaction_mode();
        assert!(!state.should_show_help());
    }

    #[test]
    fn test_manual_help_toggle_in_input_mode() {
        let mut state = TuiState::new();

        // In INPUT mode, manual help toggle should work
        assert!(!state.should_show_help());

        state.toggle_help();
        assert!(state.should_show_help());

        state.toggle_help();
        assert!(!state.should_show_help());
    }

    #[test]
    fn test_operation_mode_always_shows_help() {
        let mut state = TuiState::new();
        state.switch_to_operation_mode();

        // In OPERATION mode, should_show_help() should always return true
        assert!(state.should_show_help());

        // Even if show_help is manually set to false, should_show_help() returns true
        state.show_help = false;
        assert!(state.should_show_help());
    }
}
