use crate::app::state::{AppState, GameResult};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

/// Draw the main UI with sophisticated design
pub fn draw(f: &mut Frame, state: &AppState) {
    // Main layout with better proportions
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), // Left panel (game area)
            Constraint::Percentage(40), // Right panel (stats and info)
        ])
        .split(f.size());

    // Left panel layout
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(5), // Title and subtitle
            Constraint::Length(8), // Wordle board
            Constraint::Length(4), // Current suggestion
            Constraint::Length(4), // Input area
            Constraint::Min(0),    // Remaining space
        ])
        .split(main_chunks[0]);

    // Right panel layout
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(8), // Statistics
            Constraint::Length(6), // Game info
            Constraint::Min(0),    // History
            Constraint::Length(3), // Status bar
        ])
        .split(main_chunks[1]);

    // Draw all components
    draw_header(f, left_chunks[0]);
    draw_wordle_board(f, left_chunks[1], state);
    draw_current_suggestion(f, left_chunks[2], state);
    draw_input_area(f, left_chunks[3], state);

    // Draw game result or candidate words in the remaining left space
    if state.game_result != GameResult::InProgress {
        draw_game_result_panel(f, left_chunks[4], state);
    } else {
        draw_candidate_words_panel(f, left_chunks[4], state);
    }

    draw_statistics(f, right_chunks[0], state);
    draw_game_info(f, right_chunks[1], state);
    draw_history(f, right_chunks[2], state);
    draw_status_bar(f, right_chunks[3], state);
}

fn draw_header(f: &mut Frame, area: Rect) {
    let header_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(100, 149, 237))) // Cornflower blue
        .style(Style::default().bg(Color::Rgb(25, 25, 35))); // Dark background

    let header_text = vec![
        Line::from(vec![Span::styled(
            "🎯 WORDLE AI SOLVER",
            Style::default()
                .fg(Color::Rgb(255, 215, 0)) // Gold
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            "Advanced AI-Powered Word Guessing",
            Style::default()
                .fg(Color::Rgb(176, 196, 222)) // Light steel blue
                .add_modifier(Modifier::ITALIC),
        )]),
    ];

    let paragraph = Paragraph::new(header_text)
        .alignment(Alignment::Center)
        .block(header_block);

    f.render_widget(paragraph, area);
}

fn draw_wordle_board(f: &mut Frame, area: Rect, state: &AppState) {
    let board_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(120, 120, 120)))
        .title(" Wordle Board ")
        .title_style(
            Style::default()
                .fg(Color::Rgb(255, 215, 0))
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(Color::Rgb(30, 30, 40)));

    // Create the board content
    let mut board_lines = Vec::new();

    // Display up to 6 rows (standard Wordle)
    for i in 0..6 {
        if i < state.guess_history.len() {
            let (word, result) = &state.guess_history[i];
            let formatted_line = format_wordle_row(word, result);
            board_lines.push(Line::from(formatted_line));
        } else {
            // Empty row
            let empty_row = vec![
                Span::raw("  "),
                Span::styled("□", Style::default().fg(Color::Rgb(80, 80, 80))),
                Span::raw(" "),
                Span::styled("□", Style::default().fg(Color::Rgb(80, 80, 80))),
                Span::raw(" "),
                Span::styled("□", Style::default().fg(Color::Rgb(80, 80, 80))),
                Span::raw(" "),
                Span::styled("□", Style::default().fg(Color::Rgb(80, 80, 80))),
                Span::raw(" "),
                Span::styled("□", Style::default().fg(Color::Rgb(80, 80, 80))),
                Span::raw("  "),
            ];
            board_lines.push(Line::from(empty_row));
        }
    }

    let paragraph = Paragraph::new(board_lines)
        .alignment(Alignment::Center)
        .block(board_block);

    f.render_widget(paragraph, area);
}

fn draw_current_suggestion(f: &mut Frame, area: Rect, state: &AppState) {
    let suggestion_text = match &state.game_result {
        GameResult::InProgress => {
            if let Some(ref suggestion) = state.current_suggestion {
                vec![
                    Line::from(vec![
                        Span::styled("💡 ", Style::default().fg(Color::Rgb(255, 215, 0))),
                        Span::styled(
                            "Next Best Guess",
                            Style::default().fg(Color::Rgb(176, 196, 222)),
                        ),
                    ]),
                    Line::from(vec![Span::styled(
                        suggestion.to_uppercase(),
                        Style::default()
                            .fg(Color::Rgb(50, 205, 50)) // Lime green
                            .add_modifier(Modifier::BOLD)
                            .add_modifier(Modifier::UNDERLINED),
                    )]),
                ]
            } else {
                vec![Line::from(vec![
                    Span::styled("🤔 ", Style::default().fg(Color::Rgb(255, 165, 0))),
                    Span::styled(
                        "Calculating optimal guess...",
                        Style::default()
                            .fg(Color::Rgb(255, 165, 0))
                            .add_modifier(Modifier::SLOW_BLINK),
                    ),
                ])]
            }
        }
        GameResult::Won { .. } => {
            vec![
                Line::from(vec![
                    Span::styled("🎉 ", Style::default().fg(Color::Rgb(106, 170, 100))),
                    Span::styled(
                        "Puzzle Solved!",
                        Style::default()
                            .fg(Color::Rgb(106, 170, 100))
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![Span::styled(
                    "Press 'r' to restart",
                    Style::default()
                        .fg(Color::Rgb(176, 196, 222))
                        .add_modifier(Modifier::ITALIC),
                )]),
            ]
        }
        GameResult::Failed { .. } => {
            vec![
                Line::from(vec![
                    Span::styled("💔 ", Style::default().fg(Color::Rgb(220, 20, 60))),
                    Span::styled(
                        "Game Over",
                        Style::default()
                            .fg(Color::Rgb(220, 20, 60))
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![Span::styled(
                    "Press 'r' to restart",
                    Style::default()
                        .fg(Color::Rgb(176, 196, 222))
                        .add_modifier(Modifier::ITALIC),
                )]),
            ]
        }
    };

    let (border_color, bg_color) = match &state.game_result {
        GameResult::InProgress => (Color::Rgb(50, 205, 50), Color::Rgb(25, 35, 25)),
        GameResult::Won { .. } => (Color::Rgb(106, 170, 100), Color::Rgb(25, 35, 25)),
        GameResult::Failed { .. } => (Color::Rgb(220, 20, 60), Color::Rgb(35, 25, 25)),
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(bg_color));

    let paragraph = Paragraph::new(suggestion_text)
        .alignment(Alignment::Center)
        .block(block);

    f.render_widget(paragraph, area);
}

fn draw_input_area(f: &mut Frame, area: Rect, state: &AppState) {
    let (input_text, help_text, is_enabled) = match &state.game_result {
        GameResult::InProgress => {
            let input_text = state
                .input_buffer
                .as_ref()
                .unwrap_or(&String::new())
                .clone();

            let help_text = if input_text.is_empty() {
                "Format: word result (e.g., 'adieu 20100')".to_string()
            } else {
                input_text.clone()
            };

            (input_text, help_text, true)
        }
        GameResult::Won { .. } => (
            "".to_string(),
            "Game completed - Press 'r' to restart".to_string(),
            false,
        ),
        GameResult::Failed { .. } => (
            "".to_string(),
            "Game over - Press 'r' to restart".to_string(),
            false,
        ),
    };

    let (border_color, bg_color, text_color) = if is_enabled {
        (
            Color::Rgb(100, 149, 237),
            Color::Rgb(30, 30, 45),
            Color::Rgb(255, 255, 255),
        )
    } else {
        (
            Color::Rgb(120, 120, 120),
            Color::Rgb(40, 40, 40),
            Color::Rgb(150, 150, 150),
        )
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(" 📝 Input ")
        .title_style(
            Style::default()
                .fg(if is_enabled {
                    Color::Rgb(255, 215, 0)
                } else {
                    Color::Rgb(180, 180, 180)
                })
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(bg_color));

    let display_color = if input_text.is_empty() && is_enabled {
        Color::Rgb(128, 128, 128)
    } else {
        text_color
    };

    let input = Paragraph::new(help_text)
        .style(Style::default().fg(display_color))
        .block(block);

    f.render_widget(input, area);
}

fn draw_statistics(f: &mut Frame, area: Rect, state: &AppState) {
    let stats_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(255, 20, 147))) // Deep pink
        .title(" 📊 Game Statistics ")
        .title_style(
            Style::default()
                .fg(Color::Rgb(255, 215, 0))
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(Color::Rgb(35, 25, 35)));

    let attempts = state.guess_history.len();
    let progress = if attempts > 0 {
        (attempts as f64 / 6.0) * 100.0
    } else {
        0.0
    };

    let stats_text = vec![
        Line::from(vec![
            Span::styled("Attempts: ", Style::default().fg(Color::Rgb(176, 196, 222))),
            Span::styled(
                format!("{}/6", attempts),
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Progress: ", Style::default().fg(Color::Rgb(176, 196, 222))),
            Span::styled(
                format!("{:.1}%", progress),
                Style::default().fg(match attempts {
                    0..=2 => Color::Rgb(50, 205, 50), // Green
                    3..=4 => Color::Rgb(255, 165, 0), // Orange
                    _ => Color::Rgb(255, 69, 0),      // Red
                }),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "Efficiency: ",
                Style::default().fg(Color::Rgb(176, 196, 222)),
            ),
            Span::styled(
                match attempts {
                    0..=2 => "Excellent 🏆",
                    3..=4 => "Good 👍",
                    5 => "Fair 😐",
                    _ => "Challenging 😅",
                },
                Style::default().fg(Color::Rgb(255, 255, 255)),
            ),
        ]),
    ];

    let paragraph = Paragraph::new(stats_text).block(stats_block);

    f.render_widget(paragraph, area);
}

fn draw_game_info(f: &mut Frame, area: Rect, state: &AppState) {
    let info_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(64, 224, 208))) // Turquoise
        .title(" ℹ️  Game Info ")
        .title_style(
            Style::default()
                .fg(Color::Rgb(255, 215, 0))
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(Color::Rgb(25, 35, 35)));

    let info_text = vec![
        Line::from(vec![
            Span::styled(
                "Remaining: ",
                Style::default().fg(Color::Rgb(176, 196, 222)),
            ),
            Span::styled(
                format!("{} words", state.remaining_words_count),
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Strategy: ", Style::default().fg(Color::Rgb(176, 196, 222))),
            Span::styled(
                "Entropy Maximization",
                Style::default().fg(Color::Rgb(144, 238, 144)), // Light green
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "Algorithm: ",
                Style::default().fg(Color::Rgb(176, 196, 222)),
            ),
            Span::styled(
                "Information Theory",
                Style::default().fg(Color::Rgb(255, 182, 193)), // Light pink
            ),
        ]),
    ];

    let paragraph = Paragraph::new(info_text).block(info_block);

    f.render_widget(paragraph, area);
}

fn draw_history(f: &mut Frame, area: Rect, state: &AppState) {
    let history_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(186, 85, 211))) // Medium orchid
        .title(" 📜 Guess History ")
        .title_style(
            Style::default()
                .fg(Color::Rgb(255, 215, 0))
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(Color::Rgb(35, 25, 40)));

    let history_items: Vec<ListItem> = state
        .guess_history
        .iter()
        .enumerate()
        .map(|(i, (word, result))| {
            let result_emojis = format_result_emojis(result);
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{:2}.", i + 1),
                    Style::default().fg(Color::Rgb(128, 128, 128)),
                ),
                Span::raw(" "),
                Span::styled(
                    word.to_uppercase(),
                    Style::default()
                        .fg(Color::Rgb(255, 255, 255))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::raw(result_emojis),
            ]))
        })
        .collect();

    let history = List::new(history_items)
        .block(history_block)
        .style(Style::default().fg(Color::Rgb(255, 255, 255)));

    f.render_widget(history, area);
}

fn draw_status_bar(f: &mut Frame, area: Rect, state: &AppState) {
    let status_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(70, 130, 180))) // Steel blue
        .style(Style::default().bg(Color::Rgb(25, 25, 35)));

    let status_text = match &state.game_result {
        GameResult::InProgress => {
            vec![Line::from(vec![
                Span::styled("🎮 ", Style::default().fg(Color::Rgb(255, 215, 0))),
                Span::styled("Controls: ", Style::default().fg(Color::Rgb(176, 196, 222))),
                Span::styled(
                    "[Enter]",
                    Style::default()
                        .fg(Color::Rgb(50, 205, 50))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" Submit • ", Style::default().fg(Color::Rgb(176, 196, 222))),
                Span::styled(
                    "[Q]",
                    Style::default()
                        .fg(Color::Rgb(255, 69, 0))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" Quit", Style::default().fg(Color::Rgb(176, 196, 222))),
            ])]
        }
        GameResult::Won { .. } | GameResult::Failed { .. } => {
            vec![Line::from(vec![
                Span::styled("🎮 ", Style::default().fg(Color::Rgb(255, 215, 0))),
                Span::styled("Controls: ", Style::default().fg(Color::Rgb(176, 196, 222))),
                Span::styled(
                    "[R]",
                    Style::default()
                        .fg(Color::Rgb(50, 205, 50))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    " Restart • ",
                    Style::default().fg(Color::Rgb(176, 196, 222)),
                ),
                Span::styled(
                    "[Q]",
                    Style::default()
                        .fg(Color::Rgb(255, 69, 0))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" Quit", Style::default().fg(Color::Rgb(176, 196, 222))),
            ])]
        }
    };

    let paragraph = Paragraph::new(status_text)
        .alignment(Alignment::Center)
        .block(status_block);

    f.render_widget(paragraph, area);
}

fn format_result_emojis(result: &str) -> String {
    result
        .chars()
        .map(|c| match c {
            '2' => "🟩", // Green - Correct
            '1' => "🟨", // Yellow - Present
            '0' => "⬜", // Gray - Absent
            _ => "❓",   // Unknown
        })
        .collect()
}

fn format_wordle_row(word: &str, result: &str) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    spans.push(Span::raw("  ")); // Left padding

    for (i, (ch, result_char)) in word.chars().zip(result.chars()).enumerate() {
        if i > 0 {
            spans.push(Span::raw(" "));
        }

        let (bg_color, fg_color) = match result_char {
            '2' => (Color::Rgb(106, 170, 100), Color::Rgb(255, 255, 255)), // Green
            '1' => (Color::Rgb(201, 180, 88), Color::Rgb(255, 255, 255)),  // Yellow
            '0' => (Color::Rgb(120, 124, 126), Color::Rgb(255, 255, 255)), // Gray
            _ => (Color::Rgb(60, 60, 60), Color::Rgb(200, 200, 200)),      // Unknown
        };

        spans.push(Span::styled(
            ch.to_uppercase().to_string(),
            Style::default()
                .bg(bg_color)
                .fg(fg_color)
                .add_modifier(Modifier::BOLD),
        ));
    }

    spans.push(Span::raw("  ")); // Right padding
    spans
}

/// Draw game result overlay (win/lose screen)
/// Draw game result panel in the left bottom area
fn draw_game_result_panel(f: &mut Frame, area: Rect, state: &AppState) {
    let (title, content, border_color, bg_color) = match &state.game_result {
        GameResult::Won { word, attempts } => {
            let title = " 🎉 VICTORY! 🎉 ";

            // Create performance message based on attempts
            let performance_msg = match *attempts {
                1 => "INCREDIBLE! Perfect! 🔥",
                2 => "AMAZING! Brilliant! ⭐",
                3 => "EXCELLENT! Great! 👏",
                4 => "GOOD! Well played! 👍",
                5 => "NICE! You got it! 😊",
                6 => "PHEW! Just made it! 😅",
                _ => "You solved it! 🎯",
            };

            let content = vec![
                Line::from(vec![Span::styled(
                    performance_msg,
                    Style::default()
                        .fg(Color::Rgb(255, 215, 0))
                        .add_modifier(Modifier::BOLD),
                )]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("🎯 Word: ", Style::default().fg(Color::Rgb(176, 196, 222))),
                    Span::styled(
                        word.to_uppercase(),
                        Style::default()
                            .fg(Color::Rgb(106, 170, 100))
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        "🎲 Attempts: ",
                        Style::default().fg(Color::Rgb(176, 196, 222)),
                    ),
                    Span::styled(
                        format!("{}/6", attempts),
                        Style::default()
                            .fg(Color::Rgb(255, 215, 0))
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        "📊 Success: ",
                        Style::default().fg(Color::Rgb(176, 196, 222)),
                    ),
                    Span::styled(
                        "100%",
                        Style::default()
                            .fg(Color::Rgb(106, 170, 100))
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Press ", Style::default().fg(Color::Rgb(176, 196, 222))),
                    Span::styled(
                        "[R]",
                        Style::default()
                            .fg(Color::Rgb(50, 205, 50))
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        " to restart",
                        Style::default().fg(Color::Rgb(176, 196, 222)),
                    ),
                ]),
            ];
            (
                title,
                content,
                Color::Rgb(106, 170, 100),
                Color::Rgb(25, 35, 25),
            )
        }
        GameResult::Failed { attempts } => {
            let title = " 💔 GAME OVER 💔 ";

            let failure_reason = if state.remaining_words_count == 0 {
                "No valid words match"
            } else {
                "Max attempts reached"
            };

            let content = vec![
                Line::from(vec![Span::styled(
                    "Better luck next time!",
                    Style::default()
                        .fg(Color::Rgb(255, 165, 0))
                        .add_modifier(Modifier::BOLD),
                )]),
                Line::from(""),
                Line::from(vec![
                    Span::styled(
                        "❌ Reason: ",
                        Style::default().fg(Color::Rgb(176, 196, 222)),
                    ),
                    Span::styled(
                        failure_reason,
                        Style::default()
                            .fg(Color::Rgb(220, 20, 60))
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        "🎲 Attempts: ",
                        Style::default().fg(Color::Rgb(176, 196, 222)),
                    ),
                    Span::styled(
                        format!("{}/6", attempts),
                        Style::default()
                            .fg(Color::Rgb(255, 165, 0))
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        "🔍 Remaining: ",
                        Style::default().fg(Color::Rgb(176, 196, 222)),
                    ),
                    Span::styled(
                        if state.remaining_words_count > 0 {
                            state.remaining_words_count.to_string()
                        } else {
                            "0 (Contradiction)".to_string()
                        },
                        Style::default()
                            .fg(if state.remaining_words_count > 0 {
                                Color::Rgb(255, 215, 0)
                            } else {
                                Color::Rgb(220, 20, 60)
                            })
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Press ", Style::default().fg(Color::Rgb(176, 196, 222))),
                    Span::styled(
                        "[R]",
                        Style::default()
                            .fg(Color::Rgb(50, 205, 50))
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        " to try again",
                        Style::default().fg(Color::Rgb(176, 196, 222)),
                    ),
                ]),
            ];
            (
                title,
                content,
                Color::Rgb(220, 20, 60),
                Color::Rgb(35, 25, 25),
            )
        }
        GameResult::InProgress => return, // Should not happen
    };

    // Create a beautiful panel block
    let panel_block = Block::default()
        .borders(Borders::ALL)
        .border_style(
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        )
        .title(title)
        .title_style(
            Style::default()
                .fg(Color::Rgb(255, 255, 255))
                .bg(border_color)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(bg_color));

    let paragraph = Paragraph::new(content)
        .alignment(Alignment::Left)
        .wrap(ratatui::widgets::Wrap { trim: true })
        .block(panel_block);

    f.render_widget(paragraph, area);
}

/// Draw candidate words panel in the left bottom area during game progress
fn draw_candidate_words_panel(f: &mut Frame, area: Rect, state: &AppState) {
    let panel_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(70, 130, 180))) // Steel blue
        .title(" 🎯 Top Candidates (by Entropy) ")
        .title_style(
            Style::default()
                .fg(Color::Rgb(255, 215, 0))
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(Color::Rgb(25, 25, 35))); // Dark background

    let mut content = Vec::new();

    if state.top_candidates.is_empty() {
        // Show loading message if no candidates yet
        content.push(Line::from(vec![Span::styled(
            "🔄 Calculating optimal candidates...",
            Style::default()
                .fg(Color::Rgb(176, 196, 222))
                .add_modifier(Modifier::ITALIC),
        )]));
    } else {
        // Show header
        content.push(Line::from(vec![Span::styled(
            "Word        Entropy",
            Style::default()
                .fg(Color::Rgb(100, 149, 237))
                .add_modifier(Modifier::BOLD),
        )]));
        content.push(Line::from(""));

        // Show top candidates (limit to fit in available space)
        let max_candidates =
            ((area.height as usize).saturating_sub(4)).min(state.top_candidates.len());

        for (i, (word, entropy)) in state.top_candidates.iter().take(max_candidates).enumerate() {
            let word_style = if i == 0 && Some(word.clone()) == state.current_suggestion {
                // Highlight the current best suggestion
                Style::default()
                    .fg(Color::Rgb(106, 170, 100)) // Green
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Rgb(255, 255, 255))
            };

            let entropy_style = Style::default().fg(Color::Rgb(255, 215, 0)); // Gold

            let prefix = if i == 0 && Some(word.clone()) == state.current_suggestion {
                "➤ " // Arrow for current suggestion
            } else {
                "  "
            };

            content.push(Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Rgb(106, 170, 100))),
                Span::styled(format!("{:<8}", word.to_uppercase()), word_style),
                Span::styled(format!(" {:.3}", entropy), entropy_style),
            ]));
        }

        // Show total count if there are more candidates
        if state.top_candidates.len() > max_candidates {
            content.push(Line::from(""));
            content.push(Line::from(vec![Span::styled(
                format!(
                    "... and {} more candidates",
                    state.top_candidates.len() - max_candidates
                ),
                Style::default()
                    .fg(Color::Rgb(176, 196, 222))
                    .add_modifier(Modifier::ITALIC),
            )]));
        }
    }

    let paragraph = Paragraph::new(content)
        .alignment(Alignment::Left)
        .block(panel_block);

    f.render_widget(paragraph, area);
}
