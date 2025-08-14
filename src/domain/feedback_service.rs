use crate::core::{
    traits::FeedbackGenerator,
    types::{Feedback, FeedbackPattern, Word},
};

/// Default implementation of feedback generation
#[derive(Debug)]
pub struct DefaultFeedbackGenerator;

impl DefaultFeedbackGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultFeedbackGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl FeedbackGenerator for DefaultFeedbackGenerator {
    fn generate_feedback(&self, guess: &Word, target: &Word) -> FeedbackPattern {
        let gb = guess.bytes();
        let tb = target.bytes();
        let mut out = [Feedback::Absent; 5];
        let mut used = [false; 5];

        // Greens
        for i in 0..5 {
            if gb[i] == tb[i] {
                out[i] = Feedback::Correct;
                used[i] = true;
            }
        }
        // Yellows
        for i in 0..5 {
            if matches!(out[i], Feedback::Absent) {
                for j in 0..5 {
                    if !used[j] && gb[i] == tb[j] {
                        out[i] = Feedback::Present;
                        used[j] = true;
                        break;
                    }
                }
            }
        }
        FeedbackPattern::from_array(out)
    }

    fn is_consistent(&self, word: &Word, constraints: &[crate::core::types::Guess]) -> bool {
        for constraint in constraints {
            let expected_feedback = self.generate_feedback(&constraint.word, word);
            if expected_feedback != constraint.feedback {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feedback_generation() {
        let generator = DefaultFeedbackGenerator::new();

        let guess = Word::from_str("about").unwrap();
        let target = Word::from_str("apple").unwrap();

        let feedback = generator.generate_feedback(&guess, &target);

        // 'a' should be correct (position 0)
        // 'b', 'o', 'u', 't' should be absent
        assert_eq!(feedback.get(0), Some(Feedback::Correct));
        assert_eq!(feedback.get(1), Some(Feedback::Absent));
        assert_eq!(feedback.get(2), Some(Feedback::Absent));
        assert_eq!(feedback.get(3), Some(Feedback::Absent));
        assert_eq!(feedback.get(4), Some(Feedback::Absent));
    }

    #[test]
    fn test_consistency_check() {
        let generator = DefaultFeedbackGenerator::new();

        let guess = Word::from_str("about").unwrap();
        let target = Word::from_str("apple").unwrap();
        let feedback = generator.generate_feedback(&guess, &target);

        let constraint = crate::core::types::Guess::new(guess, feedback);

        // Target should be consistent with the constraint
        assert!(generator.is_consistent(&target, &[constraint.clone()]));

        // A different word should not be consistent
        let different_word = Word::from_str("bread").unwrap();
        assert!(!generator.is_consistent(&different_word, &[constraint]));
    }
}
