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
        let guess_chars = guess.chars();
        let target_chars = target.chars();
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

        FeedbackPattern::new(feedback).expect("Generated feedback should be valid")
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
