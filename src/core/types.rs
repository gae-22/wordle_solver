use std::fmt;

/// Feedback types for Wordle guesses with enhanced type safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Feedback {
    /// Correct letter in correct position (Green/2)
    Correct,
    /// Correct letter in wrong position (Yellow/1)
    Present,
    /// Letter not in word (Gray/0)
    Absent,
}

impl Feedback {
    /// Convert from result code character to Feedback
    pub fn from_code(code: char) -> Option<Self> {
        match code {
            '2' => Some(Feedback::Correct),
            '1' => Some(Feedback::Present),
            '0' => Some(Feedback::Absent),
            _ => None,
        }
    }

    /// Convert Feedback to result code character
    pub fn to_code(self) -> char {
        match self {
            Feedback::Correct => '2',
            Feedback::Present => '1',
            Feedback::Absent => '0',
        }
    }

    /// Get the priority score for this feedback (higher = better)
    pub fn priority_score(self) -> u8 {
        match self {
            Feedback::Correct => 2,
            Feedback::Present => 1,
            Feedback::Absent => 0,
        }
    }
}

impl fmt::Display for Feedback {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self {
            Feedback::Correct => "🟩",
            Feedback::Present => "🟨",
            Feedback::Absent => "⬜",
        };
        write!(f, "{}", symbol)
    }
}

/// Newtype for a 5-letter word with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Word(String);

impl Word {
    /// Create a new Word with validation
    pub fn new(word: String) -> Result<Self, String> {
        if word.len() != 5 {
            return Err(format!(
                "Word must be exactly 5 characters, got {}",
                word.len()
            ));
        }

        if !word.chars().all(|c| c.is_ascii_lowercase()) {
            return Err("Word must contain only lowercase ASCII letters".to_string());
        }

        Ok(Word(word))
    }

    /// Create a Word from a string slice (convenience method)
    pub fn from_str(word: &str) -> Result<Self, String> {
        Self::new(word.to_lowercase())
    }

    /// Get the underlying string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the character at the specified position
    pub fn char_at(&self, position: usize) -> Option<char> {
        self.0.chars().nth(position)
    }

    /// Get all characters as a vector
    pub fn chars(&self) -> Vec<char> {
        self.0.chars().collect()
    }
}

impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Word {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Newtype for feedback pattern with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FeedbackPattern(Vec<Feedback>);

impl FeedbackPattern {
    /// Create a new FeedbackPattern with validation
    pub fn new(feedback: Vec<Feedback>) -> Result<Self, String> {
        if feedback.len() != 5 {
            return Err(format!(
                "Feedback pattern must be exactly 5 elements, got {}",
                feedback.len()
            ));
        }
        Ok(FeedbackPattern(feedback))
    }

    /// Create from a result code string
    pub fn from_code_string(code: &str) -> Result<Self, String> {
        if code.len() != 5 {
            return Err("Code string must be exactly 5 characters".to_string());
        }

        let feedback: Result<Vec<_>, _> = code
            .chars()
            .map(|c| Feedback::from_code(c).ok_or_else(|| format!("Invalid feedback code: {}", c)))
            .collect();

        feedback.map(|f| FeedbackPattern(f))
    }

    /// Get the underlying feedback vector
    pub fn as_slice(&self) -> &[Feedback] {
        &self.0
    }

    /// Convert to code string
    pub fn to_code_string(&self) -> String {
        self.0.iter().map(|f| f.to_code()).collect()
    }

    /// Check if this pattern indicates a win (all correct)
    pub fn is_win(&self) -> bool {
        self.0.iter().all(|&f| f == Feedback::Correct)
    }

    /// Get the feedback at a specific position
    pub fn get(&self, index: usize) -> Option<Feedback> {
        self.0.get(index).copied()
    }

    /// Calculate the information content of this pattern
    pub fn information_content(&self) -> f64 {
        let correct_count = self.0.iter().filter(|&&f| f == Feedback::Correct).count();
        let present_count = self.0.iter().filter(|&&f| f == Feedback::Present).count();

        // Higher scores for more informative patterns
        (correct_count * 2 + present_count) as f64
    }
}

impl fmt::Display for FeedbackPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for feedback in &self.0 {
            write!(f, "{}", feedback)?;
        }
        Ok(())
    }
}

/// Represents a guess with its resulting feedback
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Guess {
    pub word: Word,
    pub feedback: FeedbackPattern,
}

impl Guess {
    pub fn new(word: Word, feedback: FeedbackPattern) -> Self {
        Self { word, feedback }
    }

    /// Check if this guess resulted in a win
    pub fn is_winning(&self) -> bool {
        self.feedback.is_win()
    }
}

/// Game result enumeration with more detailed information
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameResult {
    /// Game is still in progress
    InProgress,
    /// Game is won with the solution word and number of attempts
    Won { word: Word, attempts: usize },
    /// Game failed (no solution found or too many attempts)
    Failed { attempts: usize, reason: String },
}

impl GameResult {
    pub fn is_finished(&self) -> bool {
        !matches!(self, GameResult::InProgress)
    }

    pub fn is_won(&self) -> bool {
        matches!(self, GameResult::Won { .. })
    }
}

/// Statistics for solver performance
#[derive(Debug, Clone)]
pub struct SolverStatistics {
    pub total_guesses: usize,
    pub remaining_words: usize,
    pub is_solved: bool,
    pub possible_words_sample: Vec<Word>,
    pub entropy_scores: Vec<(Word, f64)>,
}

impl SolverStatistics {
    pub fn new() -> Self {
        Self {
            total_guesses: 0,
            remaining_words: 0,
            is_solved: false,
            possible_words_sample: Vec::new(),
            entropy_scores: Vec::new(),
        }
    }
}

impl Default for SolverStatistics {
    fn default() -> Self {
        Self::new()
    }
}
