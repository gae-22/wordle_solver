use crate::core::{
    error::Result,
    types::{FeedbackPattern, GameResult, Guess, SolverStatistics, Word},
};
use async_trait::async_trait;

/// Trait for word list providers
#[async_trait]
pub trait WordListProvider: Send + Sync + std::fmt::Debug {
    /// Load all available words
    async fn load_words(&mut self) -> Result<Vec<Word>>;

    /// Get answer words (possible solutions)
    fn get_answer_words(&self) -> &[Word];

    /// Get guess words (valid guesses)
    fn get_guess_words(&self) -> &[Word];

    /// Check if a word is a valid guess
    fn is_valid_guess(&self, word: &Word) -> bool;

    /// Check if a word is a possible answer
    fn is_possible_answer(&self, word: &Word) -> bool;
}

/// Trait for solving strategies
pub trait SolvingStrategy: Send + Sync + std::fmt::Debug {
    /// Get the best next guess given current constraints
    fn get_best_guess(&mut self, possible_words: &[Word], candidates: &[Word]) -> Result<Word>;

    /// Get the best first guess
    fn get_best_first_guess(&self) -> Result<Word>;

    /// Get top N candidates with their scores
    fn get_top_candidates(
        &mut self,
        possible_words: &[Word],
        candidates: &[Word],
        limit: usize,
    ) -> Vec<(Word, f64)>;

    /// Clear any internal caches
    fn clear_cache(&mut self);
}

/// Trait for entropy calculation
pub trait EntropyCalculator: Send + Sync + std::fmt::Debug {
    /// Calculate entropy for a guess against possible words
    fn calculate_entropy(&self, guess: &Word, possible_words: &[Word]) -> f64;

    /// Calculate expected information gain
    fn calculate_information_gain(&self, guess: &Word, possible_words: &[Word]) -> f64;

    /// Find the guess with maximum entropy
    fn find_max_entropy_guess(&self, candidates: &[Word], possible_words: &[Word]) -> Option<Word>;
}

/// Trait for feedback generation
pub trait FeedbackGenerator: Send + Sync + std::fmt::Debug {
    /// Generate feedback pattern for a guess against a target
    fn generate_feedback(&self, guess: &Word, target: &Word) -> FeedbackPattern;

    /// Check if a word is consistent with given constraints
    fn is_consistent(&self, word: &Word, constraints: &[Guess]) -> bool;
}

/// Trait for game engine
#[async_trait]
pub trait GameEngine: Send + Sync + std::fmt::Debug {
    /// Create a new game
    async fn new() -> Result<Self>
    where
        Self: Sized;

    /// Set the target word
    fn set_target(&mut self, word: &Word) -> Result<()>;

    /// Make a guess and get feedback
    fn make_guess(&mut self, guess: &Word) -> Result<FeedbackPattern>;

    /// Check if the game is finished
    fn is_finished(&self) -> bool;

    /// Get the current game result
    fn get_result(&self) -> GameResult;

    /// Get the number of attempts made
    fn attempts_count(&self) -> usize;

    /// Get the guess history
    fn get_history(&self) -> &[Guess];
}

/// Trait for Wordle solver
#[async_trait]
pub trait WordleSolver: Send + Sync + std::fmt::Debug {
    /// Create a new solver
    async fn new() -> Result<Self>
    where
        Self: Sized;

    /// Add a guess result and update internal state
    fn add_guess_result(&mut self, word: &Word, feedback: &FeedbackPattern) -> Result<()>;

    /// Get the best next guess
    fn get_best_guess(&mut self) -> Result<Word>;

    /// Get the best first guess
    fn get_best_first_guess(&self) -> Result<Word>;

    /// Get remaining possible words count
    fn remaining_words_count(&self) -> usize;

    /// Get current possible words (with optional limit)
    fn get_possible_words(&self, limit: Option<usize>) -> Vec<Word>;

    /// Reset solver to initial state
    fn reset(&mut self);

    /// Check if puzzle is solved
    fn is_solved(&self) -> bool;

    /// Get guess history
    fn get_guess_history(&self) -> &[Guess];

    /// Get solver statistics
    fn get_statistics(&self) -> SolverStatistics;

    /// Get top candidate guesses
    fn get_top_candidates(&mut self, limit: usize) -> Vec<(Word, f64)>;
}

/// Trait for constraint filtering
pub trait ConstraintFilter: Send + Sync + std::fmt::Debug {
    /// Filter words based on guess constraints
    fn filter_words(&self, words: &[Word], constraints: &[Guess]) -> Vec<Word>;

    /// Check if a single word satisfies all constraints
    fn satisfies_constraints(&self, word: &Word, constraints: &[Guess]) -> bool;
}

/// Trait for application state management (simplified for dyn compatibility)
pub trait StateManager: Send + Sync + std::fmt::Debug {
    type State;

    /// Get current state
    fn get_state(&self) -> &Self::State;

    /// Reset state to initial values
    fn reset_state(&mut self);
}

/// Extended state manager with update capabilities
pub trait StateUpdater<S>: Send + Sync {
    /// Update state
    fn update_state<F>(&mut self, update_fn: F) -> Result<()>
    where
        F: FnOnce(&mut S) -> Result<()>;
}

/// Trait for user interface
#[async_trait]
pub trait UserInterface: Send + Sync {
    type Event;

    /// Initialize the interface
    async fn initialize(&mut self) -> Result<()>;

    /// Handle user events
    async fn handle_event(&mut self, event: Self::Event) -> Result<bool>;

    /// Render the current state
    fn render(&mut self) -> Result<()>;

    /// Clean up resources
    async fn cleanup(&mut self) -> Result<()>;
}

/// Trait for data persistence
#[async_trait]
pub trait DataPersistence: Send + Sync {
    type Data;

    /// Load data from storage
    async fn load(&mut self) -> Result<Self::Data>;

    /// Save data to storage
    async fn save(&mut self, data: &Self::Data) -> Result<()>;

    /// Check if data exists
    async fn exists(&self) -> bool;
}
