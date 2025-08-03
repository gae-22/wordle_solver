use std::fmt;

/// Application-specific error types
#[derive(Debug)]
pub enum WordleError {
    /// Game-related errors
    Game(GameError),
    /// Solver-related errors
    Solver(SolverError),
    /// Word list or data errors
    Data(DataError),
    /// UI/Interface errors
    Interface(InterfaceError),
}

#[derive(Debug)]
pub enum GameError {
    /// Invalid word length
    InvalidWordLength { expected: usize, actual: usize },
    /// Word not in dictionary
    WordNotFound(String),
    /// No target word set
    NoTargetWord,
    /// Game already finished
    GameFinished,
}

#[derive(Debug)]
pub enum SolverError {
    /// No possible words remaining
    NoPossibleWords,
    /// Invalid feedback format
    InvalidFeedback(String),
    /// Algorithm failure
    AlgorithmFailure(String),
    /// No candidates available
    NoCandidates,
}

#[derive(Debug)]
pub enum DataError {
    /// File I/O error
    FileError(std::io::Error),
    /// JSON parsing error
    JsonError(serde_json::Error),
    /// Invalid data format
    InvalidFormat(String),
    /// Missing required data
    MissingData(String),
}

#[derive(Debug)]
pub enum InterfaceError {
    /// Terminal setup error
    TerminalError(String),
    /// Event handling error
    EventError(String),
    /// Rendering error
    RenderError(String),
}

impl fmt::Display for WordleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WordleError::Game(e) => write!(f, "Game error: {}", e),
            WordleError::Solver(e) => write!(f, "Solver error: {}", e),
            WordleError::Data(e) => write!(f, "Data error: {}", e),
            WordleError::Interface(e) => write!(f, "Interface error: {}", e),
        }
    }
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameError::InvalidWordLength { expected, actual } => {
                write!(f, "Invalid word length: expected {}, got {}", expected, actual)
            }
            GameError::WordNotFound(word) => write!(f, "Word not found: {}", word),
            GameError::NoTargetWord => write!(f, "No target word set"),
            GameError::GameFinished => write!(f, "Game already finished"),
        }
    }
}

impl fmt::Display for SolverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SolverError::NoPossibleWords => write!(f, "No possible words remaining"),
            SolverError::InvalidFeedback(msg) => write!(f, "Invalid feedback: {}", msg),
            SolverError::AlgorithmFailure(msg) => write!(f, "Algorithm failure: {}", msg),
            SolverError::NoCandidates => write!(f, "No candidates available"),
        }
    }
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::FileError(e) => write!(f, "File error: {}", e),
            DataError::JsonError(e) => write!(f, "JSON error: {}", e),
            DataError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            DataError::MissingData(field) => write!(f, "Missing required data: {}", field),
        }
    }
}

impl fmt::Display for InterfaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InterfaceError::TerminalError(msg) => write!(f, "Terminal error: {}", msg),
            InterfaceError::EventError(msg) => write!(f, "Event error: {}", msg),
            InterfaceError::RenderError(msg) => write!(f, "Render error: {}", msg),
        }
    }
}

impl std::error::Error for WordleError {}
impl std::error::Error for GameError {}
impl std::error::Error for SolverError {}
impl std::error::Error for DataError {}
impl std::error::Error for InterfaceError {}

// Convenience From implementations
impl From<GameError> for WordleError {
    fn from(err: GameError) -> Self {
        WordleError::Game(err)
    }
}

impl From<SolverError> for WordleError {
    fn from(err: SolverError) -> Self {
        WordleError::Solver(err)
    }
}

impl From<DataError> for WordleError {
    fn from(err: DataError) -> Self {
        WordleError::Data(err)
    }
}

impl From<InterfaceError> for WordleError {
    fn from(err: InterfaceError) -> Self {
        WordleError::Interface(err)
    }
}

impl From<std::io::Error> for DataError {
    fn from(err: std::io::Error) -> Self {
        DataError::FileError(err)
    }
}

impl From<serde_json::Error> for DataError {
    fn from(err: serde_json::Error) -> Self {
        DataError::JsonError(err)
    }
}

/// Result type alias for WordleError
pub type Result<T> = std::result::Result<T, WordleError>;
