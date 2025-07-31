/// Utility functions and helpers for the Wordle solver

/// Validate that a word is exactly 5 characters and contains only letters
pub fn is_valid_word_format(word: &str) -> bool {
    word.len() == 5 && word.chars().all(|c| c.is_ascii_alphabetic())
}

/// Convert a word to lowercase and validate it
pub fn normalize_word(word: &str) -> Result<String, String> {
    let normalized = word.to_lowercase();
    if is_valid_word_format(&normalized) {
        Ok(normalized)
    } else {
        Err(format!(
            "Invalid word format: '{}'. Must be exactly 5 letters.",
            word
        ))
    }
}

/// Format a word for display (uppercase)
pub fn format_word_display(word: &str) -> String {
    word.to_uppercase()
}

/// Parse command line input in the format "word result"
pub fn parse_guess_input(input: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();

    if parts.len() != 2 {
        return Err("Input must be in format 'word result' (e.g., 'adieu 20100')".to_string());
    }

    let word = normalize_word(parts[0])?;
    let result = parts[1].to_string();

    if result.len() != 5 {
        return Err("Result code must be exactly 5 digits".to_string());
    }

    if !result.chars().all(|c| matches!(c, '0' | '1' | '2')) {
        return Err("Result code must contain only 0, 1, or 2".to_string());
    }

    Ok((word, result))
}

/// Calculate character frequency in a list of words
pub fn calculate_char_frequency(words: &[String]) -> std::collections::HashMap<char, usize> {
    let mut frequency = std::collections::HashMap::new();

    for word in words {
        for ch in word.chars() {
            *frequency.entry(ch).or_insert(0) += 1;
        }
    }

    frequency
}

/// Get unique characters in a word
pub fn get_unique_chars(word: &str) -> std::collections::HashSet<char> {
    word.chars().collect()
}

/// Check if two words share any characters
pub fn words_share_chars(word1: &str, word2: &str) -> bool {
    let chars1: std::collections::HashSet<char> = word1.chars().collect();
    let chars2: std::collections::HashSet<char> = word2.chars().collect();

    !chars1.is_disjoint(&chars2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_word_format() {
        assert!(is_valid_word_format("hello"));
        assert!(is_valid_word_format("WORLD"));
        assert!(!is_valid_word_format("hell")); // too short
        assert!(!is_valid_word_format("helloo")); // too long
        assert!(!is_valid_word_format("hell0")); // contains digit
        assert!(!is_valid_word_format("hel lo")); // contains space
    }

    #[test]
    fn test_normalize_word() {
        assert_eq!(normalize_word("HELLO"), Ok("hello".to_string()));
        assert_eq!(normalize_word("World"), Ok("world".to_string()));
        assert!(normalize_word("hi").is_err()); // too short
        assert!(normalize_word("hello!").is_err()); // invalid character
    }

    #[test]
    fn test_parse_guess_input() {
        assert_eq!(
            parse_guess_input("adieu 20100"),
            Ok(("adieu".to_string(), "20100".to_string()))
        );
        assert_eq!(
            parse_guess_input("WORLD 12021"),
            Ok(("world".to_string(), "12021".to_string()))
        );
        assert!(parse_guess_input("adieu").is_err()); // missing result
        assert!(parse_guess_input("adieu 2010").is_err()); // result too short
        assert!(parse_guess_input("adieu 20103").is_err()); // invalid result code
    }
}
