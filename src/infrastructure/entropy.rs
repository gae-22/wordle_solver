use crate::core::{
    traits::{EntropyCalculator, FeedbackGenerator},
    types::{FeedbackPattern, Word},
};
use crate::domain::DefaultFeedbackGenerator;
use std::collections::HashMap;

/// High-performance entropy calculator with caching
#[derive(Debug)]
pub struct CachedEntropyCalculator {
    feedback_generator: DefaultFeedbackGenerator,
}

impl CachedEntropyCalculator {
    pub fn new() -> Self {
        Self {
            feedback_generator: DefaultFeedbackGenerator::new(),
        }
    }

    /// Group words by their feedback patterns
    fn group_by_feedback(
        &self,
        guess: &Word,
        possible_words: &[Word],
    ) -> HashMap<FeedbackPattern, Vec<Word>> {
        let mut groups = HashMap::new();

        for word in possible_words {
            let feedback = self.feedback_generator.generate_feedback(guess, word);
            groups
                .entry(feedback)
                .or_insert_with(Vec::new)
                .push(word.clone());
        }

        groups
    }

    /// Calculate information gain based on expected partition sizes
    fn calculate_information_gain_internal(&self, guess: &Word, possible_words: &[Word]) -> f64 {
        if possible_words.is_empty() {
            return 0.0;
        }

        let groups = self.group_by_feedback(guess, possible_words);
        let total_words = possible_words.len() as f64;

        // Calculate weighted average of log2(group_size)
        let expected_log_size: f64 = groups
            .values()
            .map(|group| {
                let probability = group.len() as f64 / total_words;
                let log_size = if group.len() > 0 {
                    (group.len() as f64).log2()
                } else {
                    0.0
                };
                probability * log_size
            })
            .sum();

        // Information gain = log2(total) - expected_log_size
        total_words.log2() - expected_log_size
    }
}

impl Default for CachedEntropyCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl EntropyCalculator for CachedEntropyCalculator {
    fn calculate_entropy(&self, guess: &Word, possible_words: &[Word]) -> f64 {
        if possible_words.is_empty() {
            return 0.0;
        }

        if possible_words.len() == 1 {
            // If only one word remains, entropy is 0
            return 0.0;
        }

        let groups = self.group_by_feedback(guess, possible_words);
        let total_words = possible_words.len() as f64;

        // Calculate Shannon entropy: -Σ(p * log2(p))
        groups
            .values()
            .map(|group| {
                let probability = group.len() as f64 / total_words;
                if probability > 0.0 {
                    -probability * probability.log2()
                } else {
                    0.0
                }
            })
            .sum()
    }

    fn calculate_information_gain(&self, guess: &Word, possible_words: &[Word]) -> f64 {
        self.calculate_information_gain_internal(guess, possible_words)
    }

    fn find_max_entropy_guess(&self, candidates: &[Word], possible_words: &[Word]) -> Option<Word> {
        if candidates.is_empty() || possible_words.is_empty() {
            return None;
        }

        candidates
            .iter()
            .map(|word| {
                let entropy = self.calculate_entropy(word, possible_words);
                (word.clone(), entropy)
            })
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(word, _)| word)
    }
}

/// Simple entropy calculator without caching (for testing/comparison)
#[derive(Debug)]
pub struct SimpleEntropyCalculator {
    feedback_generator: DefaultFeedbackGenerator,
}

impl SimpleEntropyCalculator {
    pub fn new() -> Self {
        Self {
            feedback_generator: DefaultFeedbackGenerator::new(),
        }
    }
}

impl Default for SimpleEntropyCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl EntropyCalculator for SimpleEntropyCalculator {
    fn calculate_entropy(&self, guess: &Word, possible_words: &[Word]) -> f64 {
        if possible_words.is_empty() {
            return 0.0;
        }

        let mut groups = HashMap::new();

        for word in possible_words {
            let feedback = self.feedback_generator.generate_feedback(guess, word);
            *groups.entry(feedback).or_insert(0) += 1;
        }

        let total_words = possible_words.len() as f64;
        groups
            .values()
            .map(|&count| {
                let probability = count as f64 / total_words;
                if probability > 0.0 {
                    -probability * probability.log2()
                } else {
                    0.0
                }
            })
            .sum()
    }

    fn calculate_information_gain(&self, guess: &Word, possible_words: &[Word]) -> f64 {
        if possible_words.is_empty() {
            return 0.0;
        }

        let entropy = self.calculate_entropy(guess, possible_words);
        // Convert entropy to information gain approximation
        let max_possible_entropy = (possible_words.len() as f64).log2();
        max_possible_entropy - entropy
    }

    fn find_max_entropy_guess(&self, candidates: &[Word], possible_words: &[Word]) -> Option<Word> {
        if candidates.is_empty() || possible_words.is_empty() {
            return None;
        }

        candidates
            .iter()
            .map(|word| {
                let entropy = self.calculate_entropy(word, possible_words);
                (word.clone(), entropy)
            })
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(word, _)| word)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entropy_calculation() {
        let calculator = SimpleEntropyCalculator::new();

        let guess = Word::from_str("adieu").unwrap();
        let possible_words = vec![
            Word::from_str("apple").unwrap(),
            Word::from_str("about").unwrap(),
            Word::from_str("bread").unwrap(),
        ];

        let entropy = calculator.calculate_entropy(&guess, &possible_words);
        assert!(entropy >= 0.0);

        // Entropy should be 0 for a single word
        let single_word = vec![Word::from_str("apple").unwrap()];
        let single_entropy = calculator.calculate_entropy(&guess, &single_word);
        assert_eq!(single_entropy, 0.0);
    }

    #[test]
    fn test_max_entropy_guess() {
        let calculator = SimpleEntropyCalculator::new();

        let candidates = vec![
            Word::from_str("adieu").unwrap(),
            Word::from_str("about").unwrap(),
        ];

        let possible_words = vec![
            Word::from_str("apple").unwrap(),
            Word::from_str("bread").unwrap(),
            Word::from_str("crane").unwrap(),
        ];

        let best_guess = calculator.find_max_entropy_guess(&candidates, &possible_words);
        assert!(best_guess.is_some());
    }
}
