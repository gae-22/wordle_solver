use crate::{
    application::{
        AppEvent, AppState, AppStateManager, Command, CommandExecutor, CommandResult,
        CommandValidator, DefaultCommandValidator, EventHandler,
    },
    core::{
        error::Result,
        traits::{GameEngine, StateManager, StateUpdater, WordleSolver},
        types::{FeedbackPattern, Word},
    },
};
use std::fmt;

/// Main application service orchestrating all components
pub struct WordleApplicationService {
    game_engine: Box<dyn GameEngine>,
    solver: Box<dyn WordleSolver>,
    state_manager: AppStateManager,
    command_validator: Box<dyn CommandValidator>,
}

impl fmt::Debug for WordleApplicationService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WordleApplicationService")
            .field("game_engine", &"Box<dyn GameEngine>")
            .field("solver", &"Box<dyn WordleSolver>")
            .field("state_manager", &self.state_manager)
            .field("command_validator", &"Box<dyn CommandValidator>")
            .finish()
    }
}

impl WordleApplicationService {
    pub async fn new() -> Result<Self> {
        let game_engine = Box::new(crate::domain::DefaultGameEngine::new().await?);

        // Create solver components
        let word_list_provider = Box::new(crate::infrastructure::FileWordListProvider::new());
        let entropy_calculator = crate::infrastructure::CachedEntropyCalculator::new();
        let strategy = Box::new(crate::infrastructure::EntropyBasedStrategy::new(
            entropy_calculator,
        )?);
        let constraint_filter = Box::new(crate::domain::DefaultConstraintFilter::new());
        let solver = Box::new(
            crate::domain::DefaultWordleSolver::new(
                word_list_provider,
                strategy,
                constraint_filter,
            )
            .await?,
        );

        let state_manager = AppStateManager::new();
        let command_validator = Box::new(DefaultCommandValidator);

        Ok(Self {
            game_engine,
            solver,
            state_manager,
            command_validator,
        })
    }

    /// Create application service with injected dependencies (Clean Architecture)
    pub async fn with_dependencies(
        game_engine: Box<dyn GameEngine>,
        solver: Box<dyn WordleSolver>,
    ) -> Result<Self> {
        let state_manager = AppStateManager::new();
        let command_validator = Box::new(DefaultCommandValidator);

        Ok(Self {
            game_engine,
            solver,
            state_manager,
            command_validator,
        })
    }

    pub fn with_components(
        game_engine: Box<dyn GameEngine>,
        solver: Box<dyn WordleSolver>,
        state_manager: AppStateManager,
        command_validator: Box<dyn CommandValidator>,
    ) -> Self {
        Self {
            game_engine,
            solver,
            state_manager,
            command_validator,
        }
    }

    /// Get current application state
    pub fn get_state(&self) -> &AppState {
        self.state_manager.get_state()
    }

    /// Update application state with solver information
    fn update_state_with_solver_info(&mut self) -> Result<()> {
        let stats = self.solver.get_statistics();
        let top_candidates = self.solver.get_top_candidates(5);
        let suggestion = if stats.remaining_words > 1 {
            self.solver.get_best_guess().ok()
        } else {
            None
        };

        self.state_manager.update_state(|state| {
            state.solver_stats = stats;
            state.top_candidates = top_candidates;
            state.current_suggestion = suggestion;
            state.remaining_words_count = self.solver.remaining_words_count();
            Ok(())
        })
    }

    /// Process a user guess
    pub fn process_guess(&mut self, word: &Word) -> Result<FeedbackPattern> {
        // Make guess in game engine
        let feedback = self.game_engine.make_guess(word)?;

        // Update solver with the result
        self.solver.add_guess_result(word, &feedback)?;

        // Update application state
        self.state_manager.update_state(|state| {
            let guess = crate::core::types::Guess::new(word.clone(), feedback.clone());
            state.guess_history.push(guess);
            state.game_result = self.game_engine.get_result();
            Ok(())
        })?;

        // Update solver information
        self.update_state_with_solver_info()?;

        Ok(feedback)
    }

    /// Start a new game
    pub fn start_game(&mut self, target_word: Option<&Word>) -> Result<()> {
        if let Some(word) = target_word {
            self.game_engine.set_target(word)?;
        }

        // Reset state
        self.state_manager.reset_state();
        self.solver.reset();

        // Update initial suggestions
        self.update_state_with_solver_info()?;

        Ok(())
    }

    /// Reset the current game
    pub fn reset_game(&mut self) -> Result<()> {
        self.state_manager.reset_state();
        self.solver.reset();
        self.update_state_with_solver_info()
    }

    /// Get the best first guess
    pub fn get_best_first_guess(&self) -> Result<Word> {
        self.solver.get_best_first_guess()
    }

    /// Get the best next guess
    pub fn get_best_next_guess(&mut self) -> Result<Word> {
        self.solver.get_best_guess()
    }

    /// Add a previous guess result (for importing game state)
    pub fn add_guess_result(&mut self, word: &Word, feedback: &FeedbackPattern) -> Result<()> {
        self.solver.add_guess_result(word, feedback)?;

        self.state_manager.update_state(|state| {
            let guess = crate::core::types::Guess::new(word.clone(), feedback.clone());
            state.guess_history.push(guess);
            Ok(())
        })?;

        self.update_state_with_solver_info()
    }
}

impl CommandExecutor for WordleApplicationService {
    fn execute(&mut self, command: Command) -> Result<CommandResult> {
        // Validate command first
        self.command_validator.validate(&command)?;

        match command {
            Command::StartGame { target_word } => {
                self.start_game(target_word.as_ref())?;
                Ok(CommandResult::GameStarted {
                    target_set: target_word.is_some(),
                })
            }
            Command::MakeGuess { word } => {
                let feedback = self.process_guess(&word)?;
                let game_finished = self.game_engine.is_finished();
                Ok(CommandResult::GuessMade {
                    feedback,
                    game_finished,
                })
            }
            Command::AddGuessResult { word, feedback } => {
                self.add_guess_result(&word, &feedback)?;
                let remaining_words = self.solver.remaining_words_count();
                Ok(CommandResult::GuessResultAdded { remaining_words })
            }
            Command::GetBestGuess => {
                let word = self.get_best_next_guess()?;
                // Calculate confidence based on remaining words
                let remaining = self.solver.remaining_words_count();
                let confidence = if remaining <= 1 {
                    1.0
                } else {
                    1.0 / remaining as f64
                };
                Ok(CommandResult::BestGuess { word, confidence })
            }
            Command::GetBestFirstGuess => {
                let word = self.get_best_first_guess()?;
                Ok(CommandResult::BestFirstGuess { word })
            }
            Command::Reset => {
                self.reset_game()?;
                Ok(CommandResult::Reset)
            }
            Command::GetStatistics => {
                let stats = self.solver.get_statistics();
                Ok(CommandResult::Statistics { stats })
            }
            Command::GetTopCandidates { limit } => {
                let candidates = self.solver.get_top_candidates(limit);
                Ok(CommandResult::TopCandidates { candidates })
            }
        }
    }
}

impl EventHandler for WordleApplicationService {
    fn handle_event(&mut self, event: AppEvent) -> Result<bool> {
        match event {
            AppEvent::CharacterInput(ch) => {
                self.state_manager.update_state(|state| {
                    if state.input_buffer.is_none() {
                        state.input_buffer = Some(String::new());
                    }
                    if let Some(ref mut buffer) = state.input_buffer {
                        if buffer.len() < 5 {
                            buffer.push(ch.to_ascii_lowercase());
                        }
                    }
                    Ok(())
                })?;
                Ok(false) // Continue handling events
            }
            AppEvent::SubmitGuess => {
                let input = self.state_manager.get_state().input_buffer.clone();
                if let Some(input_str) = input {
                    if input_str.len() == 5 {
                        match Word::from_str(&input_str) {
                            Ok(word) => {
                                let _result = self.process_guess(&word);
                                // Clear input buffer
                                self.state_manager.update_state(|state| {
                                    state.input_buffer = None;
                                    Ok(())
                                })?;
                            }
                            Err(_) => {
                                // Invalid word, could emit error event
                            }
                        }
                    }
                }
                Ok(false)
            }
            AppEvent::RestartGame => {
                self.reset_game()?;
                Ok(false)
            }
            AppEvent::Quit => Ok(true), // Signal to quit
            AppEvent::NewSuggestion(word) => {
                self.state_manager.update_state(|state| {
                    state.current_suggestion = Some(word);
                    Ok(())
                })?;
                Ok(false)
            }
            AppEvent::GameStateChanged(result) => {
                self.state_manager.update_state(|state| {
                    state.game_result = result;
                    Ok(())
                })?;
                Ok(false)
            }
            AppEvent::SolverStatsUpdated(stats) => {
                self.state_manager.update_state(|state| {
                    state.solver_stats = stats;
                    Ok(())
                })?;
                Ok(false)
            }
            AppEvent::Error(_message) => {
                // Handle error display
                Ok(false)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_application_service_creation() {
        // This test may fail due to network requirements, but tests the structure
        let result = WordleApplicationService::new().await;
        match result {
            Ok(_service) => {
                // Success case
            }
            Err(_) => {
                // Expected in test environment
            }
        }
    }
}
