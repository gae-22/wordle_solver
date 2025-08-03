use crate::core::{
    error::Result,
    traits::{StateManager, StateUpdater},
    types::{GameResult, Guess, SolverStatistics, Word},
};

/// Application state
#[derive(Debug, Clone)]
pub struct AppState {
    /// Current input buffer for user typing
    pub input_buffer: Option<String>,
    /// History of all guesses made
    pub guess_history: Vec<Guess>,
    /// Current best suggestion from solver
    pub current_suggestion: Option<Word>,
    /// Number of remaining possible words
    pub remaining_words_count: usize,
    /// Current game result
    pub game_result: GameResult,
    /// Top candidate words with scores
    pub top_candidates: Vec<(Word, f64)>,
    /// Current solver statistics
    pub solver_stats: SolverStatistics,
    /// Whether the solver is actively running
    pub solver_active: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            input_buffer: None,
            guess_history: Vec::new(),
            current_suggestion: None,
            remaining_words_count: 0,
            game_result: GameResult::InProgress,
            top_candidates: Vec::new(),
            solver_stats: SolverStatistics::new(),
            solver_active: false,
        }
    }

    pub fn reset(&mut self) {
        self.input_buffer = None;
        self.guess_history.clear();
        self.current_suggestion = None;
        self.remaining_words_count = 0;
        self.game_result = GameResult::InProgress;
        self.top_candidates.clear();
        self.solver_stats = SolverStatistics::new();
        self.solver_active = false;
    }

    pub fn is_game_finished(&self) -> bool {
        self.game_result.is_finished()
    }

    pub fn is_game_won(&self) -> bool {
        self.game_result.is_won()
    }

    /// Get the last guess made
    pub fn last_guess(&self) -> Option<&Guess> {
        self.guess_history.last()
    }

    /// Count of guesses made so far
    pub fn guess_count(&self) -> usize {
        self.guess_history.len()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// State manager for the application
#[derive(Debug)]
pub struct AppStateManager {
    state: AppState,
}

impl AppStateManager {
    pub fn new() -> Self {
        Self {
            state: AppState::new(),
        }
    }
}

impl Default for AppStateManager {
    fn default() -> Self {
        Self::new()
    }
}

impl StateManager for AppStateManager {
    type State = AppState;

    fn get_state(&self) -> &Self::State {
        &self.state
    }

    fn reset_state(&mut self) {
        self.state.reset();
    }
}

impl StateUpdater<AppState> for AppStateManager {
    fn update_state<F>(&mut self, update_fn: F) -> Result<()>
    where
        F: FnOnce(&mut AppState) -> Result<()>,
    {
        update_fn(&mut self.state)
    }
}

/// Events that can occur in the application
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// User typed a character
    CharacterInput(char),
    /// User pressed Enter to submit guess
    SubmitGuess,
    /// User requested to restart the game
    RestartGame,
    /// User requested to quit
    Quit,
    /// Solver provided a new suggestion
    NewSuggestion(Word),
    /// Game state changed
    GameStateChanged(GameResult),
    /// Solver statistics updated
    SolverStatsUpdated(SolverStatistics),
    /// Error occurred
    Error(String),
}

/// Application event handler
pub trait EventHandler {
    /// Handle an application event
    fn handle_event(&mut self, event: AppEvent) -> Result<bool>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_creation() {
        let state = AppState::new();
        assert!(!state.is_game_finished());
        assert!(!state.is_game_won());
        assert_eq!(state.guess_count(), 0);
    }

    #[test]
    fn test_state_manager() {
        let mut manager = AppStateManager::new();

        // Test getting state
        let state = manager.get_state();
        assert_eq!(state.guess_count(), 0);

        // Test updating state
        let result = manager.update_state(|state| {
            state.remaining_words_count = 100;
            Ok(())
        });

        assert!(result.is_ok());
        assert_eq!(manager.get_state().remaining_words_count, 100);

        // Test resetting state
        manager.reset_state();
        assert_eq!(manager.get_state().remaining_words_count, 0);
    }
}
