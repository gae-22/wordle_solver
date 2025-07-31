pub mod word_list;
pub mod word_list_new;
pub mod wordle;

// Use the new word list implementation by default
pub use word_list_new::*;
pub use wordle::*;
