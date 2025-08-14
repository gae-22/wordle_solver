use crate::core::{
    traits::{EntropyCalculator, FeedbackGenerator},
    types::Word,
};
use crate::domain::DefaultFeedbackGenerator;
use std::collections::HashMap;

/// High-performance entropy calculator with caching
#[derive(Debug)]
pub struct CachedEntropyCalculator {}

impl CachedEntropyCalculator {
    pub fn new() -> Self {
        Self {}
    }

    /// Compute a compact feedback index (0..243) without allocations.
    /// Encodes feedback as base-3 digits (0=Absent,1=Present,2=Correct) with position-weighted digits.
    #[inline]
    fn feedback_index_bytes(&self, guess_b: &[u8; 5], target_b: &[u8; 5]) -> usize {
        let mut used = [false; 5];
        let mut f = [0u8; 5];

        // Greens
        for i in 0..5 {
            if guess_b[i] == target_b[i] {
                f[i] = 2;
                used[i] = true;
            }
        }
        // Yellows
        for i in 0..5 {
            if f[i] == 0 {
                let g = guess_b[i];
                for j in 0..5 {
                    if !used[j] && g == target_b[j] {
                        f[i] = 1;
                        used[j] = true;
                        break;
                    }
                }
            }
        }
        // Encode base-3 little-endian
        (f[0] as usize)
            + (f[1] as usize) * 3
            + (f[2] as usize) * 9
            + (f[3] as usize) * 27
            + (f[4] as usize) * 81
    }

    #[inline]
    fn feedback_index(&self, guess: &Word, target: &Word) -> usize {
        let gb = guess.as_str().as_bytes();
        let tb = target.as_str().as_bytes();
        // Safety: words are validated 5-letter ASCII lowercase
        let gb5: &[u8; 5] = gb.try_into().expect("word length must be 5");
        let tb5: &[u8; 5] = tb.try_into().expect("word length must be 5");
        self.feedback_index_bytes(gb5, tb5)
    }

    /// Calculate information gain based on expected partition sizes
    fn calculate_information_gain_internal(&self, guess: &Word, possible_words: &[Word]) -> f64 {
        if possible_words.is_empty() {
            return 0.0;
        }
        // Count into 243 buckets
        let mut counts = [0usize; 243];
        for w in possible_words {
            let idx = self.feedback_index(guess, w);
            counts[idx] += 1;
        }
        let total = possible_words.len() as f64;
        let expected_log_size: f64 = counts
            .iter()
            .filter(|&&c| c > 0)
            .map(|&c| {
                let p = c as f64 / total;
                let log_size = (c as f64).log2();
                p * log_size
            })
            .sum();
        total.log2() - expected_log_size
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
            return 0.0;
        }
        let mut counts = [0usize; 243];
        for w in possible_words {
            let idx = self.feedback_index(guess, w);
            counts[idx] += 1;
        }
        let total = possible_words.len() as f64;
        counts
            .iter()
            .filter(|&&c| c > 0)
            .map(|&c| {
                let p = c as f64 / total;
                -p * p.log2()
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
