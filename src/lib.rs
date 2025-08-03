// Modern Rust Wordle Solver with Clean Architecture

/// Core domain types, traits, and error handling
pub mod core;

/// Domain layer - business logic and rules
pub mod domain;

/// Infrastructure layer - external dependencies and implementations
pub mod infrastructure;

/// Application layer - orchestration and use cases
pub mod application;

// Re-export main types for easy access
pub use core::error::{Result, WordleError};

// Primary interfaces
pub use core::traits::{
    ConstraintFilter, DataPersistence, EntropyCalculator, FeedbackGenerator, GameEngine,
    SolvingStrategy, StateManager, UserInterface, WordListProvider,
    WordleSolver as CoreWordleSolver,
};

// Core types
pub use core::types::{
    Feedback as CoreFeedback, FeedbackPattern, GameResult as CoreGameResult, Guess,
    SolverStatistics as CoreSolverStatistics, Word,
};

// Domain implementations
pub use domain::{
    DefaultConstraintFilter, DefaultFeedbackGenerator, DefaultGameEngine, DefaultWordleSolver,
};

// Infrastructure implementations
pub use infrastructure::{
    CachedEntropyCalculator, EntropyBasedStrategy, FileWordListProvider, FrequencyBasedStrategy,
    SimpleEntropyCalculator,
};

// Application layer
pub use application::{
    AppEvent, AppState as ApplicationState, Command, CommandExecutor, CommandResult,
    EventHandler as ApplicationEventHandler, WordleApplicationService,
};
