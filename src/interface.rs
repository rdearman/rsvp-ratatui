use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Alignment},
    widgets::{Block, Borders, Paragraph, Gauge},
    text::{Span, Line, Text},
    Terminal,
};
use ratatui::style::{Color, Style};
use std::io::{ Write};
use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{self, LeaveAlternateScreen},
};
use std::io::stdout;
use std::time::{Duration, Instant};
use crate::utilities::save_settings;
use ratatui::{Frame}; // , backend::Backend};
use std::fs::OpenOptions;
use crate::utilities::file_selector_ui;


/*
fn draw_main_ui(
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
    preferences_mode: bool, // NEW: Toggle preferences UI
    bookmark_mode: bool, // NEW: Toggle bookmark UI
 */
fn draw_main_ui(
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
    bookmarks_list: &[(usize, String)], // NEW: Accept bookmarks list
    selected_bookmark: usize, // NEW: Accept selected bookmark index
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
    let quick_keys_text = "[Q]uit | [Space] pause/resume | [L]oad File | [P]references | [B]ookmark | [↑] +10 | [↓] -10 | [PgUp] +100 | [PgDn] -100 | [1-9] chunk size | [←] -1 chunk | [→] +1 chunk";
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

    if bookmark_mode {
        let mut bookmark_items = vec!["=> Create Bookmark".to_string()]; // Default selection

        // Ensure existing bookmarks are listed
        for (i, (index, preview)) in bookmarks_list.iter().enumerate() {
            let selected = if i + 1 == selected_bookmark { "=>" } else { "  " };
            bookmark_items.push(format!("{} Word #{} ({})", selected, index, preview));
        }

        // Join all bookmark entries into a formatted string
        let bookmark_text = bookmark_items.join("\n");

        let bookmark_block = Paragraph::new(bookmark_text)
            .block(Block::default().borders(Borders::ALL).title("Bookmarks"))
            .style(Style::default().fg(Color::Yellow).bg(Color::Black));

        f.render_widget(bookmark_block, chunks[3]); // Ensure correct chunk
    } else {
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






pub fn run_ui(mut speed: u64, mut chunk_size: usize, mut total_words: usize, mut words: Vec<String>) {
    let mut current_word_index = 0;
    let mut paused = false;
    let mut preferences_mode = false; // NEW: Toggle preferences mode
    let mut bookmark_mode = false; // NEW: Toggle preferences mode
    let mut bookmark = 0;
    let mut bookmarked = false;
    let mut consume_next_event = false; // Debounce flag for key events
    let mut word_delay = Duration::from_millis(60000 / speed);
    let mut last_update = Instant::now();

    terminal::enable_raw_mode().unwrap();
    let mut stdout = stdout();
    stdout.execute(terminal::EnterAlternateScreen).unwrap();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut words_read = 0;
    let mut reading_time = 0.0;
    let mut bookmarks_list: Vec<(usize, String)> = vec![]; // Stores (word index, preview text)
    let mut selected_bookmark = 0; // Index of selected bookmark in the list

    
    // Initial draw of the main screen

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
            &bookmarks_list, // Pass bookmarks list
            selected_bookmark, // Pass selected bookmark index
        )
    }).unwrap();
    
    loop {
        // Handle key events
        if event::poll(Duration::from_millis(10)).unwrap() {
            if let Event::Key(KeyEvent { code, .. }) = event::read().unwrap() {
                // Skip event if debounce flag is set
                if consume_next_event {
                    consume_next_event = false; // Reset debounce flag
                    continue; // Skip this event
                }

                if bookmark_mode {
                    match code {
                        KeyCode::Up => {
                            if selected_bookmark > 0 {
                                selected_bookmark -= 1;
                            }
                        }
                        KeyCode::Down => {
                            if selected_bookmark < bookmarks_list.len() {
                                selected_bookmark += 1;
                            }
                        }
                        KeyCode::Enter => {
                            if selected_bookmark == 0 {
                                // Create a new bookmark
                                let preview = words
                                    .get(current_word_index..(current_word_index + 5).min(words.len()))
                                    .unwrap_or(&[])
                                    .join(" ");
                                bookmarks_list.push((current_word_index, preview));
                            } else {
                                // Jump to selected bookmark
                                current_word_index = bookmarks_list[selected_bookmark - 1].0;
                            }
                            bookmark_mode = false; // Close menu after selection
                        }
                        KeyCode::Esc => {
                            bookmark_mode = false; // Exit bookmarks menu
                        }
                        _ => {}
                    }
                } 

                if preferences_mode {
                    // Handling Preferences Mode Inputs
                    match code {
                        KeyCode::Up => speed += 10, // Increase speed
                        KeyCode::Down => speed = speed.saturating_sub(10), // Decrease speed
                        KeyCode::Right => chunk_size += 1, // Increase chunk size
                        KeyCode::Left => chunk_size = chunk_size.saturating_sub(1), // Decrease chunk size
                        KeyCode::Enter => {
                            save_settings(speed, chunk_size);
                            preferences_mode = false; // Exit preferences mode after saving
                        }
                        KeyCode::Esc => preferences_mode = false, // Exit preferences mode without saving
                        _ => {}
                    }
                } else {
                    // Normal UI Inputs
                    match code {
                        KeyCode::Char(' ') => paused = !paused, // Toggle pause
                        KeyCode::Char('p') => preferences_mode = true, // Open Preferences
                        KeyCode::Char('l') => {
                            // File Selector UI to select and load a file
                            if let Some(selected_file) = file_selector_ui() {
                                if let Ok(content) = std::fs::read_to_string(&selected_file) {
                                    let words = content.split_whitespace().map(String::from).collect::<Vec<_>>();
                                    let total_words = words.len();
                                    let current_word_index = 0;

                                    // Ensure the terminal is fully reset before reloading the UI
                                    terminal::disable_raw_mode().unwrap();
                                    terminal.backend_mut().execute(LeaveAlternateScreen).unwrap();
                                    drop(terminal); // Explicitly drop the old terminal instance

                                    // Relaunch UI with the new file
                                    run_ui(speed, chunk_size, total_words, words);

                                    // Exit after restarting to prevent duplicate instances
                                    return;
                                } else {
                                    // Handle error if the file cannot be read
                                    println!("Failed to read the selected file.");
                                }
                            }
                        }
                        KeyCode::Char('b') => {
                            if bookmark_mode {
                                bookmark_mode = false; // Close menu
                            } else {
                                bookmark_mode = true;  // Open menu
                                selected_bookmark = 0; // Reset selection to "Create Bookmark"
                            }
                        }
                      
                        KeyCode::Char('q') => {
                            // Restore terminal and exit gracefully
                            terminal::disable_raw_mode().unwrap();
                            terminal.backend_mut().execute(LeaveAlternateScreen).unwrap();
                            terminal.clear().unwrap(); // Ensure the terminal is completely cleared
                            break;
                        }
                        KeyCode::Up => speed += 10, // Increase speed
                        KeyCode::Down => speed = speed.saturating_sub(10), // Decrease speed
                        KeyCode::PageUp => speed += 100, // Large speed increase
                        KeyCode::PageDown => speed = speed.saturating_sub(100), // Large speed decrease
                        KeyCode::Right => {
                            current_word_index = (current_word_index + chunk_size).min(words.len());
                        }
                        KeyCode::Left => {
                            current_word_index = current_word_index.saturating_sub(chunk_size);
                        }
                        KeyCode::Char(c) if c.is_digit(10) => {
                            chunk_size = c.to_digit(10).unwrap() as usize;
                        }
                        _ => {}
                    }
                }

                // Redraw UI after key input
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
                        &bookmarks_list, // Pass bookmarks list
                        selected_bookmark, // Pass selected bookmark index
                    )
                }).unwrap();
                
            }
        }

        // Handle automatic word progression when not paused and not in preferences mode
        if !paused && !preferences_mode && last_update.elapsed() >= word_delay {
            last_update = Instant::now();
            if current_word_index < words.len() {
                current_word_index += chunk_size;
                words_read += chunk_size;
                reading_time += word_delay.as_secs_f64();
            } else {
                break; // End of words
            }

            // Redraw the main UI with updated state
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
                    &bookmarks_list, // Pass bookmarks list
                    selected_bookmark, // Pass selected bookmark index
                )
            }).unwrap();
        }
    }

    // Restore terminal state
    terminal::disable_raw_mode().unwrap();
    terminal.backend_mut().execute(LeaveAlternateScreen).unwrap();
}
