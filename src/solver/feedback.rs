/// Feedback types for Wordle guesses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
}

/// Parse a result string into feedback vector
pub fn parse_feedback(result: &str) -> Result<Vec<Feedback>, String> {
    if result.len() != 5 {
        return Err("Result must be exactly 5 characters".to_string());
    }

    result
        .chars()
        .map(|c| Feedback::from_code(c).ok_or_else(|| format!("Invalid feedback code: {}", c)))
        .collect()
}

/// Generate feedback pattern between guess and target
pub fn generate_feedback(guess: &str, target: &str) -> Vec<Feedback> {
    if guess.len() != 5 || target.len() != 5 {
        return vec![Feedback::Absent; 5];
    }

    let guess_chars: Vec<char> = guess.chars().collect();
    let target_chars: Vec<char> = target.chars().collect();
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

    feedback
}

/// Convert feedback vector to result string
pub fn feedback_to_string(feedback: &[Feedback]) -> String {
    feedback.iter().map(|f| f.to_code()).collect()
}
