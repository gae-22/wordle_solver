use std::collections::VecDeque;

use super::mode::InteractionMode;

/// TUI application state
#[derive(Debug, Clone)]
pub struct TuiState {
    /// Current input being typed by the user
    pub input: String,
    /// History of guesses and their feedback
    pub guess_history: Vec<GuessHistoryEntry>,
    /// Current best guess suggestion
    pub current_suggestion: Option<String>,
    /// Number of remaining possible words
    pub remaining_words: usize,
    /// Sample of remaining words for display
    pub remaining_words_sample: Vec<String>,
    /// Whether the puzzle is solved
    pub is_solved: bool,
    /// Current cursor position in input
    pub cursor_position: usize,
    /// Whether in help mode
    pub show_help: bool,
    /// Status message to display
    pub status_message: Option<StatusMessage>,
    /// Log messages for debugging/info
    pub log_messages: VecDeque<LogMessage>,
    /// Statistics
    pub stats: GameStats,
    /// Current interaction mode
    pub interaction_mode: InteractionMode,
}

#[derive(Debug, Clone)]
pub struct GuessHistoryEntry {
    pub word: String,
    pub feedback: String,
    pub remaining_count: usize,
}

#[derive(Debug, Clone)]
pub struct StatusMessage {
    pub text: String,
    pub message_type: MessageType,
}

#[derive(Debug, Clone)]
pub enum MessageType {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct LogMessage {
    pub timestamp: std::time::Instant,
    pub level: LogLevel,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum LogLevel {
    Info,
    Debug,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct GameStats {
    pub total_guesses: usize,
    pub average_remaining_words: f64,
    pub entropy_values: Vec<f64>,
}

impl Default for TuiState {
    fn default() -> Self {
        Self {
            input: String::new(),
            guess_history: Vec::new(),
            current_suggestion: None,
            remaining_words: 0,
            remaining_words_sample: Vec::new(),
            is_solved: false,
            cursor_position: 0,
            show_help: false,
            status_message: None,
            log_messages: VecDeque::with_capacity(100),
            stats: GameStats::default(),
            interaction_mode: InteractionMode::default(),
        }
    }
}

impl Default for GameStats {
    fn default() -> Self {
        Self {
            total_guesses: 0,
            average_remaining_words: 0.0,
            entropy_values: Vec::new(),
        }
    }
}

impl TuiState {
    pub fn new() -> Self {
        let mut state = Self::default();
        // Set initial help visibility based on default mode (INPUT)
        state.update_help_visibility();
        state
    }

    /// Add a new guess to the history
    pub fn add_guess(&mut self, word: String, feedback: String, remaining_count: usize) {
        self.guess_history.push(GuessHistoryEntry {
            word,
            feedback,
            remaining_count,
        });
        self.stats.total_guesses += 1;
        self.update_average_remaining_words();
    }

    /// Set the current suggestion
    pub fn set_suggestion(&mut self, suggestion: Option<String>) {
        self.current_suggestion = suggestion;
    }

    /// Update remaining words count and sample
    pub fn update_remaining_words(&mut self, count: usize, sample: Vec<String>) {
        self.remaining_words = count;
        self.remaining_words_sample = sample;
        self.update_average_remaining_words();
    }

    /// Set solved status
    pub fn set_solved(&mut self, solved: bool) {
        self.is_solved = solved;
    }

    /// Add input character at cursor position
    pub fn add_char(&mut self, c: char) {
        if self.input.len() < 5 && c.is_ascii_alphabetic() {
            self.input
                .insert(self.cursor_position, c.to_ascii_lowercase());
            self.cursor_position += 1;
        }
    }

    /// Remove character before cursor
    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.input.remove(self.cursor_position);
        }
    }

    /// Move cursor left
    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    /// Move cursor right
    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input.len() {
            self.cursor_position += 1;
        }
    }

    /// Clear current input
    pub fn clear_input(&mut self) {
        self.input.clear();
        self.cursor_position = 0;
    }

    /// Get current input as uppercase
    pub fn get_input_uppercase(&self) -> String {
        self.input.to_uppercase()
    }

    /// Toggle help display
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    /// Set status message
    pub fn set_status(&mut self, message: String, message_type: MessageType) {
        self.status_message = Some(StatusMessage {
            text: message,
            message_type,
        });
    }

    /// Clear status message
    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    /// Add log message
    pub fn add_log(&mut self, level: LogLevel, message: String) {
        self.log_messages.push_back(LogMessage {
            timestamp: std::time::Instant::now(),
            level,
            message,
        });

        // Keep only last 100 messages
        if self.log_messages.len() > 100 {
            self.log_messages.pop_front();
        }
    }

    /// Add entropy value to stats
    pub fn add_entropy_value(&mut self, entropy: f64) {
        self.stats.entropy_values.push(entropy);
    }

    /// Update average remaining words
    fn update_average_remaining_words(&mut self) {
        if !self.guess_history.is_empty() {
            let sum: usize = self
                .guess_history
                .iter()
                .map(|entry| entry.remaining_count)
                .sum();
            self.stats.average_remaining_words = sum as f64 / self.guess_history.len() as f64;
        }
    }

    /// Get the last N log messages
    pub fn get_recent_logs(&self, count: usize) -> Vec<&LogMessage> {
        self.log_messages
            .iter()
            .rev()
            .take(count)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// Check if input is valid (5 letters)
    pub fn is_input_valid(&self) -> bool {
        self.input.len() == 5 && self.input.chars().all(|c| c.is_ascii_alphabetic())
    }

    /// Toggle interaction mode
    pub fn toggle_interaction_mode(&mut self) {
        self.interaction_mode = self.interaction_mode.toggle();
        self.update_help_visibility();
        self.set_status(
            format!("Switched to {} mode", self.interaction_mode.name()),
            MessageType::Info,
        );
    }

    /// Switch to input mode
    pub fn switch_to_input_mode(&mut self) {
        if self.interaction_mode.is_operation() {
            self.interaction_mode = InteractionMode::Input;
            self.update_help_visibility();
            self.set_status("Switched to INPUT mode".to_string(), MessageType::Info);
        }
    }

    /// Switch to operation mode
    pub fn switch_to_operation_mode(&mut self) {
        if self.interaction_mode.is_input() {
            self.interaction_mode = InteractionMode::Operation;
            self.update_help_visibility();
            self.set_status("Switched to OPERATION mode".to_string(), MessageType::Info);
        }
    }

    /// Update help visibility based on current mode
    pub fn update_help_visibility(&mut self) {
        match self.interaction_mode {
            InteractionMode::Input => {
                // Hide help in INPUT mode for cleaner interface
                self.show_help = false;
            }
            InteractionMode::Operation => {
                // Always show help in OPERATION mode
                self.show_help = true;
            }
        }
    }

    /// Check if help should be shown (considering mode)
    pub fn should_show_help(&self) -> bool {
        match self.interaction_mode {
            InteractionMode::Input => self.show_help,
            InteractionMode::Operation => true, // Always show help in OPERATION mode
        }
    }

    /// Get current interaction mode
    pub fn interaction_mode(&self) -> InteractionMode {
        self.interaction_mode
    }
}
