use crate::solver::feedback::{Feedback, generate_feedback};
use std::collections::HashMap;

/// Calculate entropy for word selection
pub struct EntropyCalculator {
    /// Cache for entropy calculations
    entropy_cache: HashMap<String, f64>,
}

impl EntropyCalculator {
    pub fn new() -> Self {
        Self {
            entropy_cache: HashMap::new(),
        }
    }

    /// Calculate the entropy of a guess against remaining possible words
    pub fn calculate_entropy(&mut self, guess: &str, possible_words: &[String]) -> f64 {
        if possible_words.is_empty() {
            return 0.0;
        }

        // Create cache key
        let cache_key = format!("{}:{}", guess, possible_words.len());
        if let Some(&cached_entropy) = self.entropy_cache.get(&cache_key) {
            return cached_entropy;
        }

        // Group possible words by their feedback pattern
        let mut feedback_groups: HashMap<Vec<Feedback>, usize> = HashMap::new();

        for target in possible_words {
            let feedback_pattern = generate_feedback(guess, target);
            *feedback_groups.entry(feedback_pattern).or_insert(0) += 1;
        }

        // Calculate entropy: -Σ(p * log2(p))
        let total_words = possible_words.len() as f64;
        let entropy = feedback_groups
            .values()
            .map(|&count| {
                let probability = count as f64 / total_words;
                -probability * probability.log2()
            })
            .sum();

        // Cache the result
        self.entropy_cache.insert(cache_key, entropy);
        entropy
    }

    /// Find the word with maximum entropy from a list of candidates
    pub fn find_best_guess(
        &mut self,
        candidates: &[String],
        possible_words: &[String],
    ) -> Option<String> {
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

    /// Calculate expected remaining words after a guess
    pub fn expected_remaining_words(&mut self, guess: &str, possible_words: &[String]) -> f64 {
        if possible_words.is_empty() {
            return 0.0;
        }

        let mut feedback_groups: HashMap<Vec<Feedback>, Vec<String>> = HashMap::new();

        for target in possible_words {
            let feedback_pattern = generate_feedback(guess, target);
            feedback_groups
                .entry(feedback_pattern)
                .or_insert_with(Vec::new)
                .push(target.clone());
        }

        let total_words = possible_words.len() as f64;
        feedback_groups
            .values()
            .map(|group| {
                let probability = group.len() as f64 / total_words;
                probability * group.len() as f64
            })
            .sum()
    }

    /// Clear the entropy cache
    pub fn clear_cache(&mut self) {
        self.entropy_cache.clear();
    }

    /// Get top candidates sorted by entropy (highest first)
    pub fn get_top_candidates(
        &mut self,
        candidates: &[String],
        possible_words: &[String],
        limit: usize,
    ) -> Vec<(String, f64)> {
        if candidates.is_empty() || possible_words.is_empty() {
            return Vec::new();
        }

        let mut candidates_with_entropy: Vec<(String, f64)> = candidates
            .iter()
            .map(|word| {
                let entropy = self.calculate_entropy(word, possible_words);
                (word.clone(), entropy)
            })
            .collect();

        // Sort by entropy (highest first)
        candidates_with_entropy
            .sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

        // Return top N candidates
        candidates_with_entropy.into_iter().take(limit).collect()
    }
}
