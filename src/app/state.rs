/// Game result status
#[derive(Debug, Clone, PartialEq)]
pub enum GameResult {
    /// Game is still in progress
    InProgress,
    /// Game is won (word guessed correctly)
    Won { word: String, attempts: usize },
    /// Game is failed (too many attempts or no possible words)
    Failed { attempts: usize },
}

/// Application state management
#[derive(Debug, Clone)]
pub struct AppState {
    /// Current input buffer
    pub input_buffer: Option<String>,
    /// History of guesses and their results
    pub guess_history: Vec<(String, String)>,
    /// Current best suggestion from the solver
    pub current_suggestion: Option<String>,
    /// Number of remaining possible words
    pub remaining_words_count: usize,
    /// Whether the game is solved
    pub is_solved: bool,
    /// Game result status
    pub game_result: GameResult,
    /// Top candidate words with their entropy scores (word, entropy)
    pub top_candidates: Vec<(String, f64)>,
}

impl AppState {
    /// Create a new AppState
    pub fn new() -> Self {
        Self {
            input_buffer: None,
            guess_history: Vec::new(),
            current_suggestion: None,
            remaining_words_count: 0,
            is_solved: false,
            game_result: GameResult::InProgress,
            top_candidates: Vec::new(),
        }
    }

    /// Reset the application state
    pub fn reset(&mut self) {
        self.input_buffer = None;
        self.guess_history.clear();
        self.current_suggestion = None;
        self.remaining_words_count = 0;
        self.is_solved = false;
        self.game_result = GameResult::InProgress;
        self.top_candidates.clear();
    }
}
