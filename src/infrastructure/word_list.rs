use crate::core::{
    error::{DataError, Result},
    traits::WordListProvider,
    types::Word,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Configuration for word list sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordListConfig {
    pub answers: Vec<String>,
    pub guesses: Vec<String>,
}

impl Default for WordListConfig {
    fn default() -> Self {
        Self {
            answers: vec![
                "https://raw.githubusercontent.com/3b1b/videos/master/_2022/wordle/data/possible_words.txt".to_string(),
                // Community-maintained list of NYT Wordle possible answers (frequently updated)
                "https://raw.githubusercontent.com/tabatkins/wordle-list/main/words".to_string(),
            ],
            guesses: vec![
                "https://raw.githubusercontent.com/3b1b/videos/master/_2022/wordle/data/allowed_words.txt".to_string(),
                // Large list of common English words (includes many valid guesses)
                "https://raw.githubusercontent.com/dwyl/english-words/master/words_alpha.txt".to_string(),
            ],
        }
    }
}

/// Cached word lists
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordListCache {
    pub answer_words: Vec<String>,
    pub guess_words: Vec<String>,
    pub last_updated: u64,
}

/// File-based word list provider with online fetching
#[derive(Debug)]
pub struct FileWordListProvider {
    answer_words: Vec<Word>,
    guess_words: Vec<Word>,
    cache_path: String,
    config: WordListConfig,
}

impl FileWordListProvider {
    /// Get the default word lists cache path in the project root
    fn get_default_cache_path() -> String {
        // Try to find the project root by looking for Cargo.toml
        // Start from the executable's directory and work upwards
        let exe_path = std::env::current_exe()
            .unwrap_or_else(|_| PathBuf::from("."))
            .parent()
            .unwrap_or(&PathBuf::from("."))
            .to_path_buf();

        let mut current_dir = exe_path;

        // Look up the directory tree for Cargo.toml
        loop {
            let cargo_toml = current_dir.join("Cargo.toml");
            if cargo_toml.exists() {
                let cache_path = current_dir.join("word_lists.json");
                return cache_path.to_string_lossy().to_string();
            }

            if let Some(parent) = current_dir.parent() {
                current_dir = parent.to_path_buf();
            } else {
                break;
            }
        }

        // Fallback: try current working directory
        let mut current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        loop {
            let cargo_toml = current_dir.join("Cargo.toml");
            if cargo_toml.exists() {
                let cache_path = current_dir.join("word_lists.json");
                return cache_path.to_string_lossy().to_string();
            }

            if let Some(parent) = current_dir.parent() {
                current_dir = parent.to_path_buf();
            } else {
                // Final fallback to current directory
                return "word_lists.json".to_string();
            }
        }
    }

    pub fn new() -> Self {
        let cache_path = Self::get_default_cache_path();
        Self::with_cache_path(cache_path)
    }

    pub fn with_path(cache_path: String) -> Self {
        Self::with_cache_path(cache_path)
    }

    pub fn with_cache_path(cache_path: String) -> Self {
        let mut this = Self {
            answer_words: Vec::new(),
            guess_words: Vec::new(),
            cache_path,
            config: WordListConfig::default(),
        };
        // Load optional source override config if present
        if let Some(cfg) = this.load_config_override() {
            this.config = cfg;
        }
        this
    }

    pub fn with_config(config: WordListConfig) -> Self {
        Self {
            answer_words: Vec::new(),
            guess_words: Vec::new(),
            cache_path: Self::get_default_cache_path(),
            config,
        }
    }

    /// Returns the default path for an optional sources override file in the project root
    fn get_default_sources_config_path() -> String {
        let mut current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        loop {
            let cargo_toml = current_dir.join("Cargo.toml");
            if cargo_toml.exists() {
                return current_dir
                    .join("word_sources.json")
                    .to_string_lossy()
                    .to_string();
            }
            if let Some(parent) = current_dir.parent() {
                current_dir = parent.to_path_buf();
            } else {
                return "word_sources.json".to_string();
            }
        }
    }

    /// Load configuration override from `word_sources.json` if it exists
    fn load_config_override(&self) -> Option<WordListConfig> {
        let path = Self::get_default_sources_config_path();
        let p = Path::new(&path);
        if !p.exists() {
            return None;
        }
        match std::fs::read_to_string(p) {
            Ok(s) => match serde_json::from_str::<WordListConfig>(&s) {
                Ok(cfg) => Some(cfg),
                Err(e) => {
                    log::warn!("Failed to parse word_sources.json: {}", e);
                    None
                }
            },
            Err(e) => {
                log::warn!("Failed to read word_sources.json: {}", e);
                None
            }
        }
    }

    async fn load_from_cache(&self) -> Result<WordListCache> {
        if !Path::new(&self.cache_path).exists() {
            return Err(DataError::MissingData("Cache file not found".to_string()).into());
        }

        let content = tokio::fs::read_to_string(&self.cache_path)
            .await
            .map_err(DataError::from)?;

        let cache: WordListCache = serde_json::from_str(&content).map_err(DataError::from)?;

        // Check if cache is fresh (less than 24 hours)
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| DataError::InvalidFormat("System time error".to_string()))?
            .as_secs();

        if now - cache.last_updated > 24 * 60 * 60 {
            return Err(DataError::InvalidFormat("Cache too old".to_string()).into());
        }

        Ok(cache)
    }

    /// Load cache file ignoring staleness checks (best-effort fallback)
    async fn load_cache_unchecked(&self) -> Result<WordListCache> {
        if !Path::new(&self.cache_path).exists() {
            return Err(DataError::MissingData("Cache file not found".to_string()).into());
        }
        let content = tokio::fs::read_to_string(&self.cache_path)
            .await
            .map_err(DataError::from)?;
        let cache: WordListCache = serde_json::from_str(&content).map_err(DataError::from)?;
        Ok(cache)
    }

    async fn download_words(&self) -> Result<(Vec<String>, Vec<String>)> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(20))
            .build()
            .map_err(|e| DataError::InvalidFormat(format!("HTTP client error: {}", e)))?;
        let mut answer_words = HashSet::new();
        let mut guess_words = HashSet::new();

        // Download answer words
        for url in &self.config.answers {
            log::info!("Downloading answer words from: {}", url);
            let response = client.get(url).send().await.map_err(|e| {
                DataError::InvalidFormat(format!("HTTP error fetching {}: {}", url, e))
            })?;

            let text = response
                .text()
                .await
                .map_err(|e| DataError::InvalidFormat(format!("Response error: {}", e)))?;

            for line in text.lines() {
                let word = line.trim().to_lowercase();
                if word.len() == 5 && word.chars().all(|c| c.is_ascii_lowercase()) {
                    answer_words.insert(word);
                }
            }
        }

        // Download guess words
        for url in &self.config.guesses {
            log::info!("Downloading guess words from: {}", url);
            let response = client.get(url).send().await.map_err(|e| {
                DataError::InvalidFormat(format!("HTTP error fetching {}: {}", url, e))
            })?;

            let text = response
                .text()
                .await
                .map_err(|e| DataError::InvalidFormat(format!("Response error: {}", e)))?;

            for line in text.lines() {
                let word = line.trim().to_lowercase();
                if word.len() == 5 && word.chars().all(|c| c.is_ascii_lowercase()) {
                    guess_words.insert(word);
                }
            }
        }

        // Ensure all answer words are valid guesses
        for word in &answer_words {
            guess_words.insert(word.clone());
        }

        Ok((
            answer_words.into_iter().collect(),
            guess_words.into_iter().collect(),
        ))
    }

    async fn save_to_cache(&self, answer_words: &[String], guess_words: &[String]) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| DataError::InvalidFormat("System time error".to_string()))?
            .as_secs();

        let cache = WordListCache {
            answer_words: answer_words.to_vec(),
            guess_words: guess_words.to_vec(),
            last_updated: now,
        };

        let json = serde_json::to_string_pretty(&cache).map_err(DataError::from)?;

        tokio::fs::write(&self.cache_path, json)
            .await
            .map_err(DataError::from)?;

        Ok(())
    }

    fn convert_to_words(strings: Vec<String>) -> Result<Vec<Word>> {
        strings
            .into_iter()
            .map(|s| Word::from_str(&s).map_err(|e| DataError::InvalidFormat(e).into()))
            .collect()
    }

    /// Get the path to the word list cache file
    pub fn cache_path(&self) -> &str {
        &self.cache_path
    }

    /// Force refresh the word lists cache from remote sources.
    /// When `force` is false, it will skip if the cache is still fresh.
    pub async fn refresh_cache(&mut self, force: bool) -> Result<(usize, usize)> {
        let fresh = match self.load_from_cache().await {
            Ok(_) => true,
            Err(_) => false,
        };
        if fresh && !force {
            log::info!("Cache is fresh; skipping refresh");
            let cache = self.load_from_cache().await?;
            self.answer_words = Self::convert_to_words(cache.answer_words.clone())?;
            self.guess_words = Self::convert_to_words(cache.guess_words.clone())?;
            return Ok((self.answer_words.len(), self.guess_words.len()));
        }

        let (answer_strings, guess_strings) = self.download_words().await?;
        self.save_to_cache(&answer_strings, &guess_strings).await?;
        self.answer_words = Self::convert_to_words(answer_strings.clone())?;
        self.guess_words = Self::convert_to_words(guess_strings.clone())?;
        Ok((self.answer_words.len(), self.guess_words.len()))
    }
}

impl Default for FileWordListProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WordListProvider for FileWordListProvider {
    async fn load_words(&mut self) -> Result<Vec<Word>> {
        // Try loading from cache first
        if let Ok(cache) = self.load_from_cache().await {
            log::info!("Loaded word lists from cache");
            self.answer_words = Self::convert_to_words(cache.answer_words)?;
            self.guess_words = Self::convert_to_words(cache.guess_words)?;
        } else {
            log::info!("Downloading fresh word lists");
            match self.download_words().await {
                Ok((answer_strings, guess_strings)) => {
                    self.answer_words = Self::convert_to_words(answer_strings.clone())?;
                    self.guess_words = Self::convert_to_words(guess_strings.clone())?;
                    // Save to cache
                    self.save_to_cache(&answer_strings, &guess_strings).await?;
                }
                Err(e) => {
                    log::warn!(
                        "Download failed: {}. Falling back to stale cache if available.",
                        e
                    );
                    // Try stale cache as a last resort
                    let cache = self.load_cache_unchecked().await?;
                    self.answer_words = Self::convert_to_words(cache.answer_words)?;
                    self.guess_words = Self::convert_to_words(cache.guess_words)?;
                }
            }
        }

        let mut all_words = self.answer_words.clone();
        all_words.extend(self.guess_words.clone());
        all_words.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        all_words.dedup();

        Ok(all_words)
    }

    fn get_answer_words(&self) -> &[Word] {
        &self.answer_words
    }

    fn get_guess_words(&self) -> &[Word] {
        &self.guess_words
    }

    fn is_valid_guess(&self, word: &Word) -> bool {
        self.guess_words.contains(word) || self.answer_words.contains(word)
    }

    fn is_possible_answer(&self, word: &Word) -> bool {
        self.answer_words.contains(word)
    }

    async fn refresh(&mut self, force: bool) -> Result<(usize, usize)> {
        self.refresh_cache(force).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_word_list_provider_creation() {
        let provider = FileWordListProvider::new();
        // The cache path should now point to the project root
        assert!(provider.cache_path.ends_with("word_lists.json"));
        assert_eq!(provider.answer_words.len(), 0);
        assert_eq!(provider.guess_words.len(), 0);
    }
}
