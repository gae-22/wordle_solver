use crate::core::{
    traits::ConstraintFilter,
    types::{Guess, Word},
};

/// Default implementation of constraint filtering
#[derive(Debug)]
pub struct DefaultConstraintFilter {
    feedback_generator: crate::domain::DefaultFeedbackGenerator,
}

impl DefaultConstraintFilter {
    pub fn new() -> Self {
        Self {
            feedback_generator: crate::domain::DefaultFeedbackGenerator::new(),
        }
    }
}

impl Default for DefaultConstraintFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl ConstraintFilter for DefaultConstraintFilter {
    fn filter_words(&self, words: &[Word], constraints: &[Guess]) -> Vec<Word> {
        words
            .iter()
            .filter(|word| self.satisfies_constraints(word, constraints))
            .cloned()
            .collect()
    }

    fn satisfies_constraints(&self, word: &Word, constraints: &[Guess]) -> bool {
        use crate::core::traits::FeedbackGenerator;
        self.feedback_generator.is_consistent(word, constraints)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::FeedbackPattern;

    #[test]
    fn test_constraint_filtering() {
        let filter = DefaultConstraintFilter::new();

        // Create some test words
        let words = vec![
            Word::from_str("apple").unwrap(),
            Word::from_str("about").unwrap(),
            Word::from_str("bread").unwrap(),
            Word::from_str("crane").unwrap(),
        ];

        // Create a constraint: guess "about" got feedback "20000" (only 'a' correct)
        let guess = Word::from_str("about").unwrap();
        let feedback = FeedbackPattern::from_code_string("20000").unwrap();
        let constraint = Guess::new(guess, feedback);

        let filtered = filter.filter_words(&words, &[constraint]);

        // Only "apple" should satisfy this constraint (starts with 'a', doesn't contain 'b', 'o', 'u', 't')
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].as_str(), "apple");
    }
}
