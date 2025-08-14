use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

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

/// Newtype for a 5-letter ASCII-lowercase word with validation.
///
/// Performance note: we store both the original string and a cached
/// 5-byte array to avoid repeated conversions in hot paths (entropy/feedback).
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Word {
    s: String,
    b: [u8; 5],
}

impl Word {
    /// Fixed Wordle word length
    pub const LENGTH: usize = 5;

    /// Create a new Word with validation
    pub fn new(word: String) -> Result<Self, String> {
        if word.len() != Self::LENGTH {
            return Err(format!(
                "Word must be exactly 5 characters, got {}",
                word.len()
            ));
        }

        if !word.chars().all(|c| c.is_ascii_lowercase()) {
            return Err("Word must contain only lowercase ASCII letters".to_string());
        }

        // Safe: validated ASCII-lowercase and len == 5
        let mut arr = [0u8; Self::LENGTH];
        arr.copy_from_slice(word.as_bytes());
        Ok(Word { s: word, b: arr })
    }

    /// Create a Word from a string slice (convenience method)
    pub fn from_str(word: &str) -> Result<Self, String> {
        Self::new(word.to_lowercase())
    }

    /// Get the underlying string
    pub fn as_str(&self) -> &str {
        &self.s
    }

    /// Get the underlying 5 bytes (ASCII lowercase)
    #[inline]
    pub fn bytes(&self) -> &[u8; 5] {
        &self.b
    }

    /// Get the character at the specified position
    pub fn char_at(&self, position: usize) -> Option<char> {
        if position < Self::LENGTH {
            Some(self.b[position] as char)
        } else {
            None
        }
    }

    /// Get all characters as a vector
    pub fn chars(&self) -> Vec<char> {
        self.b.iter().map(|&u| u as char).collect()
    }
}

impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.s)
    }
}

impl AsRef<str> for Word {
    fn as_ref(&self) -> &str {
        &self.s
    }
}

impl std::fmt::Debug for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Word").field(&self.s).finish()
    }
}

impl Ord for Word {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.b.cmp(&other.b)
    }
}

impl PartialOrd for Word {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl FromStr for Word {
    type Err = String;

    /// Parse a Word from a string. Use `"apple".parse::<Word>()`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Word::from_str(s)
    }
}

impl TryFrom<&str> for Word {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Word::from_str(value)
    }
}

/// Fixed-size feedback pattern for a 5-letter Wordle guess.
///
/// Stored as a stack-allocated [Feedback; 5] to avoid heap allocations in hot paths
/// like entropy calculation and constraint checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FeedbackPattern([Feedback; 5]);

impl FeedbackPattern {
    /// Create from an exactly 5-length result code string (0/1/2)
    pub fn from_code_string(code: &str) -> Result<Self, String> {
        if code.len() != 5 {
            return Err("Code string must be exactly 5 characters".to_string());
        }

        let mut arr = [Feedback::Absent; 5];
        for (i, c) in code.chars().enumerate() {
            arr[i] =
                Feedback::from_code(c).ok_or_else(|| format!("Invalid feedback code: {}", c))?;
        }
        Ok(FeedbackPattern(arr))
    }

    /// Get the underlying feedback vector
    #[inline]
    pub fn as_slice(&self) -> &[Feedback] {
        &self.0[..]
    }

    /// Convert to code string
    pub fn to_code_string(&self) -> String {
        self.0.iter().map(|f| f.to_code()).collect()
    }

    /// Check if this pattern indicates a win (all correct)
    #[inline]
    pub fn is_win(&self) -> bool {
        self.0.iter().all(|&f| f == Feedback::Correct)
    }

    /// Get the feedback at a specific position
    #[inline]
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

    /// Construct directly from an array (no validation needed)
    #[inline]
    pub fn from_array(arr: [Feedback; 5]) -> Self {
        FeedbackPattern(arr)
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
