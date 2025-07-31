use crate::game::WordleGame;
use crate::solver::WordleSolver;
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

pub mod events;
pub mod state;
pub mod ui;

pub use events::*;
pub use state::*;
pub use ui::*;

/// Main application struct
pub struct App {
    pub state: AppState,
    pub game: WordleGame,
    pub solver: WordleSolver,
}

impl App {
    /// Create a new App instance
    pub async fn new() -> Result<Self> {
        let game = WordleGame::new().await?;
        let solver = WordleSolver::new().await?;
        let state = AppState::new();

        Ok(Self {
            state,
            game,
            solver,
        })
    }

    /// Run the TUI application
    pub async fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Get initial best guess
        self.state.current_suggestion = Some(self.solver.get_best_first_guess()?);
        self.update_top_candidates();

        let result = self.run_app(&mut terminal).await;

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result
    }

    async fn run_app<B: ratatui::backend::Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
    ) -> Result<()> {
        loop {
            terminal.draw(|f| ui::draw(f, &self.state))?;

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('r') => {
                        // Only allow restart when game is finished
                        if self.state.game_result != state::GameResult::InProgress {
                            self.state.reset();
                            self.solver.reset();
                            self.state.current_suggestion =
                                Some(self.solver.get_best_first_guess()?);
                            self.update_top_candidates();
                        } else {
                            // Treat 'r' as normal character input during game
                            if self.state.input_buffer.is_none() {
                                self.state.input_buffer = Some(String::new());
                            }
                            if let Some(ref mut buffer) = self.state.input_buffer {
                                buffer.push('r');
                            }
                        }
                    }
                    KeyCode::Enter => {
                        // Process input only when Enter is pressed and game is in progress
                        if self.state.game_result == state::GameResult::InProgress {
                            if let Some(input) = &self.state.input_buffer.clone() {
                                if !input.is_empty() {
                                    // This is where the actual command processing happens
                                    self.handle_input(input).await?;
                                }
                            }
                        }
                    }
                    KeyCode::Backspace => {
                        // Allow editing input buffer when game is in progress
                        if self.state.game_result == state::GameResult::InProgress {
                            if let Some(ref mut buffer) = self.state.input_buffer {
                                buffer.pop();
                            }
                        }
                    }
                    KeyCode::Char(c) => {
                        // Add characters to input buffer (no immediate processing)
                        if self.state.game_result == state::GameResult::InProgress {
                            if self.state.input_buffer.is_none() {
                                self.state.input_buffer = Some(String::new());
                            }
                            if let Some(ref mut buffer) = self.state.input_buffer {
                                buffer.push(c);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    async fn handle_input(&mut self, input: &str) -> Result<()> {
        let parts: Vec<&str> = input.split_whitespace().collect();

        if parts.len() == 2 {
            let word = parts[0];
            let result = parts[1];

            // Add the guess result to solver
            self.solver.add_guess_result(word, result)?;

            // Add to history
            self.state
                .guess_history
                .push((word.to_string(), result.to_string()));

            // Check if the word is solved (all 2s means correct)
            if result == "22222" {
                self.state.is_solved = true;
                self.state.game_result = state::GameResult::Won {
                    word: word.to_string(),
                    attempts: self.state.guess_history.len(),
                };
                self.state.current_suggestion = None;
            } else {
                // Update remaining words count
                self.state.remaining_words_count = self.solver.remaining_words_count();

                // Check if we have no possible words left (failure)
                if self.state.remaining_words_count == 0 {
                    self.state.game_result = state::GameResult::Failed {
                        attempts: self.state.guess_history.len(),
                    };
                    self.state.current_suggestion = None;
                } else if self.state.guess_history.len() >= 6 {
                    // Wordle typically allows 6 attempts
                    self.state.game_result = state::GameResult::Failed {
                        attempts: self.state.guess_history.len(),
                    };
                    self.state.current_suggestion = None;
                } else {
                    // Get next suggestion
                    self.state.current_suggestion = Some(self.solver.get_best_guess()?);
                    // Update candidates list for the new state
                    self.update_top_candidates();
                }
            }
        }

        // Clear input buffer
        self.state.input_buffer = None;

        Ok(())
    }

    /// Update the top candidates list with current solver state
    fn update_top_candidates(&mut self) {
        // Get top 10 candidates for display
        self.state.top_candidates = self.solver.get_top_candidates(10);
    }
}
