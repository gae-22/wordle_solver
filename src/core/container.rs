/// Dependency Injection Container for Clean Architecture
use crate::core::{
    error::Result,
    traits::{
        ConstraintFilter, EntropyCalculator, FeedbackGenerator, GameEngine, SolvingStrategy,
        WordListProvider, WordleSolver,
    },
};
use std::sync::Arc;

/// Configuration for dependency injection
#[derive(Debug, Clone)]
pub struct DependencyConfig {
    /// Strategy to use for solving
    pub strategy_type: StrategyType,
    /// Whether to use cached entropy calculator
    pub use_cached_entropy: bool,
    /// Word list source configuration
    pub word_list_config: WordListConfig,
}

#[derive(Debug, Clone)]
pub enum StrategyType {
    Entropy,
    Frequency,
    Hybrid,
}

#[derive(Debug, Clone)]
pub struct WordListConfig {
    /// Path to word list file
    pub file_path: Option<String>,
    /// Whether to load extended guess words
    pub include_extended_guesses: bool,
}

impl Default for DependencyConfig {
    fn default() -> Self {
        Self {
            strategy_type: StrategyType::Entropy,
            use_cached_entropy: true,
            word_list_config: WordListConfig {
                file_path: None,
                include_extended_guesses: true,
            },
        }
    }
}

/// Dependency injection container following Clean Architecture principles
pub struct Container {
    config: DependencyConfig,
}

impl Container {
    /// Create a new container with default configuration
    pub fn new() -> Self {
        Self {
            config: DependencyConfig::default(),
        }
    }

    /// Create a new container with custom configuration
    pub fn with_config(config: DependencyConfig) -> Self {
        Self { config }
    }

    /// Create word list provider
    pub fn create_word_list_provider(&self) -> Result<Box<dyn WordListProvider>> {
        let provider = if let Some(file_path) = &self.config.word_list_config.file_path {
            crate::infrastructure::FileWordListProvider::with_path(file_path.clone())
        } else {
            crate::infrastructure::FileWordListProvider::new()
        };

        Ok(Box::new(provider))
    }

    /// Create entropy calculator
    pub fn create_entropy_calculator(&self) -> impl EntropyCalculator {
        if self.config.use_cached_entropy {
            crate::infrastructure::CachedEntropyCalculator::new()
        } else {
            crate::infrastructure::CachedEntropyCalculator::new() // Use cached for both for now
        }
    }

    /// Create solving strategy
    pub async fn create_strategy(&self) -> Result<Box<dyn SolvingStrategy>> {
        let strategy: Box<dyn SolvingStrategy> = match self.config.strategy_type {
            StrategyType::Entropy => {
                let entropy_calc = crate::infrastructure::CachedEntropyCalculator::new();
                Box::new(crate::infrastructure::EntropyBasedStrategy::new(
                    entropy_calc,
                )?)
            }
            StrategyType::Frequency => {
                // Need to load words first for frequency analysis
                let mut word_provider = self.create_word_list_provider()?;
                word_provider.load_words().await?;
                let words = word_provider.get_answer_words();
                Box::new(crate::infrastructure::FrequencyBasedStrategy::new(words)?)
            }
            StrategyType::Hybrid => {
                // Create hybrid strategy combining entropy and frequency
                let entropy_calc = crate::infrastructure::CachedEntropyCalculator::new();
                Box::new(crate::infrastructure::HybridStrategy::new(entropy_calc)?)
            }
        };

        Ok(strategy)
    }

    /// Create constraint filter
    pub fn create_constraint_filter(&self) -> Box<dyn ConstraintFilter> {
        Box::new(crate::domain::DefaultConstraintFilter::new())
    }

    /// Create feedback generator
    pub fn create_feedback_generator(&self) -> Box<dyn FeedbackGenerator> {
        Box::new(crate::domain::DefaultFeedbackGenerator::new())
    }

    /// Create game engine
    pub async fn create_game_engine(&self) -> Result<Box<dyn GameEngine>> {
        let feedback_generator = self.create_feedback_generator();
        let game_engine =
            crate::domain::DefaultGameEngine::with_feedback_generator_async(feedback_generator)
                .await?;
        Ok(Box::new(game_engine))
    }

    /// Create wordle solver with all dependencies injected
    pub async fn create_solver(&self) -> Result<Box<dyn WordleSolver>> {
        let word_list_provider = self.create_word_list_provider()?;
        let strategy = self.create_strategy().await?;
        let constraint_filter = self.create_constraint_filter();

        let solver = crate::domain::DefaultWordleSolver::new(
            word_list_provider,
            strategy,
            constraint_filter,
        )
        .await?;

        Ok(Box::new(solver))
    }

    /// Create application service with dependency injection
    pub async fn create_application_service(
        &self,
    ) -> Result<crate::application::WordleApplicationService> {
        let game_engine = self.create_game_engine().await?;
        let solver = self.create_solver().await?;

        crate::application::WordleApplicationService::with_dependencies(game_engine, solver).await
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}

/// Factory trait for creating components
pub trait ComponentFactory<T> {
    fn create(&self) -> Result<T>;
}

/// Async factory trait for creating components
#[async_trait::async_trait]
pub trait AsyncComponentFactory<T> {
    async fn create(&self) -> Result<T>;
}

/// Registry for managing component factories
pub struct ComponentRegistry {
    container: Arc<Container>,
}

impl ComponentRegistry {
    pub fn new(container: Container) -> Self {
        Self {
            container: Arc::new(container),
        }
    }

    pub fn container(&self) -> &Container {
        &self.container
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DependencyConfig::default();
        assert!(matches!(config.strategy_type, StrategyType::Entropy));
        assert!(config.use_cached_entropy);
        assert!(config.word_list_config.include_extended_guesses);
    }

    #[test]
    fn test_container_creation() {
        let container = Container::new();
        // Test that components can be created without errors
        assert!(container.create_word_list_provider().is_ok());
        let _filter = container.create_constraint_filter();
        let _generator = container.create_feedback_generator();
    }

    #[tokio::test]
    async fn test_async_component_creation() {
        let container = Container::new();
        let result = container.create_game_engine().await;
        assert!(result.is_ok());
    }
}
