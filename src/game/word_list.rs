use anyhow::Result;
use std::collections::HashSet;

/// Word list management for Wordle
#[derive(Debug, Clone)]
pub struct WordList {
    /// All valid 5-letter words that can be answers
    answer_words: Vec<String>,
    /// All valid 5-letter words that can be guessed (includes answer_words)
    guess_words: HashSet<String>,
}

impl WordList {
    /// Create a new WordList with embedded word lists
    pub fn new() -> Result<Self> {
        // Common 5-letter words that are often used as Wordle answers
        let answer_words: Vec<String> = vec![
            "about", "above", "abuse", "actor", "acute", "admit", "adopt", "adult", "after",
            "again", "agent", "agree", "ahead", "alarm", "album", "alert", "alien", "align",
            "alike", "alive", "allow", "alone", "along", "alter", "amber", "among", "angel",
            "anger", "angle", "angry", "apart", "apple", "apply", "arena", "argue", "arise",
            "array", "arrow", "aside", "asset", "atlas", "audio", "audit", "avoid", "awake",
            "award", "aware", "badly", "baker", "bases", "basic", "batch", "beach", "began",
            "begin", "being", "below", "bench", "billy", "birth", "black", "blame", "blank",
            "blind", "block", "blood", "board", "boost", "booth", "bound", "brain", "brand",
            "brave", "bread", "break", "breed", "brief", "bring", "broad", "broke", "brown",
            "build", "built", "buyer", "cable", "calif", "carry", "catch", "cause", "chain",
            "chair", "chaos", "charm", "chart", "chase", "cheap", "check", "chest", "chief",
            "child", "china", "chose", "civil", "claim", "class", "clean", "clear", "click",
            "climb", "clock", "close", "cloud", "coach", "coast", "could", "count", "court",
            "cover", "craft", "crash", "crazy", "cream", "crime", "cross", "crowd", "crown",
            "crude", "curve", "cycle", "daily", "damage", "dance", "dated", "dealt", "death",
            "debut", "delay", "depth", "doing", "doubt", "dozen", "draft", "drama", "drank",
            "dream", "dress", "drill", "drink", "drive", "drove", "dying", "eager", "early",
            "earth", "eight", "elite", "empty", "enemy", "enjoy", "enter", "entry", "equal",
            "error", "event", "every", "exact", "exist", "extra", "faith", "false", "fault",
            "fiber", "field", "fifth", "fifty", "fight", "final", "first", "fixed", "flash",
            "fleet", "floor", "fluid", "focus", "force", "forth", "forty", "forum", "found",
            "frame", "frank", "fraud", "fresh", "front", "fruit", "fully", "funny", "giant",
            "given", "glass", "globe", "going", "grace", "grade", "grand", "grant", "grass",
            "grave", "great", "green", "gross", "group", "grown", "guard", "guest", "guide",
            "happy", "harry", "heart", "heavy", "henry", "horse", "hotel", "house", "human",
            "ideal", "image", "index", "inner", "input", "issue", "japan", "jimmy", "joint",
            "jones", "judge", "knife", "knock", "known", "label", "large", "laser", "later",
            "laugh", "layer", "learn", "lease", "least", "leave", "legal", "level", "lewis",
            "light", "limit", "links", "lives", "local", "loose", "lower", "lucky", "lunch",
            "lying", "magic", "major", "maker", "march", "maria", "match", "maybe", "mayor",
            "meant", "media", "metal", "might", "minor", "minus", "mixed", "model", "money",
            "month", "moral", "motor", "mount", "mouse", "mouth", "moved", "movie", "music",
            "needs", "never", "newly", "night", "noise", "north", "noted", "novel", "nurse",
            "occur", "ocean", "offer", "often", "order", "other", "ought", "paint", "panel",
            "paper", "party", "peace", "peter", "phase", "phone", "photo", "piano", "piece",
            "pilot", "pitch", "place", "plain", "plane", "plant", "plate", "point", "pound",
            "power", "press", "price", "pride", "prime", "print", "prior", "prize", "proof",
            "proud", "prove", "queen", "quick", "quiet", "quite", "radio", "raise", "range",
            "rapid", "ratio", "reach", "ready", "realm", "rebel", "refer", "relax", "repay",
            "reply", "right", "rigid", "river", "robot", "roger", "roman", "rough", "round",
            "route", "royal", "rural", "scale", "scene", "scope", "score", "sense", "serve",
            "setup", "seven", "shall", "shape", "share", "sharp", "sheet", "shelf", "shell",
            "shift", "shine", "shirt", "shock", "shoot", "short", "shown", "sides", "sight",
            "silly", "since", "sixth", "sixty", "sized", "skill", "sleep", "slide", "small",
            "smart", "smile", "smith", "smoke", "snake", "solar", "solid", "solve", "sorry",
            "sound", "south", "space", "spare", "speak", "speed", "spend", "spent", "split",
            "spoke", "sport", "staff", "stage", "stake", "stand", "start", "state", "steam",
            "steel", "steep", "steer", "steve", "stick", "still", "stock", "stone", "stood",
            "store", "storm", "story", "strip", "stuck", "study", "stuff", "style", "sugar",
            "suite", "super", "sweet", "table", "taken", "taste", "taxes", "teach", "teeth",
            "terry", "texas", "thank", "theft", "their", "theme", "there", "these", "thick",
            "thing", "think", "third", "those", "three", "threw", "throw", "thumb", "tiger",
            "tight", "timer", "title", "today", "topic", "total", "touch", "tough", "tower",
            "track", "trade", "train", "treat", "trend", "trial", "tribe", "trick", "tried",
            "tries", "truck", "truly", "trunk", "trust", "truth", "twice", "twist", "uncle",
            "under", "undue", "union", "unity", "until", "upper", "upset", "urban", "usage",
            "usual", "valid", "value", "video", "virus", "visit", "vital", "vocal", "voice",
            "waste", "watch", "water", "weary", "wheel", "where", "which", "while", "white",
            "whole", "whose", "woman", "women", "world", "worry", "worse", "worst", "worth",
            "would", "write", "wrong", "wrote", "yield", "young", "youth",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect();

        // Create guess words set (includes all answer words plus additional valid guesses)
        let mut guess_words: HashSet<String> = HashSet::new();
        for word in answer_words.iter() {
            guess_words.insert(word.clone());
        }

        // Add additional common guesses that might not be answers
        let additional_guesses = vec![
            "adieu", "slate", "crate", "trace", "lance", "sauce", "glace", "stare",
        ];

        for word in additional_guesses {
            guess_words.insert(word.to_string());
        }

        Ok(Self {
            answer_words,
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
}
