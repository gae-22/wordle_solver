use anyhow::Result;
use clap::{Parser, Subcommand};
use wordle_rust::app::App;
use wordle_rust::game::WordleGame;
use wordle_rust::solver::WordleSolver;

#[derive(Parser)]
#[command(name = "wordle_rust")]
#[command(about = "AI Wordle Solver with TUI Interface")]
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
        /// Previous guesses in format "word result" (e.g., "adieu 20100")
        #[arg(short, long, value_delimiter = ' ', num_args = 2)]
        guess: Vec<String>,
    },
    /// Get the best first guess
    FirstGuess,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Interactive) | None => {
            let mut app = App::new().await?;
            app.run().await?;
        }
        Some(Commands::Solve { target, guess }) => {
            let mut game = WordleGame::new().await?;
            let mut solver = WordleSolver::new().await?;

            if let Some(target_word) = target {
                game.set_target(&target_word)?;
            }

            // Process previous guesses
            let mut i = 0;
            while i < guess.len() {
                if i + 1 < guess.len() {
                    let word = &guess[i];
                    let result = &guess[i + 1];
                    solver.add_guess_result(word, result)?;
                    i += 2;
                } else {
                    break;
                }
            }

            let next_guess = solver.get_best_guess()?;
            println!("Next best guess: {}", next_guess);
        }
        Some(Commands::FirstGuess) => {
            let solver = WordleSolver::new().await?;
            let first_guess = solver.get_best_first_guess()?;
            println!("Best first guess: {}", first_guess);
        }
    }

    Ok(())
}
