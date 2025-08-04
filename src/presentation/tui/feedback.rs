use anyhow::Result;
use std::collections::VecDeque;

use super::state::{MessageType, TuiState};

/// Manages the feedback input workflow
pub struct FeedbackInputManager {
    /// Current guess waiting for feedback
    current_guess: Option<String>,
    /// Feedback input buffer
    feedback_input: String,
    /// Cursor position in feedback input
    feedback_cursor: usize,
    /// Whether we're in feedback input mode
    is_feedback_mode: bool,
    /// Queue of pending guesses
    pending_guesses: VecDeque<String>,
}

impl Default for FeedbackInputManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FeedbackInputManager {
    pub fn new() -> Self {
        Self {
            current_guess: None,
            feedback_input: String::new(),
            feedback_cursor: 0,
            is_feedback_mode: false,
            pending_guesses: VecDeque::new(),
        }
    }

    /// Start feedback input for a guess
    pub fn start_feedback_input(&mut self, guess: String, state: &mut TuiState) {
        self.current_guess = Some(guess.clone());
        self.feedback_input.clear();
        self.feedback_cursor = 0;
        self.is_feedback_mode = true;

        state.set_status(
            format!("Enter feedback for '{}' (5 characters: 0=gray, 1=yellow, 2=green):", guess.to_uppercase()),
            MessageType::Info,
        );
    }

    /// Add a character to feedback input
    pub fn add_feedback_char(&mut self, c: char, state: &mut TuiState) -> bool {
        if !self.is_feedback_mode {
            return false;
        }

        if self.feedback_input.len() < 5 && matches!(c, '0' | '1' | '2') {
            self.feedback_input.insert(self.feedback_cursor, c);
            self.feedback_cursor += 1;

            // Update status with current input
            if let Some(ref guess) = self.current_guess {
                let display_input = format!("{}{}",
                    self.feedback_input,
                    "_".repeat(5 - self.feedback_input.len())
                );
                state.set_status(
                    format!("'{}' feedback: {} (0=gray, 1=yellow, 2=green)",
                        guess.to_uppercase(),
                        display_input
                    ),
                    MessageType::Info,
                );
            }
            true
        } else {
            state.set_status(
                "Only use 0 (gray), 1 (yellow), or 2 (green) for feedback".to_string(),
                MessageType::Warning,
            );
            false
        }
    }

    /// Delete a character from feedback input
    pub fn delete_feedback_char(&mut self, state: &mut TuiState) -> bool {
        if !self.is_feedback_mode {
            return false;
        }

        if self.feedback_cursor > 0 {
            self.feedback_cursor -= 1;
            self.feedback_input.remove(self.feedback_cursor);

            // Update status
            if let Some(ref guess) = self.current_guess {
                let display_input = format!("{}{}",
                    self.feedback_input,
                    "_".repeat(5 - self.feedback_input.len())
                );
                state.set_status(
                    format!("'{}' feedback: {} (0=gray, 1=yellow, 2=green)",
                        guess.to_uppercase(),
                        display_input
                    ),
                    MessageType::Info,
                );
            }
            true
        } else {
            false
        }
    }

    /// Submit feedback
    pub fn submit_feedback(&mut self, state: &mut TuiState) -> Option<(String, String)> {
        if !self.is_feedback_mode || self.feedback_input.len() != 5 {
            state.set_status(
                "Please enter exactly 5 feedback characters".to_string(),
                MessageType::Warning,
            );
            return None;
        }

        if let Some(guess) = self.current_guess.take() {
            let feedback = self.feedback_input.clone();

            // Reset feedback input state
            self.feedback_input.clear();
            self.feedback_cursor = 0;
            self.is_feedback_mode = false;

            state.set_status(
                format!("Processing: {} -> {}", guess.to_uppercase(), feedback),
                MessageType::Info,
            );

            Some((guess, feedback))
        } else {
            None
        }
    }

    /// Cancel feedback input
    pub fn cancel_feedback(&mut self, state: &mut TuiState) {
        self.current_guess = None;
        self.feedback_input.clear();
        self.feedback_cursor = 0;
        self.is_feedback_mode = false;

        state.set_status("Feedback input cancelled".to_string(), MessageType::Info);
    }

    /// Check if we're currently in feedback mode
    pub fn is_in_feedback_mode(&self) -> bool {
        self.is_feedback_mode
    }

    /// Get current feedback input
    pub fn get_feedback_input(&self) -> &str {
        &self.feedback_input
    }

    /// Get current guess
    pub fn get_current_guess(&self) -> Option<&str> {
        self.current_guess.as_deref()
    }

    /// Move cursor left in feedback input
    pub fn move_feedback_cursor_left(&mut self) {
        if self.feedback_cursor > 0 {
            self.feedback_cursor -= 1;
        }
    }

    /// Move cursor right in feedback input
    pub fn move_feedback_cursor_right(&mut self) {
        if self.feedback_cursor < self.feedback_input.len() {
            self.feedback_cursor += 1;
        }
    }

    /// Get feedback cursor position
    pub fn get_feedback_cursor(&self) -> usize {
        self.feedback_cursor
    }

    /// Queue a guess for feedback
    pub fn queue_guess(&mut self, guess: String) {
        self.pending_guesses.push_back(guess);
    }

    /// Process next queued guess
    pub fn process_next_queued_guess(&mut self, state: &mut TuiState) -> bool {
        if let Some(guess) = self.pending_guesses.pop_front() {
            self.start_feedback_input(guess, state);
            true
        } else {
            false
        }
    }

    /// Check if there are pending guesses
    pub fn has_pending_guesses(&self) -> bool {
        !self.pending_guesses.is_empty()
    }

    /// Clear all pending guesses
    pub fn clear_pending_guesses(&mut self) {
        self.pending_guesses.clear();
    }

    /// Validate feedback string
    pub fn validate_feedback(feedback: &str) -> Result<bool> {
        if feedback.len() != 5 {
            return Ok(false);
        }

        for c in feedback.chars() {
            if !matches!(c, '0' | '1' | '2') {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Get feedback input display string for UI
    pub fn get_feedback_display(&self) -> String {
        if !self.is_feedback_mode {
            return String::new();
        }

        let mut display = self.feedback_input.clone();

        // Add cursor indicator
        if self.feedback_cursor <= display.len() {
            display.insert(self.feedback_cursor, '█');
        }

        // Pad with underscores
        while display.len() < 6 { // 5 + cursor
            display.push('_');
        }

        display
    }

    /// Auto-complete common feedback patterns
    pub fn auto_complete_feedback(&mut self, pattern: &str, state: &mut TuiState) -> bool {
        if !self.is_feedback_mode {
            return false;
        }

        match pattern {
            "all_gray" | "none" => {
                self.feedback_input = "00000".to_string();
                self.feedback_cursor = 5;
            }
            "all_green" | "correct" => {
                self.feedback_input = "22222".to_string();
                self.feedback_cursor = 5;
            }
            _ => return false,
        }

        if let Some(ref guess) = self.current_guess {
            state.set_status(
                format!("'{}' feedback: {} (auto-completed)",
                    guess.to_uppercase(),
                    self.feedback_input
                ),
                MessageType::Success,
            );
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feedback_input() {
        let mut manager = FeedbackInputManager::new();
        let mut state = TuiState::new();

        // Start feedback input
        manager.start_feedback_input("hello".to_string(), &mut state);
        assert!(manager.is_in_feedback_mode());
        assert_eq!(manager.get_current_guess(), Some("hello"));

        // Add characters
        assert!(manager.add_feedback_char('0', &mut state));
        assert!(manager.add_feedback_char('1', &mut state));
        assert!(manager.add_feedback_char('2', &mut state));
        assert_eq!(manager.get_feedback_input(), "012");

        // Invalid character should fail
        assert!(!manager.add_feedback_char('3', &mut state));
        assert_eq!(manager.get_feedback_input(), "012");

        // Complete feedback
        assert!(manager.add_feedback_char('1', &mut state));
        assert!(manager.add_feedback_char('0', &mut state));

        // Submit
        let result = manager.submit_feedback(&mut state);
        assert_eq!(result, Some(("hello".to_string(), "01210".to_string())));
        assert!(!manager.is_in_feedback_mode());
    }

    #[test]
    fn test_feedback_validation() {
        assert!(FeedbackInputManager::validate_feedback("01210").unwrap());
        assert!(FeedbackInputManager::validate_feedback("22222").unwrap());
        assert!(!FeedbackInputManager::validate_feedback("0121").unwrap());
        assert!(!FeedbackInputManager::validate_feedback("012103").unwrap());
        assert!(!FeedbackInputManager::validate_feedback("0121a").unwrap());
    }

    #[test]
    fn test_auto_complete() {
        let mut manager = FeedbackInputManager::new();
        let mut state = TuiState::new();

        manager.start_feedback_input("hello".to_string(), &mut state);

        assert!(manager.auto_complete_feedback("all_gray", &mut state));
        assert_eq!(manager.get_feedback_input(), "00000");

        manager.feedback_input.clear();
        manager.feedback_cursor = 0;

        assert!(manager.auto_complete_feedback("all_green", &mut state));
        assert_eq!(manager.get_feedback_input(), "22222");
    }
}
