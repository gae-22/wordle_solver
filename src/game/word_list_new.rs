use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Configuration for word list URLs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordListConfig {
    /// URLs for answer words (possible Wordle answers)
    pub answers: Vec<String>,
    /// URLs for guess words (all valid guesses)
    pub guesses: Vec<String>,
}

impl Default for WordListConfig {
    fn default() -> Self {
        Self {
            answers: vec![
                "https://raw.githubusercontent.com/3b1b/videos/master/_2022/wordle/data/possible_words.txt".to_string(),
                "https://raw.githubusercontent.com/charlesreid1/five-letter-words/master/sgb-words.txt".to_string(),
            ],
            guesses: vec![
                "https://raw.githubusercontent.com/3b1b/videos/master/_2022/wordle/data/allowed_words.txt".to_string(),
                "https://raw.githubusercontent.com/charlesreid1/five-letter-words/master/sgb-words.txt".to_string(),
            ],
        }
    }
}

/// Cached word lists stored in JSON format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordListCache {
    /// All valid 5-letter words that can be answers
    pub answer_words: Vec<String>,
    /// All valid 5-letter words that can be guessed (includes answer_words)
    pub guess_words: Vec<String>,
    /// Timestamp when the cache was last updated
    pub last_updated: u64,
}

/// Word list management for Wordle with online fetching capability
#[derive(Debug, Clone)]
pub struct WordList {
    /// All valid 5-letter words that can be answers
    answer_words: Vec<String>,
    /// All valid 5-letter words that can be guessed (includes answer_words)
    guess_words: HashSet<String>,
}

impl WordList {
    /// Create a new WordList by downloading from URLs and caching locally
    pub async fn new() -> Result<Self> {
        Self::new_with_cache_path("word_lists.json").await
    }

    /// Create a new WordList with custom cache file path
    pub async fn new_with_cache_path(cache_path: &str) -> Result<Self> {
        let config = WordListConfig::default();

        // Try to load from cache first
        if let Ok(cache) = Self::load_cache(cache_path).await {
            log::info!(
                "Loaded word lists from cache: {} answer words, {} guess words",
                cache.answer_words.len(),
                cache.guess_words.len()
            );
            return Self::from_cache(cache);
        }

        log::info!("Cache not found or invalid, downloading word lists from URLs...");

        // Download fresh data
        let (answer_words, guess_words) = Self::download_word_lists(&config).await?;

        // Save to cache
        Self::save_cache(cache_path, &answer_words, &guess_words).await?;

        log::info!(
            "Downloaded and cached word lists: {} answer words, {} guess words",
            answer_words.len(),
            guess_words.len()
        );

        Ok(Self {
            answer_words,
            guess_words,
        })
    }

    /// Download word lists from configured URLs
    async fn download_word_lists(
        config: &WordListConfig,
    ) -> Result<(Vec<String>, HashSet<String>)> {
        let client = reqwest::Client::new();
        let mut answer_words = HashSet::new();
        let mut guess_words = HashSet::new();

        // Download answer words
        for url in &config.answers {
            log::info!("Downloading answer words from: {}", url);
            let response = client
                .get(url)
                .send()
                .await
                .with_context(|| format!("Failed to fetch answer words from {}", url))?;

            let text = response
                .text()
                .await
                .with_context(|| format!("Failed to read response from {}", url))?;

            for line in text.lines() {
                let word = line.trim().to_lowercase();
                if word.len() == 5 && word.chars().all(|c| c.is_ascii_lowercase()) {
                    answer_words.insert(word);
                }
            }
        }

        // Download guess words
        for url in &config.guesses {
            log::info!("Downloading guess words from: {}", url);
            let response = client
                .get(url)
                .send()
                .await
                .with_context(|| format!("Failed to fetch guess words from {}", url))?;

            let text = response
                .text()
                .await
                .with_context(|| format!("Failed to read response from {}", url))?;

            for line in text.lines() {
                let word = line.trim().to_lowercase();
                if word.len() == 5 && word.chars().all(|c| c.is_ascii_lowercase()) {
                    guess_words.insert(word);
                }
            }
        }

        // Ensure all answer words are also valid guess words
        for word in &answer_words {
            guess_words.insert(word.clone());
        }

        let answer_words: Vec<String> = answer_words.into_iter().collect();

        Ok((answer_words, guess_words))
    }

    /// Load word lists from cache file
    async fn load_cache(cache_path: &str) -> Result<WordListCache> {
        if !Path::new(cache_path).exists() {
            return Err(anyhow::anyhow!("Cache file does not exist"));
        }

        let content = fs::read_to_string(cache_path)
            .with_context(|| format!("Failed to read cache file: {}", cache_path))?;

        let cache: WordListCache = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse cache file: {}", cache_path))?;

        // Check if cache is relatively fresh (less than 24 hours old)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        if now - cache.last_updated > 24 * 60 * 60 {
            return Err(anyhow::anyhow!("Cache is too old, will refresh"));
        }

        Ok(cache)
    }

    /// Save word lists to cache file
    async fn save_cache(
        cache_path: &str,
        answer_words: &[String],
        guess_words: &HashSet<String>,
    ) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        let cache = WordListCache {
            answer_words: answer_words.to_vec(),
            guess_words: guess_words.iter().cloned().collect(),
            last_updated: now,
        };

        let json =
            serde_json::to_string_pretty(&cache).context("Failed to serialize word list cache")?;

        fs::write(cache_path, json)
            .with_context(|| format!("Failed to write cache file: {}", cache_path))?;

        Ok(())
    }

    /// Create WordList from cached data
    fn from_cache(cache: WordListCache) -> Result<Self> {
        let guess_words: HashSet<String> = cache.guess_words.into_iter().collect();

        Ok(Self {
            answer_words: cache.answer_words,
            guess_words,
        })
    }

    /// Get all possible answer words
    pub fn get_answer_words(&self) -> &[String] {
        &self.answer_words
    }

    /// Check if a word is a valid guess
    pub fn is_valid_guess(&self, word: &str) -> bool {
        if word.len() != 5 {
            return false;
        }
        self.guess_words.contains(&word.to_lowercase())
    }

    /// Check if a word is a possible answer
    pub fn is_possible_answer(&self, word: &str) -> bool {
        if word.len() != 5 {
            return false;
        }
        self.answer_words.contains(&word.to_lowercase())
    }

    /// Get the number of possible answer words
    pub fn answer_count(&self) -> usize {
        self.answer_words.len()
    }

    /// Get the number of valid guess words
    pub fn guess_count(&self) -> usize {
        self.guess_words.len()
    }

    /// Force refresh the word lists from URLs
    pub async fn refresh(&mut self) -> Result<()> {
        self.refresh_with_cache_path("word_lists.json").await
    }

    /// Force refresh with custom cache path
    pub async fn refresh_with_cache_path(&mut self, cache_path: &str) -> Result<()> {
        let config = WordListConfig::default();
        let (answer_words, guess_words) = Self::download_word_lists(&config).await?;

        Self::save_cache(cache_path, &answer_words, &guess_words).await?;

        self.answer_words = answer_words;
        self.guess_words = guess_words;

        log::info!(
            "Refreshed word lists: {} answer words, {} guess words",
            self.answer_words.len(),
            self.guess_words.len()
        );

        Ok(())
    }
}
