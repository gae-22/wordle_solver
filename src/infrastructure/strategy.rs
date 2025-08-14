use crate::core::{
    error::{Result, SolverError},
    traits::{EntropyCalculator, SolvingStrategy},
    types::Word,
};
use rayon::prelude::*;

/// Entropy-based solving strategy
#[derive(Debug)]
pub struct EntropyBasedStrategy<E: EntropyCalculator> {
    pub(crate) entropy_calculator: E,
    pub(crate) best_first_guess: Word,
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
            // Optional heuristic prefilter (disabled by default to preserve accuracy)
            let use_prefilter = std::env::var("WORDLE_FAST_PREFILTER")
                .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE"))
                .unwrap_or(false);
            if use_prefilter && possible_words.len() > 200 {
                let filtered: Vec<_> = candidates
                    .iter()
                    .filter(|w| {
                        let s = w.as_str().as_bytes();
                        let mut seen = [false; 26];
                        for &b in s {
                            let i = (b - b'a') as usize;
                            if seen[i] {
                                return false;
                            }
                            seen[i] = true;
                        }
                        true
                    })
                    .cloned()
                    .collect();
                let pool = if !filtered.is_empty() {
                    &filtered
                } else {
                    candidates
                };
                // Parallel scan if pool is large
                let par_threshold = 256;
                if pool.len() >= par_threshold {
                    pool.par_iter()
                        .map(|w| {
                            let e = self.entropy_calculator.calculate_entropy(w, possible_words);
                            (w, e)
                        })
                        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                        .map(|(w, _)| w.clone())
                } else {
                    self.entropy_calculator
                        .find_max_entropy_guess(pool, possible_words)
                }
            } else {
                // Parallel scan if candidates large
                let par_threshold = 256;
                if candidates.len() >= par_threshold {
                    candidates
                        .par_iter()
                        .map(|w| {
                            let e = self.entropy_calculator.calculate_entropy(w, possible_words);
                            (w, e)
                        })
                        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                        .map(|(w, _)| w.clone())
                } else {
                    self.entropy_calculator
                        .find_max_entropy_guess(candidates, possible_words)
                }
            }
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

        // Parallel scoring for top candidates if set is large
        let par_threshold = 256;
        let mut scored_candidates: Vec<_> = if candidates.len() >= par_threshold {
            candidates
                .par_iter()
                .map(|word| {
                    let score = self
                        .entropy_calculator
                        .calculate_information_gain(word, possible_words);
                    (word.clone(), score)
                })
                .collect()
        } else {
            candidates
                .iter()
                .map(|word| {
                    let score = self
                        .entropy_calculator
                        .calculate_information_gain(word, possible_words);
                    (word.clone(), score)
                })
                .collect()
        };

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

/// Hybrid strategy combining entropy and frequency analysis
#[derive(Debug)]
pub struct HybridStrategy<E: EntropyCalculator> {
    entropy_calculator: E,
    frequency_weights: std::collections::HashMap<char, f64>,
    best_first_guess: Word,
    use_entropy_threshold: usize,
}

impl<E: EntropyCalculator> HybridStrategy<E> {
    pub fn new(entropy_calculator: E) -> Result<Self> {
        let best_first_guess =
            Word::from_str("adieu").map_err(|e| SolverError::AlgorithmFailure(e))?;

        // Initialize common letter frequencies
        let mut frequency_weights = std::collections::HashMap::new();
        frequency_weights.insert('e', 12.02);
        frequency_weights.insert('t', 9.10);
        frequency_weights.insert('a', 8.12);
        frequency_weights.insert('o', 7.68);
        frequency_weights.insert('i', 6.97);
        frequency_weights.insert('n', 6.75);
        frequency_weights.insert('s', 6.33);
        frequency_weights.insert('h', 6.09);
        frequency_weights.insert('r', 5.99);

        Ok(Self {
            entropy_calculator,
            frequency_weights,
            best_first_guess,
            use_entropy_threshold: 50, // Use entropy when more than 50 words remain
        })
    }

    fn calculate_frequency_score(&self, word: &Word) -> f64 {
        let mut score = 0.0;
        let mut used_chars = std::collections::HashSet::new();

        for ch in word.as_str().chars() {
            if !used_chars.contains(&ch) {
                score += self.frequency_weights.get(&ch).unwrap_or(&0.0);
                used_chars.insert(ch);
            }
        }

        score
    }

    fn calculate_hybrid_score(&self, word: &Word, possible_words: &[Word]) -> f64 {
        let entropy = self
            .entropy_calculator
            .calculate_entropy(word, possible_words);
        let frequency = self.calculate_frequency_score(word);

        // Weight entropy more heavily when many words remain
        let entropy_weight = if possible_words.len() > self.use_entropy_threshold {
            0.8
        } else {
            0.3
        };
        let frequency_weight = 1.0 - entropy_weight;

        entropy * entropy_weight + frequency * frequency_weight
    }
}

impl<E: EntropyCalculator> SolvingStrategy for HybridStrategy<E> {
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

        // Find the best guess using hybrid scoring
        let best_candidate = candidates
            .iter()
            .max_by(|a, b| {
                let score_a = self.calculate_hybrid_score(a, possible_words);
                let score_b = self.calculate_hybrid_score(b, possible_words);
                score_a
                    .partial_cmp(&score_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .ok_or(SolverError::NoCandidates)?;

        Ok(best_candidate.clone())
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
        let mut scored_candidates: Vec<_> = candidates
            .iter()
            .map(|word| {
                (
                    word.clone(),
                    self.calculate_hybrid_score(word, possible_words),
                )
            })
            .collect();

        scored_candidates
            .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored_candidates.truncate(limit);
        scored_candidates
    }

    fn clear_cache(&mut self) {
        // Clear frequency weights if needed
        self.frequency_weights.clear();
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
