use crate::core::{
    error::Result,
    types::{FeedbackPattern, Word},
};

/// Commands that can be executed in the application
#[derive(Debug, Clone)]
pub enum Command {
    /// Start a new game
    StartGame { target_word: Option<Word> },
    /// Make a guess
    MakeGuess { word: Word },
    /// Add a guess result (for importing previous game state)
    AddGuessResult {
        word: Word,
        feedback: FeedbackPattern,
    },
    /// Get the best next guess
    GetBestGuess,
    /// Get the best first guess
    GetBestFirstGuess,
    /// Reset the game/solver
    Reset,
    /// Get current statistics
    GetStatistics,
    /// Get top candidate guesses
    GetTopCandidates { limit: usize },
}

/// Result of executing a command
#[derive(Debug, Clone)]
pub enum CommandResult {
    /// Game started successfully
    GameStarted { target_set: bool },
    /// Guess was made successfully
    GuessMade {
        feedback: FeedbackPattern,
        game_finished: bool,
    },
    /// Guess result was added
    GuessResultAdded { remaining_words: usize },
    /// Best guess determined
    BestGuess { word: Word, confidence: f64 },
    /// Best first guess determined
    BestFirstGuess { word: Word },
    /// Game/solver was reset
    Reset,
    /// Statistics retrieved
    Statistics {
        stats: crate::core::types::SolverStatistics,
    },
    /// Top candidates retrieved
    TopCandidates { candidates: Vec<(Word, f64)> },
    /// Command failed
    Error { message: String },
}

/// Trait for command execution
pub trait CommandExecutor {
    /// Execute a command and return the result
    fn execute(&mut self, command: Command) -> Result<CommandResult>;
}

/// Command validation
pub trait CommandValidator: std::fmt::Debug {
    /// Validate a command before execution
    fn validate(&self, command: &Command) -> Result<()>;
}

/// Default command validator
#[derive(Debug, Default)]
pub struct DefaultCommandValidator;

impl CommandValidator for DefaultCommandValidator {
    fn validate(&self, command: &Command) -> Result<()> {
        match command {
            Command::StartGame { .. } => Ok(()),
            Command::MakeGuess { word } => {
                // Validate word length and characters
                if word.as_str().len() != 5 {
                    return Err(crate::core::error::GameError::InvalidWordLength {
                        expected: 5,
                        actual: word.as_str().len(),
                    }
                    .into());
                }
                Ok(())
            }
            Command::AddGuessResult { word, feedback } => {
                // Validate word and feedback consistency
                if word.as_str().len() != 5 {
                    return Err(crate::core::error::GameError::InvalidWordLength {
                        expected: 5,
                        actual: word.as_str().len(),
                    }
                    .into());
                }
                if feedback.as_slice().len() != 5 {
                    return Err(crate::core::error::SolverError::InvalidFeedback(
                        "Feedback must have exactly 5 elements".to_string(),
                    )
                    .into());
                }
                Ok(())
            }
            Command::GetTopCandidates { limit } => {
                if *limit == 0 {
                    return Err(crate::core::error::SolverError::InvalidFeedback(
                        "Limit must be greater than 0".to_string(),
                    )
                    .into());
                }
                Ok(())
            }
            // Other commands don't need validation
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_validation() {
        let validator = DefaultCommandValidator;

        // Valid command
        let word = Word::from_str("adieu").unwrap();
        let command = Command::MakeGuess { word };
        assert!(validator.validate(&command).is_ok());

        // Invalid limit
        let command = Command::GetTopCandidates { limit: 0 };
        assert!(validator.validate(&command).is_err());
    }
}
