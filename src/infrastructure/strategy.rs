use crate::core::{
    error::{Result, SolverError},
    traits::{EntropyCalculator, SolvingStrategy},
    types::Word,
};

/// Entropy-based solving strategy
#[derive(Debug)]
pub struct EntropyBasedStrategy<E: EntropyCalculator> {
    entropy_calculator: E,
    best_first_guess: Word,
}

impl<E: EntropyCalculator> EntropyBasedStrategy<E> {
    pub fn new(entropy_calculator: E) -> Result<Self> {
        let best_first_guess =
            Word::from_str("adieu").map_err(|e| SolverError::AlgorithmFailure(e))?;

        Ok(Self {
            entropy_calculator,
            best_first_guess,
        })
    }

    pub fn with_first_guess(entropy_calculator: E, first_guess: Word) -> Self {
        Self {
            entropy_calculator,
            best_first_guess: first_guess,
        }
    }
}

impl<E: EntropyCalculator> SolvingStrategy for EntropyBasedStrategy<E> {
    fn get_best_guess(&mut self, possible_words: &[Word], candidates: &[Word]) -> Result<Word> {
        if possible_words.is_empty() {
            return Err(SolverError::NoPossibleWords.into());
        }

        if possible_words.len() == 1 {
            return Ok(possible_words[0].clone());
        }

        if candidates.is_empty() {
            return Err(SolverError::NoCandidates.into());
        }

        // Use information gain for better performance in endgame
        let best_word = if possible_words.len() <= 3 {
            // When few words remain, prefer words that are possible answers
            let answer_candidates: Vec<_> = candidates
                .iter()
                .filter(|word| possible_words.contains(word))
                .cloned()
                .collect();

            if !answer_candidates.is_empty() {
                self.entropy_calculator
                    .find_max_entropy_guess(&answer_candidates, possible_words)
                    .or_else(|| {
                        self.entropy_calculator
                            .find_max_entropy_guess(candidates, possible_words)
                    })
            } else {
                self.entropy_calculator
                    .find_max_entropy_guess(candidates, possible_words)
            }
        } else {
            self.entropy_calculator
                .find_max_entropy_guess(candidates, possible_words)
        };

        best_word.ok_or_else(|| {
            SolverError::AlgorithmFailure("Could not find best guess".to_string()).into()
        })
    }

    fn get_best_first_guess(&self) -> Result<Word> {
        Ok(self.best_first_guess.clone())
    }

    fn get_top_candidates(
        &mut self,
        possible_words: &[Word],
        candidates: &[Word],
        limit: usize,
    ) -> Vec<(Word, f64)> {
        if possible_words.is_empty() || candidates.is_empty() {
            return Vec::new();
        }

        let mut scored_candidates: Vec<_> = candidates
            .iter()
            .map(|word| {
                let score = self
                    .entropy_calculator
                    .calculate_information_gain(word, possible_words);
                (word.clone(), score)
            })
            .collect();

        // Sort by score (descending)
        scored_candidates
            .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        scored_candidates.truncate(limit);
        scored_candidates
    }

    fn clear_cache(&mut self) {
        // EntropyCalculator trait doesn't have clear_cache method
        // This would need to be implemented if the calculator has caching
    }
}

/// Frequency-based strategy that considers letter frequency
#[derive(Debug)]
pub struct FrequencyBasedStrategy {
    letter_frequencies: std::collections::HashMap<char, f64>,
    best_first_guess: Word,
}

impl FrequencyBasedStrategy {
    pub fn new(word_list: &[Word]) -> Result<Self> {
        let mut letter_counts = std::collections::HashMap::new();
        let mut total_letters = 0;

        // Count letter frequencies
        for word in word_list {
            for ch in word.as_str().chars() {
                *letter_counts.entry(ch).or_insert(0) += 1;
                total_letters += 1;
            }
        }

        // Convert to frequencies
        let letter_frequencies = letter_counts
            .into_iter()
            .map(|(ch, count)| (ch, count as f64 / total_letters as f64))
            .collect();

        let best_first_guess =
            Word::from_str("adieu").map_err(|e| SolverError::AlgorithmFailure(e))?;

        Ok(Self {
            letter_frequencies,
            best_first_guess,
        })
    }

    fn score_word(&self, word: &Word) -> f64 {
        let mut score = 0.0;
        let mut seen_letters = std::collections::HashSet::new();

        for ch in word.as_str().chars() {
            if seen_letters.insert(ch) {
                // Only count each letter once per word
                score += self.letter_frequencies.get(&ch).unwrap_or(&0.0);
            }
        }

        score
    }
}

impl SolvingStrategy for FrequencyBasedStrategy {
    fn get_best_guess(&mut self, possible_words: &[Word], candidates: &[Word]) -> Result<Word> {
        if possible_words.is_empty() {
            return Err(SolverError::NoPossibleWords.into());
        }

        if possible_words.len() == 1 {
            return Ok(possible_words[0].clone());
        }

        if candidates.is_empty() {
            return Err(SolverError::NoCandidates.into());
        }

        // Score candidates by letter frequency
        let best_word = candidates
            .iter()
            .map(|word| (word.clone(), self.score_word(word)))
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(word, _)| word);

        best_word.ok_or_else(|| {
            SolverError::AlgorithmFailure("Could not find best guess".to_string()).into()
        })
    }

    fn get_best_first_guess(&self) -> Result<Word> {
        Ok(self.best_first_guess.clone())
    }

    fn get_top_candidates(
        &mut self,
        _possible_words: &[Word],
        candidates: &[Word],
        limit: usize,
    ) -> Vec<(Word, f64)> {
        let mut scored_candidates: Vec<_> = candidates
            .iter()
            .map(|word| (word.clone(), self.score_word(word)))
            .collect();

        scored_candidates
            .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored_candidates.truncate(limit);
        scored_candidates
    }

    fn clear_cache(&mut self) {
        // No cache to clear in frequency-based strategy
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::SimpleEntropyCalculator;

    #[test]
    fn test_entropy_strategy_creation() {
        let calculator = SimpleEntropyCalculator::new();
        let strategy = EntropyBasedStrategy::new(calculator);
        assert!(strategy.is_ok());
    }

    #[test]
    fn test_frequency_strategy_creation() {
        let words = vec![
            Word::from_str("apple").unwrap(),
            Word::from_str("bread").unwrap(),
            Word::from_str("crane").unwrap(),
        ];

        let strategy = FrequencyBasedStrategy::new(&words);
        assert!(strategy.is_ok());
    }
}
