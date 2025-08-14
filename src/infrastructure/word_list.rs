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
            // Default to a single comprehensive English word list (unfiltered)
            // Reference: https://raw.githubusercontent.com/dwyl/english-words/master/words_alpha.txt
            // Note: We apply a 5-letter lowercase filter during download to fit Wordle rules.
            answers: vec![
                "https://raw.githubusercontent.com/dwyl/english-words/master/words_alpha.txt"
                    .to_string(),
            ],
            guesses: vec![
                "https://raw.githubusercontent.com/dwyl/english-words/master/words_alpha.txt"
                    .to_string(),
            ],
        }
    }
}

/// Frequency data for initial heuristics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FrequencyData {
    #[serde(default)]
    pub letter_counts: [u32; 26],
    #[serde(default)]
    pub position_counts: [[u32; 26]; 5],
    /// Bigram counts for adjacent pairs across positions 0-3 (pairs 0-1,1-2,2-3,3-4)
    #[serde(default)]
    pub bigram_counts: [[[u32; 26]; 26]; 4],
}

/// Cached word lists (JSON-compatible in-memory shape)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordListCache {
    pub answer_words: Vec<String>,
    pub guess_words: Vec<String>,
    pub last_updated: u64,
    /// Optional frequency data (WLF2+)
    #[serde(default)]
    pub frequency: FrequencyData,
}

/// File-based word list provider with online fetching
#[derive(Debug)]
pub struct FileWordListProvider {
    answer_words: Vec<Word>,
    guess_words: Vec<Word>,
    cache_path: String,
    /// Path to compact binary cache optimized for Wordle lookups
    bin_cache_path: String,
    config: WordListConfig,
    frequency: Option<FrequencyData>,
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

    /// Get the default binary cache path in the project root
    fn get_default_bin_cache_path() -> String {
        // Mirror get_default_cache_path but use .wlf extension
        fn in_dir(dir: &Path) -> String {
            dir.join("word_lists.wlf").to_string_lossy().to_string()
        }
        // Try walking up from current exe
        let exe_path = std::env::current_exe()
            .unwrap_or_else(|_| PathBuf::from("."))
            .parent()
            .unwrap_or(&PathBuf::from("."))
            .to_path_buf();
        let mut current_dir = exe_path;
        loop {
            let cargo_toml = current_dir.join("Cargo.toml");
            if cargo_toml.exists() {
                return in_dir(&current_dir);
            }
            if let Some(parent) = current_dir.parent() {
                current_dir = parent.to_path_buf();
            } else {
                break;
            }
        }
        // Fallback to CWD
        let mut current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        loop {
            let cargo_toml = current_dir.join("Cargo.toml");
            if cargo_toml.exists() {
                return in_dir(&current_dir);
            }
            if let Some(parent) = current_dir.parent() {
                current_dir = parent.to_path_buf();
            } else {
                return "word_lists.wlf".to_string();
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
        let bin_cache_path = Self::get_default_bin_cache_path();
        let mut this = Self {
            answer_words: Vec::new(),
            guess_words: Vec::new(),
            cache_path,
            bin_cache_path,
            config: WordListConfig::default(),
            frequency: None,
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
            bin_cache_path: Self::get_default_bin_cache_path(),
            config,
            frequency: None,
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
        // WLF-only cache
        if !Path::new(&self.bin_cache_path).exists() {
            return Err(DataError::MissingData("Cache file not found".to_string()).into());
        }
        let cache = self.read_wlf(&self.bin_cache_path).await?;
        Ok(cache)
    }

    /// Load cache file ignoring staleness checks (best-effort fallback)
    async fn load_cache_unchecked(&self) -> Result<WordListCache> {
        // WLF-only (ignore freshness)
        if !Path::new(&self.bin_cache_path).exists() {
            return Err(DataError::MissingData("Cache file not found".to_string()).into());
        }
        let cache = self.read_wlf_unchecked(&self.bin_cache_path).await?;
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

        // Dedup for stable output
        let mut a = answer_words.to_vec();
        let mut g = guess_words.to_vec();
        a.sort();
        a.dedup();
        g.sort();
        g.dedup();

        let frequency = Self::compute_frequency(&a);

        let cache = WordListCache {
            answer_words: a,
            guess_words: g,
            last_updated: now,
            frequency,
        };

        // Write compact binary cache for fast load (WLF only)
        self.write_wlf(&self.bin_cache_path, &cache).await?;
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

    /// Get the path to the binary word list cache file (WLF)
    pub fn bin_cache_path(&self) -> &str {
        &self.bin_cache_path
    }

    /// Get frequency data if available
    pub fn frequency_data(&self) -> Option<&FrequencyData> {
        self.frequency.as_ref()
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

impl FileWordListProvider {
    /// Ensure cache freshness (< 24h)
    fn ensure_fresh(&self, cache: &WordListCache) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| DataError::InvalidFormat("System time error".to_string()))?
            .as_secs();
        if now - cache.last_updated > 24 * 60 * 60 {
            return Err(DataError::InvalidFormat("Cache too old".to_string()).into());
        }
        Ok(())
    }

    /// Read the compact Wordle List Format (WLF1), verifying freshness
    async fn read_wlf(&self, path: &str) -> Result<WordListCache> {
        let bytes = tokio::fs::read(path).await.map_err(DataError::from)?;
        let cache = Self::parse_wlf(&bytes)?;
        self.ensure_fresh(&cache)?;
        Ok(cache)
    }

    /// Read WLF without freshness check
    async fn read_wlf_unchecked(&self, path: &str) -> Result<WordListCache> {
        let bytes = tokio::fs::read(path).await.map_err(DataError::from)?;
        let cache = Self::parse_wlf(&bytes)?;
        Ok(cache)
    }

    /// Write the compact WLF file (WLF3)
    async fn write_wlf(&self, path: &str, cache: &WordListCache) -> Result<()> {
        // Format (WLF3):
        // magic: b"WLF3" (4)
        // last_updated: u64 LE (8)
        // answers_count: u32 LE (4)
        // guesses_count: u32 LE (4)
        // answers words: answers_count * 5 bytes (ASCII a-z)
        // guesses words: guesses_count * 5 bytes
        // letter_counts: 26 * u32
        // position_counts: 5 * 26 * u32
        // bigram_counts: 4 * 26 * 26 * u32
        let mut buf = Vec::with_capacity(
            4 + 8
                + 4
                + 4
                + (cache.answer_words.len() + cache.guess_words.len()) * 5
                + 26 * 4
                + 5 * 26 * 4
                + 4 * 26 * 26 * 4,
        );
        buf.extend_from_slice(b"WLF3");
        buf.extend_from_slice(&cache.last_updated.to_le_bytes());
        let a = cache.answer_words.len() as u32;
        let g = cache.guess_words.len() as u32;
        buf.extend_from_slice(&a.to_le_bytes());
        buf.extend_from_slice(&g.to_le_bytes());
        for w in &cache.answer_words {
            Self::push_word5(&mut buf, w)?;
        }
        for w in &cache.guess_words {
            Self::push_word5(&mut buf, w)?;
        }
        // frequency
        for i in 0..26 {
            buf.extend_from_slice(&cache.frequency.letter_counts[i].to_le_bytes());
        }
        for pos in 0..5 {
            for i in 0..26 {
                buf.extend_from_slice(&cache.frequency.position_counts[pos][i].to_le_bytes());
            }
        }
        for pair in 0..4 {
            for a in 0..26 {
                for b in 0..26 {
                    buf.extend_from_slice(&cache.frequency.bigram_counts[pair][a][b].to_le_bytes());
                }
            }
        }
        tokio::fs::write(path, buf).await.map_err(DataError::from)?;
        Ok(())
    }

    fn push_word5(buf: &mut Vec<u8>, w: &str) -> Result<()> {
        if w.len() != 5 || !w.chars().all(|c| c.is_ascii_lowercase()) {
            return Err(DataError::InvalidFormat(format!("Invalid word in cache: {}", w)).into());
        }
        for b in w.as_bytes() {
            buf.push(*b);
        }
        Ok(())
    }

    pub(crate) fn parse_wlf(bytes: &[u8]) -> Result<WordListCache> {
        if bytes.len() < 4 + 8 + 4 + 4 {
            return Err(DataError::InvalidFormat("WLF too small".to_string()).into());
        }
        let magic = &bytes[0..4];
        let is_v1 = magic == b"WLF1";
        let is_v2 = magic == b"WLF2";
        let is_v3 = magic == b"WLF3";
        if !is_v1 && !is_v2 && !is_v3 {
            return Err(DataError::InvalidFormat("WLF magic mismatch".to_string()).into());
        }
        let mut off = 4;
        let read_u64 = |data: &[u8], off: &mut usize| -> u64 {
            let mut arr = [0u8; 8];
            arr.copy_from_slice(&data[*off..*off + 8]);
            *off += 8;
            u64::from_le_bytes(arr)
        };
        let read_u32 = |data: &[u8], off: &mut usize| -> u32 {
            let mut arr = [0u8; 4];
            arr.copy_from_slice(&data[*off..*off + 4]);
            *off += 4;
            u32::from_le_bytes(arr)
        };
        let last_updated = read_u64(bytes, &mut off);
        let a = read_u32(bytes, &mut off) as usize;
        let g = read_u32(bytes, &mut off) as usize;
        let mut needed = 4 + 8 + 4 + 4 + (a + g) * 5;
        if is_v2 || is_v3 {
            needed += 26 * 4 + 5 * 26 * 4;
            if is_v3 {
                needed += 4 * 26 * 26 * 4;
            }
        }
        if bytes.len() != needed {
            return Err(DataError::InvalidFormat("WLF size mismatch".to_string()).into());
        }
        let mut answer_words = Vec::with_capacity(a);
        for i in 0..a {
            let start = off + i * 5;
            let end = start + 5;
            let s = std::str::from_utf8(&bytes[start..end])
                .map_err(|e| DataError::InvalidFormat(format!("UTF-8 error: {}", e)))?;
            if !s.chars().all(|c| c.is_ascii_lowercase()) {
                return Err(DataError::InvalidFormat("Non-lowercase word".to_string()).into());
            }
            answer_words.push(s.to_string());
        }
        let mut guess_words = Vec::with_capacity(g);
        let base = off + a * 5;
        for i in 0..g {
            let start = base + i * 5;
            let end = start + 5;
            let s = std::str::from_utf8(&bytes[start..end])
                .map_err(|e| DataError::InvalidFormat(format!("UTF-8 error: {}", e)))?;
            if !s.chars().all(|c| c.is_ascii_lowercase()) {
                return Err(DataError::InvalidFormat("Non-lowercase word".to_string()).into());
            }
            guess_words.push(s.to_string());
        }
        let mut frequency = FrequencyData::default();
        if is_v2 || is_v3 {
            let mut p = base + g * 5;
            for i in 0..26 {
                let mut arr = [0u8; 4];
                arr.copy_from_slice(&bytes[p..p + 4]);
                p += 4;
                frequency.letter_counts[i] = u32::from_le_bytes(arr);
            }
            for pos in 0..5 {
                for i in 0..26 {
                    let mut arr = [0u8; 4];
                    arr.copy_from_slice(&bytes[p..p + 4]);
                    p += 4;
                    frequency.position_counts[pos][i] = u32::from_le_bytes(arr);
                }
            }
            if is_v3 {
                for pair in 0..4 {
                    for a in 0..26 {
                        for b in 0..26 {
                            let mut arr = [0u8; 4];
                            arr.copy_from_slice(&bytes[p..p + 4]);
                            p += 4;
                            frequency.bigram_counts[pair][a][b] = u32::from_le_bytes(arr);
                        }
                    }
                }
            }
        }
        Ok(WordListCache {
            answer_words,
            guess_words,
            last_updated,
            frequency,
        })
    }

    fn compute_frequency(words: &[String]) -> FrequencyData {
        let mut freq = FrequencyData::default();
        for w in words {
            let bytes = w.as_bytes();
            let mut seen = [false; 26];
            for (pos, &b) in bytes.iter().enumerate().take(5) {
                let idx = (b - b'a') as usize;
                if idx < 26 {
                    freq.position_counts[pos][idx] += 1;
                    if !seen[idx] {
                        freq.letter_counts[idx] += 1;
                        seen[idx] = true;
                    }
                }
            }
            // bigrams: pairs 0-1,1-2,2-3,3-4
            for pair in 0..4 {
                let a = (bytes[pair] - b'a') as usize;
                let b = (bytes[pair + 1] - b'a') as usize;
                if a < 26 && b < 26 {
                    freq.bigram_counts[pair][a][b] += 1;
                }
            }
        }
        freq
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
        // Prefer fast local binary cache without freshness check (bundled file)
        match self.load_cache_unchecked().await {
            Ok(cache) => {
                log::info!("Loaded word lists from local binary cache (unchecked)");
                self.answer_words = Self::convert_to_words(cache.answer_words)?;
                self.guess_words = Self::convert_to_words(cache.guess_words)?;
                self.frequency = Some(cache.frequency);
            }
            Err(_) => {
                // Try fresh-checked cache next
                if let Ok(cache) = self.load_from_cache().await {
                    log::info!("Loaded word lists from cache (fresh)");
                    self.answer_words = Self::convert_to_words(cache.answer_words)?;
                    self.guess_words = Self::convert_to_words(cache.guess_words)?;
                    self.frequency = Some(cache.frequency);
                } else {
                    // Last resort: network download
                    log::info!("Downloading fresh word lists");
                    match self.download_words().await {
                        Ok((answer_strings, guess_strings)) => {
                            self.answer_words = Self::convert_to_words(answer_strings.clone())?;
                            self.guess_words = Self::convert_to_words(guess_strings.clone())?;
                            // Save to cache
                            self.save_to_cache(&answer_strings, &guess_strings).await?;
                            // After save, reload frequency from written cache
                            if let Ok(cache) = self.load_cache_unchecked().await {
                                self.frequency = Some(cache.frequency);
                            }
                        }
                        Err(e) => {
                            log::warn!("Download failed: {}. No local cache available.", e);
                            return Err(e.into());
                        }
                    }
                }
            }
        }

        // Ensure sorted unique internal lists for fast binary_search
        self.answer_words.sort();
        self.answer_words.dedup();
        self.guess_words.sort();
        self.guess_words.dedup();

        let mut all_words = Vec::with_capacity(self.answer_words.len() + self.guess_words.len());
        all_words.extend(self.answer_words.iter().cloned());
        all_words.extend(self.guess_words.iter().cloned());
        all_words.sort();
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
        // Use binary search over sorted lists
        self.guess_words.binary_search(word).is_ok()
            || self.answer_words.binary_search(word).is_ok()
    }

    fn is_possible_answer(&self, word: &Word) -> bool {
        self.answer_words.binary_search(word).is_ok()
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
        // The binary cache path should now point to the project root
        assert!(provider.bin_cache_path().ends_with("word_lists.wlf"));
        assert_eq!(provider.answer_words.len(), 0);
        assert_eq!(provider.guess_words.len(), 0);
    }
}
