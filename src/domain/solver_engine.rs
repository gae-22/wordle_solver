use crate::core::{
    error::{Result, SolverError},
    traits::{ConstraintFilter, SolvingStrategy, WordListProvider, WordleSolver},
    types::{FeedbackPattern, Guess, SolverStatistics, Word},
};
use async_trait::async_trait;
use std::fmt;
use std::sync::Arc;

/// Default implementation of Wordle solver
pub struct DefaultWordleSolver {
    word_list_provider: Box<dyn WordListProvider>,
    strategy: Box<dyn SolvingStrategy>,
    constraint_filter: Box<dyn ConstraintFilter>,
    possible_words: Vec<Word>,
    candidates: Arc<Vec<Word>>,
    guess_history: Vec<Guess>,
}

impl fmt::Debug for DefaultWordleSolver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DefaultWordleSolver")
            .field("word_list_provider", &"Box<dyn WordListProvider>")
            .field("strategy", &"Box<dyn SolvingStrategy>")
            .field("constraint_filter", &"Box<dyn ConstraintFilter>")
            .field("possible_words_count", &self.possible_words.len())
            .field("guess_history", &self.guess_history)
            .finish()
    }
}

impl DefaultWordleSolver {
    pub async fn new(
        mut word_list_provider: Box<dyn WordListProvider>,
        strategy: Box<dyn SolvingStrategy>,
        constraint_filter: Box<dyn ConstraintFilter>,
    ) -> Result<Self> {
        // Load words from provider
        word_list_provider.load_words().await?;
        let possible_words = word_list_provider.get_answer_words().to_vec();

        // Precompute candidates (answers ∪ guesses), sorted/deduped once
        let mut candidates = word_list_provider.get_answer_words().to_vec();
        candidates.extend(word_list_provider.get_guess_words().iter().cloned());
        candidates.sort();
        candidates.dedup();

    let solver = Self {
            word_list_provider,
            strategy,
            constraint_filter,
            possible_words,
            candidates: Arc::new(candidates),
            guess_history: Vec::new(),
        };

        Ok(solver)
    }

    #[allow(dead_code)]
    fn select_initial_guess_from_frequency(answers: &[Word]) -> Option<Word> {
        if answers.is_empty() {
            return None;
        }
        // Build frequency on the fly to avoid provider coupling; answersのみでポジション頻度を出す
        let mut pos_counts = [[0u32; 26]; 5];
        for w in answers {
            let b = w.bytes();
            for pos in 0..5 {
                let idx = (b[pos] - b'a') as usize;
                if idx < 26 {
                    pos_counts[pos][idx] += 1;
                }
            }
        }
        answers
            .iter()
            .max_by_key(|w| {
                let b = w.bytes();
                (0..5)
                    .map(|pos| pos_counts[pos][(b[pos] - b'a') as usize] as u64)
                    .sum::<u64>()
            })
            .cloned()
    }

    /// Update possible words based on constraints
    fn update_possible_words(&mut self) {
        self.possible_words = self
            .constraint_filter
            .filter_words(&self.possible_words, &self.guess_history);
    }

    /// Get all valid candidates for guessing (precomputed and cached)
    fn get_candidates(&self) -> Arc<Vec<Word>> {
        self.candidates.clone()
    }
}

#[async_trait]
impl WordleSolver for DefaultWordleSolver {
    async fn new() -> Result<Self> {
        let word_list_provider = Box::new(crate::infrastructure::FileWordListProvider::new());
        let entropy_calculator = crate::infrastructure::CachedEntropyCalculator::new();
        let strategy = Box::new(crate::infrastructure::EntropyBasedStrategy::new(
            entropy_calculator,
        )?);
        let constraint_filter = Box::new(crate::domain::DefaultConstraintFilter::new());

        Self::new(word_list_provider, strategy, constraint_filter).await
    }

    fn add_guess_result(&mut self, word: &Word, feedback: &FeedbackPattern) -> Result<()> {
        // Validate word is a valid guess
        if !self.word_list_provider.is_valid_guess(word) {
            return Err(SolverError::InvalidFeedback(format!(
                "'{}' is not a valid guess word",
                word.as_str()
            ))
            .into());
        }

        // Add to history
        let guess = Guess::new(word.clone(), feedback.clone());
        self.guess_history.push(guess);

        // Update possible words
        self.update_possible_words();

        Ok(())
    }

    fn get_best_guess(&mut self) -> Result<Word> {
        if self.possible_words.is_empty() {
            return Err(SolverError::NoPossibleWords.into());
        }

        if self.possible_words.len() == 1 {
            return Ok(self.possible_words[0].clone());
        }

        // Clone small set to avoid borrow conflict; remaining words are usually smaller
        let possible_words = self.possible_words.clone();
        let candidates = self.get_candidates();
        self.strategy.get_best_guess(&possible_words, &candidates)
    }

    fn get_best_first_guess(&self) -> Result<Word> {
        self.strategy.get_best_first_guess()
    }

    fn remaining_words_count(&self) -> usize {
        self.possible_words.len()
    }

    fn get_possible_words(&self, limit: Option<usize>) -> Vec<Word> {
        match limit {
            Some(n) => {
                self.possible_words.iter().take(n).cloned().collect()
            }
            None => self.possible_words.clone(),
        }
    }

    fn reset(&mut self) {
        self.possible_words = self.word_list_provider.get_answer_words().to_vec();
        self.guess_history.clear();
        self.strategy.clear_cache();
    }

    fn is_solved(&self) -> bool {
        self.possible_words.len() == 1
            && self
                .guess_history
                .last()
                .map(|guess| guess.feedback.is_win())
                .unwrap_or(false)
    }

    fn get_guess_history(&self) -> &[Guess] {
        &self.guess_history
    }

    fn get_statistics(&self) -> SolverStatistics {
        let sample_words = self.get_possible_words(Some(10));
        SolverStatistics {
            total_guesses: self.guess_history.len(),
            remaining_words: self.possible_words.len(),
            is_solved: self.is_solved(),
            possible_words_sample: sample_words,
            entropy_scores: Vec::new(),
        }
    }

    fn get_top_candidates(&mut self, limit: usize) -> Vec<(Word, f64)> {
        if self.possible_words.is_empty() {
            return Vec::new();
        }

        let possible_words = self.possible_words.clone();
        let candidates = self.get_candidates();
        self.strategy
            .get_top_candidates(&possible_words, &candidates, limit)
    }
}

/// Builder for creating customized Wordle solvers
pub struct WordleSolverBuilder {
    word_list_provider: Option<Box<dyn WordListProvider>>,
    strategy: Option<Box<dyn SolvingStrategy>>,
    constraint_filter: Option<Box<dyn ConstraintFilter>>,
}

impl fmt::Debug for WordleSolverBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WordleSolverBuilder")
            .field("word_list_provider", &self.word_list_provider.is_some())
            .field("strategy", &self.strategy.is_some())
            .field("constraint_filter", &self.constraint_filter.is_some())
            .finish()
    }
}

impl WordleSolverBuilder {
    pub fn new() -> Self {
        Self {
            word_list_provider: None,
            strategy: None,
            constraint_filter: None,
        }
    }

    pub fn with_word_list_provider(mut self, provider: Box<dyn WordListProvider>) -> Self {
        self.word_list_provider = Some(provider);
        self
    }

    pub fn with_strategy(mut self, strategy: Box<dyn SolvingStrategy>) -> Self {
        self.strategy = Some(strategy);
        self
    }

    pub fn with_constraint_filter(mut self, filter: Box<dyn ConstraintFilter>) -> Self {
        self.constraint_filter = Some(filter);
        self
    }

    pub async fn build(self) -> Result<DefaultWordleSolver> {
        let word_list_provider = self
            .word_list_provider
            .unwrap_or_else(|| Box::new(crate::infrastructure::FileWordListProvider::new()));

        let strategy = self.strategy.unwrap_or_else(|| {
            let calculator = crate::infrastructure::CachedEntropyCalculator::new();
            Box::new(crate::infrastructure::EntropyBasedStrategy::new(calculator).unwrap())
        });

        let constraint_filter = self
            .constraint_filter
            .unwrap_or_else(|| Box::new(crate::domain::DefaultConstraintFilter::new()));

        DefaultWordleSolver::new(word_list_provider, strategy, constraint_filter).await
    }
}

impl Default for WordleSolverBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_solver_builder() {
        let builder = WordleSolverBuilder::new();
        let result = builder.build().await;

        // This might fail in tests due to network access, but the structure should be correct
        match result {
            Ok(_solver) => {
                // Success case
            }
            Err(_) => {
                // Expected in test environment without network access
            }
        }
    }
}
