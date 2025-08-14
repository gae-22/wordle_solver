use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use tokio::time::{Duration, interval};

use crate::{
    Command, CommandResult, Container,
    application::{WordleApplicationService, commands::CommandExecutor},
    core::types::{FeedbackPattern, Word},
    presentation::tui::{
        events::{EventLoop, KeyAction, TuiEvent},
        feedback::FeedbackInputManager,
        layout::{LayoutManager, LayoutPreset},
        state::{LogLevel, MessageType, TuiState},
    },
};

/// Main TUI application
pub struct TuiApp {
    /// Terminal interface
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    /// Application state
    state: TuiState,
    /// Wordle application service
    app_service: WordleApplicationService,
    /// Event loop
    event_loop: EventLoop,
    /// Feedback input manager
    feedback_manager: FeedbackInputManager,
    /// Whether the application should quit
    should_quit: bool,
}

impl TuiApp {
    /// Create a new TUI application
    pub async fn new() -> Result<Self> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        // Check terminal size
        let size = terminal.size()?;
        if !LayoutManager::is_size_adequate(size.width, size.height) {
            let (min_w, min_h) = LayoutManager::min_size();
            let (rec_w, rec_h) = LayoutManager::recommended_size();
            return Err(anyhow::anyhow!(
                "Terminal too small! Current: {}x{}, Minimum: {}x{}, Recommended: {}x{}",
                size.width,
                size.height,
                min_w,
                min_h,
                rec_w,
                rec_h
            ));
        }

        // Initialize application components
        let container = Container::new();
        let mut state = TuiState::new();

        // Optionally refresh word lists on interactive start
        // Controlled by WORDLE_REFRESH_ON_START=1
        if std::env::var("WORDLE_REFRESH_ON_START")
            .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE"))
            .unwrap_or(false)
        {
            state.add_log(
                LogLevel::Info,
                "Refreshing word lists from remote sources...".to_string(),
            );
            match container.create_word_list_provider() {
                Ok(mut provider) => match provider.refresh(true).await {
                    Ok((answers, guesses)) => {
                        state.add_log(
                            LogLevel::Info,
                            format!(
                                "Word lists updated. Answers: {}, Guesses: {}",
                                answers, guesses
                            ),
                        );
                    }
                    Err(e) => {
                        state.add_log(
                            LogLevel::Warning,
                            format!(
                                "Failed to refresh word lists: {} (using cache if available)",
                                e
                            ),
                        );
                    }
                },
                Err(e) => {
                    state.add_log(
                        LogLevel::Warning,
                        format!("Could not create word list provider: {}", e),
                    );
                }
            }
        } else {
            state.add_log(
                LogLevel::Info,
                "Using existing cached word lists (set WORDLE_REFRESH_ON_START=1 to refresh)."
                    .to_string(),
            );
        }

        // Create application service after refresh so it picks up fresh cache
        let app_service = container.create_application_service().await?;
        let event_loop = EventLoop::default();

        // Get initial suggestion
        state.add_log(LogLevel::Info, "Initializing Wordle Solver...".to_string());

        Ok(Self {
            terminal,
            state,
            app_service,
            event_loop,
            feedback_manager: FeedbackInputManager::new(),
            should_quit: false,
        })
    }

    /// Run the TUI application
    pub async fn run(&mut self) -> Result<()> {
        // Get initial first guess
        self.get_first_guess().await?;

        // Main application loop
        let mut tick_interval = interval(Duration::from_millis(250));

        loop {
            // Draw the UI
            self.draw()?;

            // Handle events
            tokio::select! {
                // Handle keyboard/terminal events
                event_result = self.event_loop.next_event() => {
                    match event_result {
                        Ok(event) => {
                            // If solved, exit on any key press
                            if self.state.is_solved {
                                if let TuiEvent::Key(_) = event {
                                    self.should_quit = true;
                                }
                            } else {
                                // Normal event processing
                                let is_typing = !self.state.input.is_empty();
                                let current_mode = self.state.interaction_mode();
                                let action = self.event_loop.process_event(event, current_mode, is_typing);
                                self.handle_action(action).await?;
                            }
                        }
                        Err(e) => {
                            self.state.add_log(
                                LogLevel::Error,
                                format!("Event error: {}", e)
                            );
                        }
                    }
                }

                // Periodic updates
                _ = tick_interval.tick() => {
                    self.update().await?;
                }
            }

            if self.should_quit {
                break;
            }
        }

        Ok(())
    }

    /// Draw the UI
    fn draw(&mut self) -> Result<()> {
        self.terminal.draw(|frame| {
            let size = frame.size();
            let preset = LayoutPreset::from_size(size.width, size.height);
            preset.render(frame, &self.state, &self.feedback_manager);
        })?;
        Ok(())
    }

    /// Handle user actions
    async fn handle_action(&mut self, action: KeyAction) -> Result<()> {
        // If we're in feedback mode, handle feedback-specific actions
        if self.feedback_manager.is_in_feedback_mode() {
            return self.handle_feedback_action(action).await;
        }

        match action {
            KeyAction::AddChar(c) => {
                // Only handle character input in input mode and for relevant characters
                if self.state.interaction_mode().is_input() {
                    if c.is_ascii_alphabetic() || c.is_ascii_digit() {
                        self.state.add_char(c);
                        self.state.clear_status();
                    }
                }
            }

            KeyAction::DeleteChar => {
                if self.state.interaction_mode().is_input() {
                    self.state.delete_char();
                    self.state.clear_status();
                }
            }

            KeyAction::MoveCursorLeft => {
                if self.state.interaction_mode().is_input() {
                    self.state.move_cursor_left();
                }
            }

            KeyAction::MoveCursorRight => {
                if self.state.interaction_mode().is_input() {
                    self.state.move_cursor_right();
                }
            }

            KeyAction::Submit => {
                if self.state.interaction_mode().is_input() {
                    self.submit_guess().await?;
                }
            }

            KeyAction::Clear => {
                self.state.clear_input();
                self.state
                    .set_status("Input cleared".to_string(), MessageType::Info);
            }

            KeyAction::ToggleHelp => {
                self.state.toggle_help();
            }

            KeyAction::GetFirstGuess => {
                self.get_first_guess().await?;
            }

            KeyAction::ShowStats => {
                self.show_detailed_stats().await?;
            }

            KeyAction::Reset => {
                self.reset_game().await?;
            }

            KeyAction::Quit => {
                self.should_quit = true;
            }

            KeyAction::SwitchToInputMode => {
                self.state.switch_to_input_mode();
            }

            KeyAction::SwitchToOperationMode => {
                self.state.switch_to_operation_mode();
            }

            KeyAction::ToggleMode => {
                self.state.toggle_interaction_mode();
            }

            KeyAction::None => {
                // Do nothing
            }
        }

        Ok(())
    }

    /// Handle actions when in feedback input mode
    async fn handle_feedback_action(&mut self, action: KeyAction) -> Result<()> {
        match action {
            KeyAction::AddChar(c) if matches!(c, '0' | '1' | '2') => {
                self.feedback_manager.add_feedback_char(c, &mut self.state);
            }

            KeyAction::AddChar(c) => {
                // 無効な文字が入力された場合の処理
                if c.is_ascii_digit() {
                    self.state.set_status(
                        "Only use 0 (gray), 1 (yellow), or 2 (green) for feedback".to_string(),
                        MessageType::Warning,
                    );
                } else {
                    self.state.set_status(
                        "Please enter feedback numbers: 0, 1, or 2".to_string(),
                        MessageType::Warning,
                    );
                }
            }

            KeyAction::DeleteChar => {
                self.feedback_manager.delete_feedback_char(&mut self.state);
            }

            KeyAction::MoveCursorLeft => {
                self.feedback_manager.move_feedback_cursor_left();
            }

            KeyAction::MoveCursorRight => {
                self.feedback_manager.move_feedback_cursor_right();
            }

            KeyAction::Submit => {
                if let Some((word, feedback)) =
                    self.feedback_manager.submit_feedback(&mut self.state)
                {
                    self.process_guess_feedback(word, feedback).await?;
                }
            }

            KeyAction::Quit => {
                // In feedback mode, ESC should cancel feedback input
                self.feedback_manager.cancel_feedback(&mut self.state);
            }

            KeyAction::Clear => {
                self.feedback_manager.cancel_feedback(&mut self.state);
            }

            _ => {
                // Other actions are ignored in feedback mode
            }
        }

        Ok(())
    }

    /// Process guess feedback and add to game state
    async fn process_guess_feedback(&mut self, word: String, feedback: String) -> Result<()> {
        self.add_guess_result(word, feedback).await
    }

    /// Submit the current guess
    async fn submit_guess(&mut self) -> Result<()> {
        if !self.state.is_input_valid() {
            self.state.set_status(
                "Please enter a 5-letter word".to_string(),
                MessageType::Warning,
            );
            return Ok(());
        }

        let guess_word = self.state.input.clone();

        // Clear input immediately for better UX
        self.state.clear_input();

        // Start feedback input process
        self.feedback_manager
            .start_feedback_input(guess_word.clone(), &mut self.state);

        self.state.add_log(
            LogLevel::Info,
            format!(
                "Submitted guess: {} - waiting for feedback",
                guess_word.to_uppercase()
            ),
        );

        Ok(())
    }

    /// Get the best first guess
    async fn get_first_guess(&mut self) -> Result<()> {
        self.state
            .add_log(LogLevel::Info, "Getting best first guess...".to_string());

        match self.app_service.get_best_first_guess() {
            Ok(guess) => {
                self.state.set_suggestion(Some(guess.to_string()));
                self.state.set_status(
                    format!("Best first guess: {}", guess.to_string().to_uppercase()),
                    MessageType::Success,
                );
                self.state.add_log(
                    LogLevel::Info,
                    format!("Got first guess: {}", guess.to_string().to_uppercase()),
                );
            }
            Err(e) => {
                self.state.set_status(
                    format!("Error getting first guess: {}", e),
                    MessageType::Error,
                );
                self.state
                    .add_log(LogLevel::Error, format!("First guess error: {}", e));
            }
        }

        Ok(())
    }

    /// Show detailed statistics
    async fn show_detailed_stats(&mut self) -> Result<()> {
        let stats_text = format!(
            "Detailed Statistics:\n• Total Guesses: {}\n• Remaining Words: {}\n• Average Remaining: {:.1}\n• Entropy Values: {:?}",
            self.state.stats.total_guesses,
            self.state.remaining_words,
            self.state.stats.average_remaining_words,
            self.state
                .stats
                .entropy_values
                .iter()
                .take(5)
                .collect::<Vec<_>>()
        );

        self.state.set_status(stats_text, MessageType::Info);
        self.state
            .add_log(LogLevel::Info, "Displayed detailed statistics".to_string());

        Ok(())
    }

    /// Reset the game
    async fn reset_game(&mut self) -> Result<()> {
        self.state = TuiState::new();
        self.app_service = Container::new().create_application_service().await?;
        self.feedback_manager = FeedbackInputManager::new();

        self.state.add_log(LogLevel::Info, "Game reset".to_string());
        self.state
            .set_status("Game reset successfully".to_string(), MessageType::Success);

        // Get new first guess
        self.get_first_guess().await?;

        Ok(())
    }

    /// Periodic update function
    async fn update(&mut self) -> Result<()> {
        // Clear temporary status messages after some time
        if let Some(_status) = &self.state.status_message {
            // This is a simple implementation - in a real app you might want
            // to track timestamp and clear after a certain duration
        }

        Ok(())
    }

    /// Add a guess result to the game
    pub async fn add_guess_result(&mut self, word: String, feedback: String) -> Result<()> {
        // Parse the word and feedback
        let word_obj =
            Word::from_str(&word).map_err(|e| anyhow::anyhow!("Invalid word '{}': {}", word, e))?;

        let feedback_pattern = FeedbackPattern::from_code_string(&feedback)
            .map_err(|e| anyhow::anyhow!("Invalid feedback '{}': {}", feedback, e))?;

        // Add to application service
        let result = self.app_service.execute(Command::AddGuessResult {
            word: word_obj,
            feedback: feedback_pattern.clone(),
        })?;

        if let CommandResult::GuessResultAdded { remaining_words } = result {
            // Update UI state
            self.state
                .add_guess(word.clone(), feedback.clone(), remaining_words);
            self.state.update_remaining_words(remaining_words, vec![]); // TODO: Get actual sample

            // Check if solved
            if feedback == "22222" {
                self.state.set_solved(true);
                self.state.set_status(
                    format!(
                        "🎉 Congratulations! You solved it with '{}'!  Press any key to exit.",
                        word.to_uppercase()
                    ),
                    MessageType::Success,
                );
            } else {
                // Get next best guess
                self.get_next_guess().await?;
            }

            self.state.add_log(
                LogLevel::Info,
                format!(
                    "Added guess: {} -> {} ({} remaining)",
                    word.to_uppercase(),
                    feedback,
                    remaining_words
                ),
            );
        }

        Ok(())
    }

    /// Get the next best guess
    async fn get_next_guess(&mut self) -> Result<()> {
        match self.app_service.execute(Command::GetBestGuess) {
            Ok(CommandResult::BestGuess {
                word, confidence, ..
            }) => {
                self.state.set_suggestion(Some(word.to_string()));
                // We'll need to get remaining words count separately

                self.state.set_status(
                    format!(
                        "Next best guess: {} (confidence: {:.2})",
                        word.to_string().to_uppercase(),
                        confidence
                    ),
                    MessageType::Success,
                );
            }
            Ok(_) => {
                self.state.set_status(
                    "Unexpected response from solver".to_string(),
                    MessageType::Warning,
                );
            }
            Err(e) => {
                self.state.set_status(
                    format!("Error getting next guess: {}", e),
                    MessageType::Error,
                );
            }
        }

        Ok(())
    }
}

impl Drop for TuiApp {
    fn drop(&mut self) {
        // Cleanup terminal
        if let Err(e) = disable_raw_mode() {
            eprintln!("Failed to disable raw mode: {}", e);
        }
        if let Err(e) = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        ) {
            eprintln!("Failed to cleanup terminal: {}", e);
        }
    }
}

/// Helper function to run the TUI application
pub async fn run_tui() -> Result<()> {
    let mut app = TuiApp::new().await?;
    let run_result = app.run().await;

    // Capture needed info before dropping the app (which tears down the TUI)
    let solved = app.state.is_solved;
    let history: Vec<(String, String)> = app
        .state
        .guess_history
        .iter()
        .map(|e| (e.word.clone(), e.feedback.clone()))
        .collect();

    // Explicitly drop TUI to leave alternate screen and raw mode
    drop(app);

    // Print after returning to the normal terminal screen
    if solved {
        // Show only per-guess feedback like real Wordle (no legend/message/attempts)
        if !history.is_empty() {
            print_history_summary(&history);
        }
    }

    run_result
}

fn print_history_summary(rows: &[(String, String)]) {
    println!("");
    for (i, (word, feedback)) in rows.iter().enumerate() {
        let squares = format_feedback_squares(feedback);
        println!("{:>2}) {:<8} {}", i + 1, word.to_uppercase(), squares);
    }
    println!();
}

fn format_feedback_squares(feedback_code: &str) -> String {
    // Use fixed-width ANSI background-colored cells to avoid emoji width issues.
    // Each cell is two spaces with background color; separated by one normal space.
    const RESET: &str = "\x1b[0m";
    // Basic, widely-supported colors
    const GREEN_BG: &str = "\x1b[42m"; // HIT
    const YELLOW_BG: &str = "\x1b[43m"; // BITE
    const GRAY_BG: &str = "\x1b[100m"; // ABSENT (bright black)

    let mut out = String::with_capacity(5 * (2 + 1) * 4); // rough capacity accounting for ANSI
    for (idx, ch) in feedback_code.chars().take(5).enumerate() {
        if idx > 0 {
            out.push(' ');
        }
        let bg = match ch {
            '2' => GREEN_BG,
            '1' => YELLOW_BG,
            _ => GRAY_BG,
        };
        out.push_str(bg);
        out.push_str("  ");
        out.push_str(RESET);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_app_creation() {
        // This test might fail in CI environments without a proper terminal
        // but it's useful for local development
        if std::env::var("CI").is_err() {
            let result = TuiApp::new().await;
            // We expect this to either succeed or fail with a terminal size error
            match result {
                Ok(_) => {
                    // App created successfully
                }
                Err(e) => {
                    // Should be a terminal size error in test environments
                    assert!(
                        e.to_string().contains("Terminal too small")
                            || e.to_string().contains("not a tty")
                    );
                }
            }
        }
    }
}
