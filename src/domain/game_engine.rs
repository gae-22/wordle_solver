use crate::core::{
    error::{GameError, Result},
    traits::*,
    types::{FeedbackPattern, GameResult, Guess, Word},
};
use async_trait::async_trait;
use std::fmt;

/// Default implementation of the Wordle game engine
pub struct DefaultGameEngine {
    target_word: Option<Word>,
    history: Vec<Guess>,
    result: GameResult,
    feedback_generator: Box<dyn FeedbackGenerator>,
}

impl fmt::Debug for DefaultGameEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DefaultGameEngine")
            .field("target_word", &self.target_word)
            .field("history", &self.history)
            .field("result", &self.result)
            .field("feedback_generator", &"Box<dyn FeedbackGenerator>")
            .finish()
    }
}

impl DefaultGameEngine {
    /// Create a new game engine with a feedback generator
    pub fn with_feedback_generator(feedback_generator: Box<dyn FeedbackGenerator>) -> Self {
        Self {
            target_word: None,
            history: Vec::new(),
            result: GameResult::InProgress,
            feedback_generator,
        }
    }

    /// Create a new game engine with a feedback generator (async version for compatibility)
    pub async fn with_feedback_generator_async(feedback_generator: Box<dyn FeedbackGenerator>) -> Result<Self> {
        Ok(Self::with_feedback_generator(feedback_generator))
    }

    /// Get the target word (if set)
    pub fn target_word(&self) -> Option<&Word> {
        self.target_word.as_ref()
    }
}

#[async_trait]
impl GameEngine for DefaultGameEngine {
    async fn new() -> Result<Self> {
        let feedback_generator = Box::new(crate::domain::DefaultFeedbackGenerator::new());
        Ok(Self::with_feedback_generator(feedback_generator))
    }

    fn set_target(&mut self, word: &Word) -> Result<()> {
        self.target_word = Some(word.clone());
        self.result = GameResult::InProgress;
        Ok(())
    }

    fn make_guess(&mut self, guess: &Word) -> Result<FeedbackPattern> {
        if self.is_finished() {
            return Err(GameError::GameFinished.into());
        }

        let target = self.target_word.as_ref().ok_or(GameError::NoTargetWord)?;

        let feedback = self.feedback_generator.generate_feedback(guess, target);
        let guess_entry = Guess::new(guess.clone(), feedback.clone());

        self.history.push(guess_entry);

        // Update game result
        if feedback.is_win() {
            self.result = GameResult::Won {
                word: target.clone(),
                attempts: self.history.len(),
            };
        } else if self.history.len() >= 6 {
            // Standard Wordle has 6 attempts
            self.result = GameResult::Failed {
                attempts: self.history.len(),
                reason: "Maximum attempts exceeded".to_string(),
            };
        }

        Ok(feedback)
    }

    fn is_finished(&self) -> bool {
        self.result.is_finished()
    }

    fn get_result(&self) -> GameResult {
        self.result.clone()
    }

    fn attempts_count(&self) -> usize {
        self.history.len()
    }

    fn get_history(&self) -> &[Guess] {
        &self.history
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_game_engine_basic_flow() {
        let mut game = DefaultGameEngine::new().await.unwrap();

        let target = Word::from_str("apple").unwrap();
        game.set_target(&target).unwrap();

        let guess = Word::from_str("about").unwrap();
        let _ = game.make_guess(&guess).unwrap();

        assert_eq!(game.attempts_count(), 1);
        assert!(!game.is_finished());

        // Make the winning guess
        let winning_guess = Word::from_str("apple").unwrap();
        let winning_feedback = game.make_guess(&winning_guess).unwrap();

        assert!(winning_feedback.is_win());
        assert!(game.is_finished());
        assert!(game.get_result().is_won());
    }
}
