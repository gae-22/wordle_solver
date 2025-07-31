use crate::solver::feedback::Feedback;
use anyhow::{Result, anyhow};

/// Core Wordle game logic
#[derive(Debug, Clone)]
pub struct WordleGame {
    target_word: Option<String>,
    attempts: Vec<String>,
    feedbacks: Vec<Vec<Feedback>>,
}

impl WordleGame {
    /// Create a new Wordle game
    pub async fn new() -> Result<Self> {
        Ok(Self {
            target_word: None,
            attempts: Vec::new(),
            feedbacks: Vec::new(),
        })
    }

    /// Set the target word for the game
    pub fn set_target(&mut self, word: &str) -> Result<()> {
        if word.len() != 5 {
            return Err(anyhow!("Target word must be exactly 5 characters"));
        }
        self.target_word = Some(word.to_lowercase());
        Ok(())
    }

    /// Make a guess and get feedback
    pub fn make_guess(&mut self, guess: &str) -> Result<Vec<Feedback>> {
        if guess.len() != 5 {
            return Err(anyhow!("Guess must be exactly 5 characters"));
        }

        let target = self
            .target_word
            .as_ref()
            .ok_or_else(|| anyhow!("No target word set"))?;

        let guess = guess.to_lowercase();
        let feedback = self.calculate_feedback(&guess, target);

        self.attempts.push(guess);
        self.feedbacks.push(feedback.clone());

        Ok(feedback)
    }

    /// Calculate feedback for a guess against the target
    pub fn calculate_feedback(&self, guess: &str, target: &str) -> Vec<Feedback> {
        let guess_chars: Vec<char> = guess.chars().collect();
        let target_chars: Vec<char> = target.chars().collect();
        let mut feedback = vec![Feedback::Absent; 5];
        let mut target_used = vec![false; 5];

        // First pass: mark correct positions
        for i in 0..5 {
            if guess_chars[i] == target_chars[i] {
                feedback[i] = Feedback::Correct;
                target_used[i] = true;
            }
        }

        // Second pass: mark present but wrong position
        for i in 0..5 {
            if feedback[i] == Feedback::Absent {
                for j in 0..5 {
                    if !target_used[j] && guess_chars[i] == target_chars[j] {
                        feedback[i] = Feedback::Present;
                        target_used[j] = true;
                        break;
                    }
                }
            }
        }

        feedback
    }

    /// Check if the game is won
    pub fn is_won(&self) -> bool {
        if let Some(last_feedback) = self.feedbacks.last() {
            last_feedback.iter().all(|&f| f == Feedback::Correct)
        } else {
            false
        }
    }

    /// Get the number of attempts made
    pub fn attempts_count(&self) -> usize {
        self.attempts.len()
    }
}
