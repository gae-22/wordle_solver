use crate::game::WordList;
use crate::solver::{EntropyCalculator, Feedback, generate_feedback, parse_feedback};
use anyhow::{Result, anyhow};

/// Main Wordle solver using entropy-based strategy
pub struct WordleSolver {
    word_list: WordList,
    entropy_calculator: EntropyCalculator,
    possible_words: Vec<String>,
    guess_history: Vec<(String, Vec<Feedback>)>,
}

impl WordleSolver {
    /// Create a new WordleSolver
    pub async fn new() -> Result<Self> {
        let word_list = WordList::new().await?;
        let possible_words = word_list.get_answer_words().to_vec();

        Ok(Self {
            word_list,
            entropy_calculator: EntropyCalculator::new(),
            possible_words,
            guess_history: Vec::new(),
        })
    }

    /// Get the best first guess (pre-calculated optimal starting word)
    pub fn get_best_first_guess(&self) -> Result<String> {
        // "adieu" is statistically one of the best starting words
        // due to its high vowel content and common letters
        Ok("adieu".to_string())
    }

    /// Add a guess result and update the possible words
    pub fn add_guess_result(&mut self, word: &str, result: &str) -> Result<()> {
        let feedback =
            parse_feedback(result).map_err(|e| anyhow!("Failed to parse feedback: {}", e))?;

        if word.len() != 5 {
            return Err(anyhow!("Word must be exactly 5 characters"));
        }

        // Add to history
        self.guess_history
            .push((word.to_lowercase(), feedback.clone()));

        // Filter possible words based on this feedback
        self.filter_possible_words(word, &feedback);

        Ok(())
    }

    /// Get the best next guess based on current state
    pub fn get_best_guess(&mut self) -> Result<String> {
        if self.possible_words.is_empty() {
            return Err(anyhow!("No possible words remaining"));
        }

        if self.possible_words.len() == 1 {
            return Ok(self.possible_words[0].clone());
        }

        // Get all valid guess words as candidates
        let all_words = self.word_list.get_answer_words();
        let candidates: Vec<String> = all_words
            .iter()
            .filter(|word| self.word_list.is_valid_guess(word))
            .cloned()
            .collect();

        // Find the word with maximum entropy
        self.entropy_calculator
            .find_best_guess(&candidates, &self.possible_words)
            .ok_or_else(|| anyhow!("Failed to find best guess"))
    }

    /// Filter possible words based on feedback
    fn filter_possible_words(&mut self, guess: &str, feedback: &[Feedback]) {
        self.possible_words.retain(|word| {
            let expected_feedback = generate_feedback(guess, word);
            expected_feedback == feedback
        });
    }

    /// Get the number of remaining possible words
    pub fn remaining_words_count(&self) -> usize {
        self.possible_words.len()
    }

    /// Get the current possible words (limited to first N for performance)
    pub fn get_possible_words(&self, limit: Option<usize>) -> Vec<String> {
        match limit {
            Some(n) => self.possible_words.iter().take(n).cloned().collect(),
            None => self.possible_words.clone(),
        }
    }

    /// Reset the solver to initial state
    pub fn reset(&mut self) {
        self.possible_words = self.word_list.get_answer_words().to_vec();
        self.guess_history.clear();
        self.entropy_calculator.clear_cache();
    }

    /// Check if the puzzle is solved
    pub fn is_solved(&self) -> bool {
        self.possible_words.len() == 1
            && self
                .guess_history
                .last()
                .map(|(_, feedback)| feedback.iter().all(|&f| f == Feedback::Correct))
                .unwrap_or(false)
    }

    /// Get guess history
    pub fn get_guess_history(&self) -> &[(String, Vec<Feedback>)] {
        &self.guess_history
    }

    /// Calculate statistics for the current state
    pub fn get_statistics(&self) -> SolverStatistics {
        SolverStatistics {
            total_guesses: self.guess_history.len(),
            remaining_words: self.possible_words.len(),
            is_solved: self.is_solved(),
            possible_words_sample: self.get_possible_words(Some(10)),
        }
    }

    /// Get top candidate words sorted by entropy
    pub fn get_top_candidates(&mut self, limit: usize) -> Vec<(String, f64)> {
        if self.possible_words.is_empty() {
            return Vec::new();
        }

        // Get all valid guess words as candidates
        let all_words = self.word_list.get_answer_words();
        let candidates: Vec<String> = all_words
            .iter()
            .filter(|word| self.word_list.is_valid_guess(word))
            .cloned()
            .collect();

        self.entropy_calculator
            .get_top_candidates(&candidates, &self.possible_words, limit)
    }
}

/// Statistics for the solver state
#[derive(Debug, Clone)]
pub struct SolverStatistics {
    pub total_guesses: usize,
    pub remaining_words: usize,
    pub is_solved: bool,
    pub possible_words_sample: Vec<String>,
}
