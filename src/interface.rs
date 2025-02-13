#![allow(unused_mut)]
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Alignment},
    widgets::{Block, Borders, Paragraph, Gauge},
    text::{Span, Text},
    Terminal,
};
use ratatui::style::{Color, Style};
// use std::io::{ Write};
use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{self, LeaveAlternateScreen},
};
use std::io::stdout;
use std::time::{Duration, Instant};
use crate::utilities::save_settings;
use ratatui::{Frame}; // , backend::Backend};
//use std::fs::OpenOptions;
use crate::utilities::file_selector_ui;
use std::collections::HashMap;
use serde_json::Value;
use serde_json::json;

fn draw_main_ui (
    f: &mut Frame,
    current_word_index: usize,
    chunk_size: usize,
    words: &[String],
    total_words: usize,
    speed: u64,
    words_read: usize,
    reading_time: f64,
    bookmarked: bool,
    bookmark: usize,
    preferences_mode: bool,
    bookmark_mode: bool,
    pause_mode: bool, // NEW: Accept pause_mode
    bookmarks_list: &[(usize, String)],
    selected_bookmark: usize,
) {
    const BGRND: Color = Color::Rgb(10, 34, 171); // Background color
    const TXT: Color = Color::Rgb(63, 252, 123); // Text color
    const SCRTEXT: Color = Color::Rgb(230, 230, 250); // Screen text color

    let size = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(5),  // Quick Keys
            Constraint::Percentage(if preferences_mode { 31 } else { 31 }), // Top Spacer (shrinks when preferences are active)
            Constraint::Percentage(8),  // Text Block
            Constraint::Percentage(if bookmark_mode {28} else {28} ), // Bottom Spacer
            Constraint::Percentage(26), // Stats & Progress
        ])
        .split(size);

    f.render_widget(Block::default().style(Style::default().bg(BGRND)), size);

    // **Quick Keys Block**
    let quick_keys_text = "[Q]uit | [Space] pause/resume | [L]oad File | [P]references | [B]ookmark | [↑] +10 | [↓] -10 | [PgUp] +100 | [PgDn] -100 | [1-9] chunk size ";
    let quick_keys = Paragraph::new(quick_keys_text)
        .block(Block::default().borders(Borders::ALL).title("Menu Keys"))
        .style(Style::default().fg(SCRTEXT).bg(BGRND));
    f.render_widget(quick_keys, chunks[0]);

  
    // **PREFERENCES UI (IN TOP SPACER)**
    if preferences_mode {
        let preferences_text = format!(
            "Preferences:\nSpeed: {} WPM  [↑] +10 | [↓] -10\nChunk Size: {} [←] -1 | [→] +1\n[Enter] Save | [Esc] Cancel",
            speed, chunk_size
        );

        let preferences_block = Paragraph::new(preferences_text)
            .block(Block::default().borders(Borders::ALL).title("Preferences"))
            .style(Style::default().fg(Color::Yellow).bg(Color::Black));
        
        f.render_widget(preferences_block, chunks[1]); // Use the Top Spacer
    } else {
        let top_spacer = Block::default().style(Style::default().bg(BGRND));
        f.render_widget(top_spacer, chunks[1]);
    }

    // **BOOKMARK/PAUSE UI (IN BOTTOM SPACER)**    
    if pause_mode {
        // Clear panel before displaying context
        let start = current_word_index.saturating_sub(10);
        let end = (current_word_index + 10).min(words.len());
        let context_text = format!(
            "[Context]\n... {} ...", words[start..end].join(" ")
        );

        let context_block = Paragraph::new(context_text)
            .block(Block::default().borders(Borders::ALL).title("Paused Context"))
            .style(Style::default().fg(Color::Yellow).bg(Color::Black));

        f.render_widget(context_block, chunks[3]);
    } else if bookmark_mode {
        // Clear panel before displaying bookmarks
        let mut bookmark_items = vec!["=> Create Bookmark".to_string()];
        for (i, (index, preview)) in bookmarks_list.iter().enumerate() {
            let selected = if i + 1 == selected_bookmark { "=>" } else { "  " };
            bookmark_items.push(format!("{} Word #{} ({})", selected, index, preview));
        }
        let bookmark_text = bookmark_items.join("\n");

        let bookmark_block = Paragraph::new(bookmark_text)
            .block(Block::default().borders(Borders::ALL).title("Bookmarks"))
            .style(Style::default().fg(Color::Yellow).bg(Color::Black));

        f.render_widget(bookmark_block, chunks[3]);
    } else {
        // Default blank panel
        let bottom_spacer = Block::default().style(Style::default().bg(BGRND));
        f.render_widget(bottom_spacer, chunks[3]);
    }

    
    
    // **Text Block**
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

    // **Bottom Spacer**
    let bottom_spacer = Block::default().style(Style::default().bg(BGRND));
    f.render_widget(bottom_spacer, chunks[3]);

    // **Stats & Progress Block**
    let stats_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)]) // 70% for stats, 30% for progress
        .split(chunks[4]);

    // **Stats Layout (Split into Two Panels)**
    let stats_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)]) // Left/Right Stats
        .split(stats_chunks[0]);

    // **Left Stats**
    let left_stats_text = format!(
        "Words Read This Session: {}\nTotal Words: {} of {}\nReading Time: {:.2} seconds",
        words_read, words_read, total_words, reading_time
    );
    let left_stats = Paragraph::new(left_stats_text)
        .block(Block::default().borders(Borders::ALL).title("Reading Statistics"))
        .style(Style::default().fg(SCRTEXT).bg(BGRND));
    f.render_widget(left_stats, stats_split[0]);

    // **Right Stats**
    let right_stats_text = format!(
        "Speed: {} WPM\nChunk Size: {}\nBookmarked: {}\nBookmark Word#: {}",
        speed, chunk_size, bookmarked, bookmark
    );
    let right_stats = Paragraph::new(right_stats_text)
        .block(Block::default().borders(Borders::ALL).title("Settings"))
        .style(Style::default().fg(SCRTEXT).bg(BGRND));
    f.render_widget(right_stats, stats_split[1]);

    // **Progress Bar**
    let progress_ratio = words_read as f64 / total_words as f64;
    let progress_bar = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Progress"))
        .gauge_style(Style::default().fg(Color::Green).bg(BGRND))
        .ratio(progress_ratio);
    f.render_widget(progress_bar, stats_chunks[1]);
}

pub fn run_ui(
    mut speed: u64,
    mut chunk_size: usize,
    mut total_words: usize,
    mut words: Vec<String>,
    book_data: &mut HashMap<String, Value>,
    file_path: &str,  // ✅ Added file path
) -> usize {
    let mut current_word_index = 0;
    let mut paused = false;
    let mut preferences_mode = false;
    let mut bookmark_mode = false;
    let mut bookmark = 0;
    let mut bookmarked = false;
    let mut consume_next_event = false;
    let mut word_delay = Duration::from_millis(60000 / speed);
    let mut last_update = Instant::now();

    let mut stdout = stdout();
    terminal::enable_raw_mode().unwrap();
    stdout.execute(terminal::EnterAlternateScreen).unwrap();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut words_read = 0;
    let mut reading_time = 0.0;
    let mut bookmarks_list: Vec<(usize, String)> = vec![];
    let mut selected_bookmark = 0;
    let mut pause_mode = false;

    terminal.draw(|f| {
        draw_main_ui(
            f,
            current_word_index,
            chunk_size,
            &words,
            total_words,
            speed,
            words_read,
            reading_time,
            bookmarked,
            bookmark,
            preferences_mode,
            bookmark_mode,
            pause_mode,
            &bookmarks_list,
            selected_bookmark,
        )
    }).unwrap();

    loop {
        if event::poll(Duration::from_millis(10)).unwrap() {
            if let Event::Key(KeyEvent { code, .. }) = event::read().unwrap() {
                if consume_next_event {
                    consume_next_event = false;
                    continue;
                }

                if bookmark_mode {
                    match code {
                        KeyCode::Enter => {
                            if selected_bookmark == 0 {
                                let preview = words
                                    .get(current_word_index..(current_word_index + 5).min(words.len()))
                                    .unwrap_or(&[])
                                    .join(" ");

                                // ✅ Ensure bookmarks are stored in `book_data`
                                if let Some(book) = book_data.get_mut(file_path) { // ✅ Use file_path instead of absolute_path_str                                    
                                    let mut bookmarks = book["bookmarks"]
                                        .as_array()
                                        .cloned()
                                        .unwrap_or_else(|| vec![]);

                                    bookmarks.push(json!({ "position": current_word_index, "name": preview }));
                                    book["bookmarks"] = json!(bookmarks);
                                }
                            } else {
                                current_word_index = bookmarks_list[selected_bookmark - 1].0;
                            }
                            bookmark_mode = false;
                        }
                        _ => {}
                        
                    }
                }

                if preferences_mode {
                    match code {
                        KeyCode::Up => speed += 10,
                        KeyCode::Down => speed = speed.saturating_sub(10),
                        KeyCode::Right => chunk_size += 1,
                        KeyCode::Left => chunk_size = chunk_size.saturating_sub(1),
                        KeyCode::Enter => {
                            save_settings(speed, chunk_size, book_data.clone(), None, None);
                            preferences_mode = false;
                        }
                        KeyCode::Esc => preferences_mode = false,
                        _ => {}
                    }
                } else {
                    match code {
                        KeyCode::Char(' ') => pause_mode = !pause_mode,
                        KeyCode::Char('p') => preferences_mode = true,
                        KeyCode::Char('l') => {
                            if let Some(selected_file) = file_selector_ui() {
                                if let Ok(content) = std::fs::read_to_string(&selected_file) {
                                    let words = content.split_whitespace().map(String::from).collect::<Vec<_>>();
                                    let total_words = words.len();
                                    let start_position = 0;

                                    // Fully reset UI before restarting
                                    terminal::disable_raw_mode().unwrap();
                                    terminal.backend_mut().execute(LeaveAlternateScreen).unwrap();
                                    drop(terminal);

                                    // Relaunch with the new file
                                    run_ui(speed, chunk_size, total_words, words, book_data, file_path);
                                    return current_word_index; // ✅ Correct return type
                                } else {
                                    println!("Failed to read the selected file.");
                                }
                            }
                        }
                        KeyCode::Char('b') => {
                            if bookmark_mode {
                                bookmark_mode = false;
                            } else {
                                bookmark_mode = true;
                                selected_bookmark = 0;
                            }
                        }
                        KeyCode::Char('q') => {
                            terminal::disable_raw_mode().unwrap();
                            terminal.backend_mut().execute(LeaveAlternateScreen).unwrap();
                            terminal.clear().unwrap();
                            break;
                        }
                        KeyCode::Up => speed += 10,
                        KeyCode::Down => speed = speed.saturating_sub(10),
                        KeyCode::PageUp => speed += 100,
                        KeyCode::PageDown => speed = speed.saturating_sub(100),
                        KeyCode::Right => current_word_index = (current_word_index + chunk_size).min(words.len()),
                        KeyCode::Left => current_word_index = current_word_index.saturating_sub(chunk_size),
                        KeyCode::Char(c) if c.is_digit(10) => {
                            chunk_size = c.to_digit(10).unwrap() as usize;
                        }
                        _ => {}
                    }
                }

                terminal.draw(|f| {
                    draw_main_ui(
                        f,
                        current_word_index,
                        chunk_size,
                        &words,
                        total_words,
                        speed,
                        words_read,
                        reading_time,
                        bookmarked,
                        bookmark,
                        preferences_mode,
                        bookmark_mode,
                        pause_mode,
                        &bookmarks_list,
                        selected_bookmark,
                    )
                }).unwrap();
            }
        }

        if !pause_mode && !preferences_mode && last_update.elapsed() >= word_delay {
            last_update = Instant::now();
            if current_word_index < words.len() {
                current_word_index += chunk_size;
                words_read += chunk_size;
                reading_time += word_delay.as_secs_f64();
            } else {
                break;
            }

            terminal.draw(|f| {
                draw_main_ui(
                    f,
                    current_word_index,
                    chunk_size,
                    &words,
                    total_words,
                    speed,
                    words_read,
                    reading_time,
                    bookmarked,
                    bookmark,
                    preferences_mode,
                    bookmark_mode,
                    pause_mode,
                    &bookmarks_list,
                    selected_bookmark,
                )
            }).unwrap();
        }
    }

    terminal::disable_raw_mode().unwrap();
    terminal.backend_mut().execute(LeaveAlternateScreen).unwrap();
    return current_word_index;
}

