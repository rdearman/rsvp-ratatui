use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Alignment},
    widgets::{Block, Borders, Paragraph, Gauge},
    text::{Span, Line, Text},
    Terminal,
};
use ratatui::style::{Color, Style};

use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{self, LeaveAlternateScreen},
};
use std::io::stdout;
use std::time::{Duration, Instant};




pub fn run_ui(mut speed: u64, mut chunk_size: usize, total_words: usize, words: Vec<String>) {
    let mut current_word_index = 0;
    let mut paused = false; // Track if the display is paused
    let mut word_delay = Duration::from_millis(60000 / speed); // Delay between words
    let mut last_update = Instant::now();
    let mut last_event_time = Instant::now(); // Track last event to debounce input

    terminal::enable_raw_mode().unwrap();
    let mut stdout = stdout();
    stdout.execute(terminal::EnterAlternateScreen).unwrap();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut words_read = 0;
    let mut reading_time = 0.0;

    loop {
        // Handle word progression if not paused and delay has elapsed
        if !paused && last_update.elapsed() >= word_delay {
            last_update = Instant::now(); // Reset the timer
            if current_word_index < words.len() {
                current_word_index += chunk_size;
                words_read += chunk_size;
                reading_time += word_delay.as_secs_f64();
            } else {
                break; // End of words
            }
        }

        terminal.draw(|f| {
            let word_display = if current_word_index < words.len() {
                words[current_word_index..current_word_index + chunk_size.min(words.len() - current_word_index)]
                    .join(" ")
            } else {
                "End of text".to_string()
            };

            let size = f.area();

            // Split the screen vertically
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(5),  // Quick Keys
                    Constraint::Percentage(37), // Top Spacer
                    Constraint::Percentage(5),  // Text Block
                    Constraint::Percentage(38), // Bottom Spacer
                    Constraint::Percentage(15), // Stats
                ])
                .split(size);

            // Quick Keys Block
            let quick_keys_text = "[Q]uit | [Space] pause/resume | [L]oad File | [P]references | [↑] +10 | [↓] -10 | [PgUp] +100 | [PgDn] -100 | [1-9] chunk size | [←] skip back 5 | [→] skip forward 5";
            let quick_keys = Paragraph::new(quick_keys_text)
                .block(Block::default().borders(Borders::ALL).title("Quick Keys"));
            f.render_widget(quick_keys, chunks[0]);

            // Top Spacer Block
            let top_spacer = Block::default().style(Style::default().bg(Color::Black));
            f.render_widget(top_spacer, chunks[1]);

            // Text Block
            let text_content = Paragraph::new(word_display)
                .block(Block::default().borders(Borders::ALL).title("Text"))
                .alignment(Alignment::Center);
            f.render_widget(text_content, chunks[2]);

            // Bottom Spacer Block
            let bottom_spacer = Block::default().style(Style::default().bg(Color::Black));
            f.render_widget(bottom_spacer, chunks[3]);

            // Stats and Progress Block: Split vertically instead of horizontally
            let stats_progress_chunks = Layout::default()
                .direction(Direction::Vertical) // Stack vertically
                .constraints([
                    Constraint::Percentage(70), // Stats (top half)
                    Constraint::Percentage(30), // Progress (bottom half)
                ])
                .split(chunks[4]);

            // Split the Stats Block into two horizontal sections
            let stats_chunks = Layout::default()
                .direction(Direction::Horizontal) // Side-by-side layout
                .constraints([
                    Constraint::Percentage(50), // Left Stats (Part 1)
                    Constraint::Percentage(50), // Right Stats (Part 2)
                ])
                .split(stats_progress_chunks[0]); // Use the Stats block chunk

            // Left Stats Block (Part 1)
            let left_stats_text = Text::from(vec![
                Line::from(Span::raw(format!("Current Time Reading: {:.2} seconds", reading_time))),
                Line::from(Span::raw(format!("Words Read This Session: {}", words_read))),
                Line::from(Span::raw(format!("Total Words Read: {} of {}", words_read, total_words))),
            ]);
            let left_stats = Paragraph::new(left_stats_text)
                .block(Block::default().borders(Borders::ALL).title("Reading Statistics"));
            f.render_widget(left_stats, stats_chunks[0]);

            // Right Stats Block (Part 2)
            let right_stats_text = Text::from(vec![
                Line::from(Span::raw(format!("Speed: {} WPM", speed))),
                Line::from(Span::raw(format!("Chunk Size: {}", chunk_size))),
            ]);
            let right_stats = Paragraph::new(right_stats_text)
                .block(Block::default().borders(Borders::ALL).title("Speed Statistics"));
            f.render_widget(right_stats, stats_chunks[1]);

            // Progress Block
            let progress_percentage = words_read as f64 / total_words as f64 * 100.0;
            let progress = Gauge::default()
                .block(Block::default().borders(Borders::ALL).title("Progress"))
                .gauge_style(ratatui::style::Style::default())
                .ratio(progress_percentage / 100.0);
            f.render_widget(progress, stats_progress_chunks[1]);
        }).unwrap();

        // Event handling
        if event::poll(Duration::from_millis(50)).unwrap() {
            if let Event::Key(KeyEvent { code, .. }) = event::read().unwrap() {
                if last_event_time.elapsed() >= Duration::from_millis(100) {
                    match code {
                        KeyCode::Char(' ') => {
                            paused = !paused; // Toggle pause
                            last_update = Instant::now(); // Reset timer to avoid skipping
                        }
                        KeyCode::Up => speed += 10,
                        KeyCode::Down => speed = speed.saturating_sub(10),
                        KeyCode::PageUp => speed += 100,
                        KeyCode::PageDown => speed = speed.saturating_sub(100),
                        KeyCode::Char('q') => break, // Quit the program
                        KeyCode::Right => current_word_index = (current_word_index + chunk_size).min(words.len()),
                        KeyCode::Left => current_word_index = current_word_index.saturating_sub(chunk_size),
                        KeyCode::Char(c) if c.is_digit(10) => {
                            chunk_size = c.to_digit(10).unwrap() as usize;
                        }
                        _ => {}
                    }
                    last_event_time = Instant::now(); // Reset event time for debouncing
                    word_delay = Duration::from_millis(60000 / speed); // Recalculate delay
                }
            }
        }
    }

    terminal::disable_raw_mode().unwrap();
    terminal.backend_mut().execute(LeaveAlternateScreen).unwrap();
}
