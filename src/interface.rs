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
use crate::utilities::save_settings;
use ratatui::{Frame, backend::Backend};

fn draw_main_ui(
    f: &mut Frame,
    current_word_index: usize,
    chunk_size: usize,
    words: &[String],
    total_words: usize,
    speed: u64,
    words_read: usize,
    reading_time: f64,
) {
    const BGRND: Color = Color::Rgb(10, 34, 171); // Background color
    const TXT: Color = Color::Rgb(63, 252, 123); // Text color
    const SCRTEXT: Color = Color::Rgb(230, 230, 250); // Screen text color

    // Get terminal area
    let size = f.area();

    // Render a background block that spans the entire terminal area
    f.render_widget(Block::default().style(Style::default().bg(BGRND)), size);

    // Define layout chunks
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
    let quick_keys_text = "[Q]uit | [Space] pause/resume | [L]oad File | [P]references | [B]ookmark | [↑] +10 | [↓] -10 | [PgUp] +100 | [PgDn] -100 | [1-9] chunk size | [←] skip back 5 | [→] skip forward 5";
    let quick_keys = Paragraph::new(quick_keys_text)
        .block(Block::default().borders(Borders::ALL).title("Menu Keys"))
        .style(Style::default().fg(SCRTEXT).bg(BGRND));
    f.render_widget(quick_keys, chunks[0]);

    // Top Spacer Block
    let top_spacer = Block::default().style(Style::default().bg(BGRND));
    f.render_widget(top_spacer, chunks[1]);

    // Text Block
    let word_display = if current_word_index < words.len() {
        words[current_word_index..current_word_index + chunk_size.min(words.len() - current_word_index)].join(" ")
    } else {
        "End of text".to_string()
    };
    let styled_text = Text::from(Span::styled(word_display, Style::default().fg(TXT)));
    let text_content = Paragraph::new(styled_text)
        .block(Block::default().borders(Borders::ALL).title("Text"))
        .alignment(Alignment::Center)
        .style(Style::default().bg(BGRND).fg(TXT));
    f.render_widget(text_content, chunks[2]);

    // Bottom Spacer Block
    let bottom_spacer = Block::default().style(Style::default().bg(BGRND));
    f.render_widget(bottom_spacer, chunks[3]);

    // Stats and Progress Block
    let stats_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[4]);

    // Left Stats
    let left_stats_text = Text::from(vec![
        Line::from(Span::raw(format!("Current Time Reading: {:.2} seconds", reading_time))),
        Line::from(Span::raw(format!("Words Read This Session: {}", words_read))),
        Line::from(Span::raw(format!("Total Words Read: {} of {}", words_read, total_words))),
    ]);
    let left_stats = Paragraph::new(left_stats_text)
        .block(Block::default().borders(Borders::ALL).title("Reading Statistics"))
        .style(Style::default().fg(SCRTEXT).bg(BGRND));
    f.render_widget(left_stats, stats_chunks[0]);

    // Right Stats
    let right_stats_text = Text::from(vec![
        Line::from(Span::raw(format!("Speed: {} WPM", speed))),
        Line::from(Span::raw(format!("Chunk Size: {}", chunk_size))),
    ]);
    let right_stats = Paragraph::new(right_stats_text)
        .block(Block::default().borders(Borders::ALL).title("Speed Statistics"))
        .style(Style::default().fg(SCRTEXT).bg(BGRND));
    f.render_widget(right_stats, stats_chunks[1]);

    // Progress Block
    let progress_percentage = words_read as f64 / total_words as f64 * 100.0;
    let progress = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Progress"))
        .gauge_style(Style::default().fg(Color::Green).bg(BGRND))
        .ratio(progress_percentage / 100.0);
    f.render_widget(progress, chunks[4]);
}





pub fn run_ui(mut speed: u64, mut chunk_size: usize, mut total_words: usize, mut words: Vec<String>) {
    let mut current_word_index = 0;
    let mut paused = false;
    let mut word_delay = Duration::from_millis(60000 / speed);
    let mut last_update = Instant::now();

    terminal::enable_raw_mode().unwrap();
    let mut stdout = stdout();
    stdout.execute(terminal::EnterAlternateScreen).unwrap();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut words_read = 0;
    let mut reading_time = 0.0;

    // Initial draw of the main screen
    terminal.draw(|f| draw_main_ui(f, current_word_index, chunk_size, &words, total_words, speed, words_read, reading_time)).unwrap();

    loop {
        // Handle key events
        if event::poll(Duration::from_millis(10)).unwrap() {
            if let Event::Key(KeyEvent { code, .. }) = event::read().unwrap() {
                match code {
                    KeyCode::Char(' ') => paused = !paused, // Toggle pause
                    KeyCode::Char('p') => {
                        // Preferences screen
                        show_preferences_ui(&mut speed, &mut chunk_size);
						terminal.clear().unwrap();
                        terminal.draw(|f| draw_main_ui(f, current_word_index, chunk_size, &words, total_words, speed, words_read, reading_time)).unwrap();

                    }
                    KeyCode::Char('l') => {
                        // Load file screen
                        if let Some(new_words) = show_load_file_ui(&mut String::new()) {
                            words = new_words;
                            total_words = words.len();
                            current_word_index = 0;
                        }
						terminal.clear().unwrap();
                        terminal.draw(|f| draw_main_ui(f, current_word_index, chunk_size, &words, total_words, speed, words_read, reading_time)).unwrap();

                    }
                    KeyCode::Char('q') => break, // Quit
                    KeyCode::Up => speed += 10, // Increase speed
                    KeyCode::Down => speed = speed.saturating_sub(10), // Decrease speed
                    KeyCode::PageUp => speed += 100, // Large speed increase
                    KeyCode::PageDown => speed = speed.saturating_sub(100), // Large speed decrease
                    KeyCode::Right => {
                        // Move forward by chunk size
                        current_word_index = (current_word_index + chunk_size).min(words.len());
                    }
                    KeyCode::Left => {
                        // Move backward by chunk size
                        current_word_index = current_word_index.saturating_sub(chunk_size);
                    }
                    KeyCode::Char(c) if c.is_digit(10) => {
                        // Set chunk size based on numeric input
                        chunk_size = c.to_digit(10).unwrap() as usize;
                    }
                    _ => {}
                }

                // Update word delay if speed changes
                word_delay = Duration::from_millis(60000 / speed);
            }
        }

        // Handle automatic word progression when not paused
        if !paused && last_update.elapsed() >= word_delay {
            last_update = Instant::now();
            if current_word_index < words.len() {
                current_word_index += chunk_size;
                words_read += chunk_size;
                reading_time += word_delay.as_secs_f64();
            } else {
                break; // End of words
            }

            // Redraw the main UI with updated state
            terminal.draw(|f| draw_main_ui(f, current_word_index, chunk_size, &words, total_words, speed, words_read, reading_time)).unwrap();
        }
    }

    // Restore terminal state
    terminal::disable_raw_mode().unwrap();
    terminal.backend_mut().execute(LeaveAlternateScreen).unwrap();
}



pub fn prompt_user(message: &str) -> String {
    use std::io::{stdin, stdout, Write};
    let mut input = String::new();
    print!("{} ", message);
    stdout().flush().unwrap();
    stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}



pub fn show_preferences_ui(speed: &mut u64, chunk_size: &mut usize) {
    let mut selected_option = 0; // 0 = Speed, 1 = Chunk Size, 2 = Save, 3 = Cancel
    let options = ["Set Speed", "Set Chunk Size", "Save Preferences", "Cancel"];
    let mut terminal = {
        terminal::enable_raw_mode().unwrap();
        let backend = CrosstermBackend::new(stdout());
        Terminal::new(backend).unwrap()
    };

    // Clear the screen
    terminal.clear().unwrap();

    loop {
        terminal.draw(|f| {
            let size = f.size();

            // Layout for the menu
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    options.iter().map(|_| Constraint::Length(3)).chain([Constraint::Min(1)]),
                )
                .split(size);

            for (i, option) in options.iter().enumerate() {
                let style = if i == selected_option {
                    Style::default().fg(Color::Yellow).bg(Color::Blue)
                } else {
                    Style::default()
                };

                let paragraph = Paragraph::new(*option)
                    .block(Block::default().borders(Borders::ALL).style(style));
                f.render_widget(paragraph, chunks[i]);
            }
        }).unwrap();

        if let Ok(event::Event::Key(KeyEvent { code, .. })) = event::read() {
            match code {
                KeyCode::Up => {
                    if selected_option > 0 {
                        selected_option -= 1;
                    }
                }
                KeyCode::Down => {
                    if selected_option < options.len() - 1 {
                        selected_option += 1;
                    }
                }
                KeyCode::Enter => {
                    match selected_option {
                        0 => {
                            // Set Speed
                            *speed = prompt_user("Enter the new speed: ").parse().unwrap_or(*speed);
                        }
                        1 => {
                            // Set Chunk Size
                            *chunk_size = prompt_user("Enter the new chunk size: ").parse().unwrap_or(*chunk_size);
                        }
                        2 => {
                            // Save Preferences
                            save_settings(*speed, *chunk_size);
                            break;
                        }
                        3 => break, // Cancel
                        _ => {}
                    }
                }
                KeyCode::Esc => break, // Exit preferences on ESC
                _ => {}
            }
        }
    }

    terminal::disable_raw_mode().unwrap();
}


pub fn show_load_file_ui(file_path: &mut String) -> Option<Vec<String>> {
    let mut terminal = {
        terminal::enable_raw_mode().unwrap();
        let backend = CrosstermBackend::new(stdout());
        Terminal::new(backend).unwrap()
    };

    // Clear the screen
    terminal.clear().unwrap();

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(20),
                    Constraint::Percentage(60),
                    Constraint::Percentage(20),
                ])
                .split(size);

            let input_block = Paragraph::new(file_path.clone())
                .block(Block::default().borders(Borders::ALL).title("Load File"));
            f.render_widget(input_block, chunks[1]);
        }).unwrap();

        if let Ok(event::Event::Key(KeyEvent { code, .. })) = event::read() {
            match code {
                KeyCode::Enter => {
                    if let Ok(content) = std::fs::read_to_string(file_path.clone()) {
                        terminal::disable_raw_mode().unwrap();
                        return Some(content.split_whitespace().map(String::from).collect());
                    } else {
                        file_path.clear();
                    }
                }
                KeyCode::Backspace => {
                    file_path.pop();
                }
                KeyCode::Char(c) => {
                    file_path.push(c);
                }
                KeyCode::Esc => {
                    terminal::disable_raw_mode().unwrap();
                    return None;
                }
                _ => {}
            }
        }
    }
}
