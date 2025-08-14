use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Gauge, List, ListItem, Paragraph, Wrap},
};

use crate::presentation::tui::mode::InteractionMode;
use crate::presentation::tui::state::{LogLevel, MessageType, TuiState};

/// Color scheme for the TUI
pub struct Colors;

impl Colors {
    pub const BACKGROUND: Color = Color::Black;
    pub const FOREGROUND: Color = Color::White;
    pub const ACCENT: Color = Color::Cyan;
    pub const SUCCESS: Color = Color::Green;
    pub const WARNING: Color = Color::Yellow;
    pub const ERROR: Color = Color::Red;
    pub const INFO: Color = Color::Blue;
    pub const MUTED: Color = Color::Gray;

    // Wordle colors
    pub const CORRECT: Color = Color::Green;
    pub const PRESENT: Color = Color::Yellow;
    pub const ABSENT: Color = Color::Gray;
    pub const INPUT: Color = Color::Cyan;
}

/// Render the main title bar
pub fn render_title(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new("🎯 Modern Wordle Solver · Press 'h' for help")
        .style(
            Style::default()
                .fg(Colors::ACCENT)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Colors::ACCENT))
                .title(" Wordle AI ")
                .title_style(
                    Style::default()
                        .fg(Colors::ACCENT)
                        .add_modifier(Modifier::BOLD),
                ),
        );
    frame.render_widget(title, area);
}

/// Render the input area
pub fn render_input(frame: &mut Frame, area: Rect, state: &TuiState) {
    let input_text = if state.input.len() < 5 {
        format!(
            "{}{}",
            state.input.to_uppercase(),
            "█".repeat(5 - state.input.len())
        )
    } else {
        state.input.to_uppercase()
    };

    let style = if state.is_input_valid() {
        Style::default().fg(Colors::SUCCESS)
    } else {
        Style::default().fg(Colors::INPUT)
    };

    let input = Paragraph::new(input_text)
        .style(style.add_modifier(Modifier::BOLD))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Colors::ACCENT))
                .title(" Enter Your Guess ")
                .title_style(Style::default().fg(Colors::ACCENT)),
        );

    frame.render_widget(input, area);
}

/// Render the current suggestion
pub fn render_suggestion(frame: &mut Frame, area: Rect, state: &TuiState) {
    let suggestion_text = match &state.current_suggestion {
        Some(word) => format!("💡 Suggested: {}", word.to_uppercase()),
        None => "💡 Getting suggestion...".to_string(),
    };

    let suggestion = Paragraph::new(suggestion_text)
        .style(
            Style::default()
                .fg(Colors::WARNING)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Colors::WARNING))
                .title(" AI Suggestion ")
                .title_style(Style::default().fg(Colors::WARNING)),
        );

    frame.render_widget(suggestion, area);
}

/// Render the guess history
pub fn render_history(frame: &mut Frame, area: Rect, state: &TuiState) {
    let history_items: Vec<ListItem> = state
        .guess_history
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let feedback_line = colorize_feedback(&entry.feedback, &entry.word);
            let mut full_line = vec![Span::styled(
                format!("{}. ", i + 1),
                Style::default().fg(Colors::MUTED),
            )];
            full_line.extend(feedback_line.spans);
            full_line.push(Span::styled(
                format!(" ({} left)", entry.remaining_count),
                Style::default().fg(Colors::MUTED),
            ));

            ListItem::new(Line::from(full_line))
        })
        .collect();

    let history = List::new(history_items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Colors::INFO))
            .title(format!(" Guess History ({}) ", state.guess_history.len()))
            .title_style(Style::default().fg(Colors::INFO)),
    );

    frame.render_widget(history, area);
}

/// Render statistics panel
pub fn render_stats(frame: &mut Frame, area: Rect, state: &TuiState) {
    let stats_text = vec![
        Line::from(vec![
            Span::styled("Remaining Words: ", Style::default().fg(Colors::FOREGROUND)),
            Span::styled(
                format!("{}", state.remaining_words),
                Style::default()
                    .fg(Colors::ACCENT)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Total Guesses: ", Style::default().fg(Colors::FOREGROUND)),
            Span::styled(
                format!("{}", state.stats.total_guesses),
                Style::default()
                    .fg(Colors::INFO)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Avg. Remaining: ", Style::default().fg(Colors::FOREGROUND)),
            Span::styled(
                format!("{:.1}", state.stats.average_remaining_words),
                Style::default()
                    .fg(Colors::WARNING)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Status: ", Style::default().fg(Colors::FOREGROUND)),
            Span::styled(
                if state.is_solved {
                    "SOLVED! 🎉"
                } else {
                    "In Progress..."
                },
                if state.is_solved {
                    Style::default()
                        .fg(Colors::SUCCESS)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Colors::WARNING)
                },
            ),
        ]),
    ];

    let stats = Paragraph::new(stats_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Colors::SUCCESS))
            .title(" Statistics ")
            .title_style(Style::default().fg(Colors::SUCCESS)),
    );

    frame.render_widget(stats, area);
}

/// Render remaining words sample
pub fn render_remaining_words(frame: &mut Frame, area: Rect, state: &TuiState) {
    let words_text = if state.remaining_words_sample.is_empty() {
        "No words available".to_string()
    } else {
        state
            .remaining_words_sample
            .iter()
            .take(10)
            .map(|w| w.to_uppercase())
            .collect::<Vec<_>>()
            .join(", ")
    };

    let remaining = Paragraph::new(words_text)
        .style(Style::default().fg(Colors::MUTED))
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Colors::MUTED))
                .title(" Possible Words (Sample) ")
                .title_style(Style::default().fg(Colors::MUTED)),
        );

    frame.render_widget(remaining, area);
}

/// Render help popup
pub fn render_help(frame: &mut Frame, area: Rect, state: &TuiState) {
    let mode = state.interaction_mode();
    let mode_commands = mode.available_commands();

    let mut help_text = vec![
        Line::from("🎯 Modern Wordle Solver - Help"),
        Line::from(""),
        Line::from(format!(
            "Current Mode: {} ({})",
            mode.name(),
            mode.description()
        )),
        Line::from(""),
        Line::from("📝 Mode Switching:"),
        Line::from("  • Esc/Tab: Switch between INPUT and OPERATION modes"),
        Line::from(""),
        Line::from(format!("🔧 {} Mode Commands:", mode.name())),
    ];

    // Add mode-specific commands
    for (key, description) in mode_commands {
        help_text.push(Line::from(format!("  • {}: {}", key, description)));
    }

    help_text.extend(vec![
        Line::from(""),
        Line::from("🌐 Global Commands:"),
        Line::from("  • Ctrl+Q/Ctrl+C: Quit application"),
        Line::from(""),
        Line::from("🎨 Feedback Colors:"),
        Line::from(vec![
            Span::styled("  • Green: ", Style::default().fg(Colors::FOREGROUND)),
            Span::styled("Correct position", Style::default().fg(Colors::CORRECT)),
        ]),
        Line::from(vec![
            Span::styled("  • Yellow: ", Style::default().fg(Colors::FOREGROUND)),
            Span::styled("Wrong position", Style::default().fg(Colors::PRESENT)),
        ]),
        Line::from(vec![
            Span::styled("  • Gray: ", Style::default().fg(Colors::FOREGROUND)),
            Span::styled("Not in word", Style::default().fg(Colors::ABSENT)),
        ]),
        Line::from(""),
    ]);

    // Add closing message only in INPUT mode (when help is toggled manually)
    if mode.is_input() {
        help_text.push(Line::from("Press any key to close help..."));
    } else {
        help_text.push(Line::from("Available commands for OPERATION mode"));
    }

    let help_title = if mode.is_operation() {
        format!(" {} Mode Help ", mode.name())
    } else {
        " Help ".to_string()
    };

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Colors::FOREGROUND))
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Colors::INFO))
                .title(help_title)
                .title_style(
                    Style::default()
                        .fg(Colors::INFO)
                        .add_modifier(Modifier::BOLD),
                ),
        );

    // Clear the area first
    frame.render_widget(Clear, area);
    frame.render_widget(help, area);
}

/// Render status message
pub fn render_status(frame: &mut Frame, area: Rect, state: &TuiState) {
    if let Some(status) = &state.status_message {
        let style = match status.message_type {
            MessageType::Info => Style::default().fg(Colors::INFO),
            MessageType::Success => Style::default().fg(Colors::SUCCESS),
            MessageType::Warning => Style::default().fg(Colors::WARNING),
            MessageType::Error => Style::default().fg(Colors::ERROR),
        };

        let status_widget = Paragraph::new(status.text.clone())
            .style(style.add_modifier(Modifier::BOLD))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(style),
            );

        frame.render_widget(status_widget, area);
    }
}

/// Render log messages
pub fn render_logs(frame: &mut Frame, area: Rect, state: &TuiState) {
    let log_items: Vec<ListItem> = state
        .get_recent_logs(area.height.saturating_sub(2) as usize)
        .iter()
        .map(|log| {
            let style = match log.level {
                LogLevel::Info => Style::default().fg(Colors::INFO),
                LogLevel::Debug => Style::default().fg(Colors::MUTED),
                LogLevel::Warning => Style::default().fg(Colors::WARNING),
                LogLevel::Error => Style::default().fg(Colors::ERROR),
            };

            let line = Line::from(vec![
                Span::styled(
                    format!("[{:?}] ", log.level),
                    style.add_modifier(Modifier::BOLD),
                ),
                Span::styled(log.message.clone(), style),
            ]);
            ListItem::new(line)
        })
        .collect();

    let logs = List::new(log_items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Colors::MUTED))
            .title(" Logs ")
            .title_style(Style::default().fg(Colors::MUTED)),
    );

    frame.render_widget(logs, area);
}

/// Render progress bar for remaining words
pub fn render_progress(frame: &mut Frame, area: Rect, state: &TuiState) {
    // Calculate progress (inverse of remaining words)
    let total_words = 2315.0; // Approximate total Wordle words
    let progress = if state.remaining_words > 0 {
        1.0 - (state.remaining_words as f64 / total_words).min(1.0)
    } else {
        1.0
    };

    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Colors::ACCENT))
                .title(" Solving Progress ")
                .title_style(Style::default().fg(Colors::ACCENT)),
        )
        .gauge_style(Style::default().fg(Colors::SUCCESS))
        .percent((progress * 100.0) as u16)
        .label(format!("{:.1}% Complete", progress * 100.0));

    frame.render_widget(gauge, area);
}

/// Helper function to colorize feedback text
fn colorize_feedback(feedback: &str, word: &str) -> Line<'static> {
    // Render letters as rounded "tiles" with colored backgrounds
    let mut spans = Vec::new();

    for (c, f) in word.chars().zip(feedback.chars()) {
        let bg = match f {
            '2' => Colors::CORRECT,
            '1' => Colors::PRESENT,
            '0' => Colors::ABSENT,
            _ => Colors::MUTED,
        };

        spans.push(Span::styled(
            format!(" {} ", c.to_uppercase()),
            Style::default()
                .fg(Color::Black)
                .bg(bg)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(" "));
    }

    if !spans.is_empty() {
        spans.pop();
    }

    Line::from(spans)
}

/// Render feedback input area (when in feedback mode)
pub fn render_feedback_input(
    frame: &mut Frame,
    area: Rect,
    current_guess: &str,
    feedback_input: &str,
    cursor_pos: usize,
) {
    let title = format!(" Feedback for '{}' ", current_guess.to_uppercase());

    // Build a line of 5 tiles representing feedback digits with a blinking cursor tile
    let mut spans: Vec<Span> = Vec::with_capacity(9);
    for i in 0..5 {
        let (label, bg) = match feedback_input.chars().nth(i) {
            Some('2') => (" 2 ", Colors::CORRECT),
            Some('1') => (" 1 ", Colors::PRESENT),
            Some('0') => (" 0 ", Colors::ABSENT),
            _ => ("   ", Colors::MUTED),
        };

        let mut style = Style::default()
            .fg(Color::Black)
            .bg(bg)
            .add_modifier(Modifier::BOLD);
        let mut text = label.to_string();

        // Visual cursor: invert colors on the current tile
        if i == cursor_pos.min(4) && feedback_input.len() < 5 {
            style = Style::default()
                .fg(Colors::ACCENT)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD | Modifier::REVERSED);
            text = " ▌ ".to_string();
        }

        spans.push(Span::styled(text, style));
        if i < 4 {
            spans.push(Span::raw(" "));
        }
    }

    let feedback_widget = Paragraph::new(Line::from(spans))
        .style(Style::default().fg(Colors::FOREGROUND))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Colors::WARNING))
                .title(title)
                .title_style(
                    Style::default()
                        .fg(Colors::WARNING)
                        .add_modifier(Modifier::BOLD),
                ),
        );

    frame.render_widget(feedback_widget, area);
}

/// Render feedback input help
pub fn render_feedback_help(frame: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from("🎨 Feedback Input:"),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "0",
                Style::default()
                    .fg(Colors::ABSENT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " = Gray (letter not in word)",
                Style::default().fg(Colors::FOREGROUND),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "1",
                Style::default()
                    .fg(Colors::PRESENT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " = Yellow (letter in word, wrong position)",
                Style::default().fg(Colors::FOREGROUND),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "2",
                Style::default()
                    .fg(Colors::CORRECT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " = Green (letter in correct position)",
                Style::default().fg(Colors::FOREGROUND),
            ),
        ]),
        Line::from(""),
        Line::from("⌨️ Controls:"),
        Line::from("  • 0, 1, 2: Enter feedback"),
        Line::from("  • Enter: Submit feedback"),
        Line::from("  • Backspace: Delete character"),
        Line::from("  • Esc: Cancel feedback input"),
        Line::from(""),
        Line::from("🚀 Quick patterns:"),
        Line::from("  • Type 'none' + Enter: All gray (00000)"),
        Line::from("  • Type 'correct' + Enter: All green (22222)"),
    ];

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Colors::FOREGROUND))
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Colors::INFO))
                .title(" Feedback Help ")
                .title_style(Style::default().fg(Colors::INFO)),
        );

    frame.render_widget(help, area);
}
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Render the current interaction mode
pub fn render_mode_indicator(frame: &mut Frame, area: Rect, state: &TuiState) {
    let mode = state.interaction_mode();
    let (mode_text, mode_color) = match mode {
        InteractionMode::Input => ("INPUT", Colors::SUCCESS),
        InteractionMode::Operation => ("OPERATION", Colors::WARNING),
    };

    let mode_indicator = Paragraph::new(format!("Mode: {}", mode_text))
        .style(Style::default().fg(mode_color).add_modifier(Modifier::BOLD))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(mode_color))
                .title(format!(" {} Mode ", mode_text))
                .title_style(Style::default().fg(mode_color).add_modifier(Modifier::BOLD)),
        );

    frame.render_widget(mode_indicator, area);
}

/// Render a compact footer command bar with common shortcuts
pub fn render_footer(frame: &mut Frame, area: Rect, state: &TuiState) {
    let mode = state.interaction_mode();
    let key = |k: &str| {
        Span::styled(
            format!(" {} ", k),
            Style::default()
                .fg(Colors::ACCENT)
                .add_modifier(Modifier::BOLD),
        )
    };
    let sep = Span::styled("  •  ", Style::default().fg(Colors::MUTED));

    let mut spans: Vec<Span> = Vec::new();
    spans.push(key("Esc/Tab"));
    spans.push(Span::styled(
        " Switch Mode",
        Style::default().fg(Colors::MUTED),
    ));
    spans.push(sep.clone());
    spans.push(key("Enter"));
    spans.push(Span::styled(" Submit", Style::default().fg(Colors::MUTED)));
    spans.push(sep.clone());
    spans.push(key("h"));
    spans.push(Span::styled(" Help", Style::default().fg(Colors::MUTED)));
    spans.push(sep.clone());
    spans.push(key("f"));
    spans.push(Span::styled(" First", Style::default().fg(Colors::MUTED)));
    spans.push(sep.clone());
    spans.push(key("s"));
    spans.push(Span::styled(" Stats", Style::default().fg(Colors::MUTED)));
    spans.push(sep.clone());
    spans.push(key("r"));
    spans.push(Span::styled(" Reset", Style::default().fg(Colors::MUTED)));
    spans.push(sep);
    spans.push(key("q"));
    spans.push(Span::styled(" Quit", Style::default().fg(Colors::MUTED)));

    // Mode hint at the end
    spans.push(Span::styled(
        format!("   [{} mode]", mode.name()),
        Style::default()
            .fg(Colors::INFO)
            .add_modifier(Modifier::ITALIC),
    ));

    let footer = Paragraph::new(Line::from(spans))
        .style(Style::default().fg(Colors::FOREGROUND))
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(Colors::MUTED)),
        );

    frame.render_widget(footer, area);
}
