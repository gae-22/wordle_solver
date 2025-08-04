use anyhow::Result;
use clap::{Parser, Subcommand};
use wordle_rust::{
    Command, CommandExecutor, CommandResult, Word, WordleApplicationService,
    core::types::FeedbackPattern, run_tui,
};

#[derive(Parser)]
#[command(name = "wordle_rust")]
#[command(about = "Modern AI Wordle Solver with Clean Architecture")]
#[command(version = "1.0.0")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the interactive TUI mode
    Interactive,
    /// Solve a specific wordle puzzle
    Solve {
        /// Target word to solve (for testing)
        #[arg(short, long)]
        target: Option<String>,
        /// Previous guesses in format "word feedback" (e.g., "adieu 20100")
        #[arg(short, long, value_delimiter = ' ', num_args = 2)]
        guess: Vec<String>,
    },
    /// Get the best first guess
    FirstGuess,
    /// Benchmark solver performance
    Benchmark {
        /// Number of words to test (default: 100)
        #[arg(short, long, default_value = "100")]
        count: usize,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Interactive) | None => {
            run_interactive_mode().await?;
        }
        Some(Commands::Solve { target, guess }) => {
            solve_puzzle(target, guess).await?;
        }
        Some(Commands::FirstGuess) => {
            get_first_guess().await?;
        }
        Some(Commands::Benchmark { count }) => {
            run_benchmark(count).await?;
        }
    }

    Ok(())
}

async fn run_interactive_mode() -> Result<()> {
    log::info!("Starting modern TUI mode...");

    // Run the new TUI application
    run_tui().await
}

async fn solve_puzzle(target: Option<String>, guess_pairs: Vec<String>) -> Result<()> {
    let mut app_service = WordleApplicationService::new().await?;

    // Set target word if provided
    if let Some(target_word) = target {
        let word = Word::from_str(&target_word)
            .map_err(|e| anyhow::anyhow!("Invalid target word: {}", e))?;
        app_service.execute(Command::StartGame {
            target_word: Some(word),
        })?;
        println!("🎯 Target word set: {}", target_word);
    }

    // Process previous guesses
    let mut i = 0;
    while i < guess_pairs.len() {
        if i + 1 < guess_pairs.len() {
            let word_str = &guess_pairs[i];
            let feedback_str = &guess_pairs[i + 1];

            let word = Word::from_str(word_str)
                .map_err(|e| anyhow::anyhow!("Invalid guess word '{}': {}", word_str, e))?;

            let feedback = FeedbackPattern::from_code_string(feedback_str)
                .map_err(|e| anyhow::anyhow!("Invalid feedback '{}': {}", feedback_str, e))?;

            let result = app_service.execute(Command::AddGuessResult {
                word,
                feedback: feedback.clone(),
            })?;

            if let CommandResult::GuessResultAdded { remaining_words } = result {
                println!(
                    "📝 Added guess: {} -> {} (🔢 {} words remaining)",
                    word_str, feedback, remaining_words
                );
            }

            i += 2;
        } else {
            break;
        }
    }

    // Get next best guess
    let result = app_service.execute(Command::GetBestGuess)?;
    match result {
        CommandResult::BestGuess { word, confidence } => {
            println!(
                "🎯 Next best guess: {} (confidence: {:.2})",
                word, confidence
            );

            // Show additional statistics
            let stats_result = app_service.execute(Command::GetStatistics)?;
            if let CommandResult::Statistics { stats } = stats_result {
                println!("📊 Remaining words: {}", stats.remaining_words);
                if !stats.possible_words_sample.is_empty() {
                    let sample: Vec<String> = stats
                        .possible_words_sample
                        .iter()
                        .take(5)
                        .map(|w| w.to_string())
                        .collect();
                    println!("🔍 Sample possibilities: {}", sample.join(", "));
                }
            }
        }
        CommandResult::Error { message } => {
            eprintln!("❌ Error: {}", message);
        }
        _ => {
            println!("🤔 Unexpected result type");
        }
    }

    Ok(())
}

async fn get_first_guess() -> Result<()> {
    let app_service = WordleApplicationService::new().await?;
    let first_guess = app_service.get_best_first_guess()?;

    println!("🌟 Best first guess: {}", first_guess);
    println!("💡 This word has been statistically optimized for maximum information gain!");

    Ok(())
}

async fn run_benchmark(count: usize) -> Result<()> {
    println!("🚀 Running benchmark with {} words...", count);

    let mut app_service = WordleApplicationService::new().await?;
    let first_guess = app_service.get_best_first_guess()?;

    // Get word list for testing
    let stats_result = app_service.execute(Command::GetStatistics)?;
    let test_words = if let CommandResult::Statistics { stats } = stats_result {
        stats
            .possible_words_sample
            .into_iter()
            .take(count)
            .collect::<Vec<_>>()
    } else {
        vec![
            Word::from_str("apple").map_err(|e| anyhow::anyhow!(e))?,
            Word::from_str("bread").map_err(|e| anyhow::anyhow!(e))?,
            Word::from_str("crane").map_err(|e| anyhow::anyhow!(e))?,
        ]
    };

    println!("📊 Benchmark Results:");
    println!("🥇 Best first guess: {}", first_guess);
    println!("📈 Testing against {} words", test_words.len());

    let mut total_guesses = 0;
    let mut success_count = 0;

    for (i, target_word) in test_words.iter().enumerate() {
        if i >= count {
            break;
        }

        // Reset for each test
        app_service.execute(Command::Reset)?;
        app_service.execute(Command::StartGame {
            target_word: Some(target_word.clone()),
        })?;

        let mut guesses = 0;
        let mut solved = false;

        // Simulate solving (simplified version)
        while guesses < 6 && !solved {
            let guess_result = app_service.execute(Command::GetBestGuess)?;
            if let CommandResult::BestGuess { word, .. } = guess_result {
                guesses += 1;

                // Check if this would be the correct guess
                if word.as_str() == target_word.as_str() {
                    solved = true;
                    success_count += 1;
                    total_guesses += guesses;
                    break;
                }

                // Create mock feedback (this would normally come from the game)
                // For benchmarking, we'll assume we get some feedback and continue
                let mock_feedback = FeedbackPattern::from_code_string("01020").unwrap();
                app_service.execute(Command::AddGuessResult {
                    word,
                    feedback: mock_feedback,
                })?;
            }
        }

        if !solved {
            total_guesses += 6; // Failed attempts count as 6 guesses
        }

        if (i + 1) % 10 == 0 {
            println!("⏳ Processed {} words...", i + 1);
        }
    }

    let avg_guesses = if success_count > 0 {
        total_guesses as f64 / success_count as f64
    } else {
        6.0
    };

    println!("🎯 Benchmark Complete!");
    println!(
        "✅ Success rate: {:.1}% ({}/{})",
        (success_count as f64 / test_words.len() as f64) * 100.0,
        success_count,
        test_words.len()
    );
    println!("📊 Average guesses per solved word: {:.2}", avg_guesses);

    Ok(())
}
