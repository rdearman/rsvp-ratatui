use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Alignment},
    widgets::{Block, Borders, Paragraph, Gauge},
    text::{Span, Line, Text},
    Terminal,
};
use ratatui::style::{Color, Style};
use std::io::{self, Write};
use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{self, LeaveAlternateScreen},
};
use std::io::stdout;
use std::time::{Duration, Instant};
use crate::utilities::save_settings;
use ratatui::{Frame, backend::Backend};
use std::fs::OpenOptions;



/* ======= Draw UI =========== */
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
	// Split chunks[4] into two: one for stats, one for the progress bar
	let stats_progress_chunks = Layout::default()
		.direction(Direction::Vertical) // Stack stats and progress vertically
		.constraints([Constraint::Percentage(70), Constraint::Percentage(30)]) // 70% for stats, 30% for progress
		.split(chunks[4]);

	// Split the stats block into two horizontal parts
	let stats_chunks = Layout::default()
		.direction(Direction::Horizontal)
		.constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
		.split(stats_progress_chunks[0]);

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
		Line::from(Span::raw(format!("Bookmarked: {}", bookmarked))),
	]);
	let right_stats = Paragraph::new(right_stats_text)
		.block(Block::default().borders(Borders::ALL).title("Settings"))
		.style(Style::default().fg(SCRTEXT).bg(BGRND));
	f.render_widget(right_stats, stats_chunks[1]);

	// Progress Block
	let progress_percentage = words_read as f64 / total_words as f64 * 100.0;
	let progress = Gauge::default()
		.block(Block::default().borders(Borders::ALL).title("Progress"))
		.gauge_style(Style::default().fg(Color::Green).bg(BGRND))
		.ratio(progress_percentage / 100.0);
	f.render_widget(progress, stats_progress_chunks[1]);



}

/* ======= Run UI =========== */

pub fn run_ui(mut speed: u64, mut chunk_size: usize, mut total_words: usize, mut words: Vec<String>) {
    let mut current_word_index = 0;
    let mut paused = false;
	let mut bookmark = 0;
	let mut bookmarked = false;
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
    terminal.draw(|f| draw_main_ui(f, current_word_index, chunk_size, &words, total_words, speed, words_read, reading_time, bookmarked)).unwrap();

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
                        terminal.draw(|f| draw_main_ui(f, current_word_index, chunk_size, &words, total_words, speed, words_read, reading_time,bookmarked)).unwrap();

                    }
                    KeyCode::Char('l') => {
                        // Load file screen
                        if let Some(new_words) = show_load_file_ui(&mut String::new()) {
                            words = new_words;
                            total_words = words.len();
                            current_word_index = 0;
                        }
						terminal.clear().unwrap();
                        terminal.draw(|f| draw_main_ui(f, current_word_index, chunk_size, &words, total_words, speed, words_read, reading_time, bookmarked)).unwrap();

                    }
					KeyCode::Char('b') => {
						bookmarked = !bookmarked; // Toggle bookmark state
						if bookmarked {
							// Set the bookmark
							bookmark = current_word_index.min(words.len());
						} else {
							// Restore the bookmark
							current_word_index = bookmark.min(words.len());
							// Redraw the screen only after restoring the bookmark
							terminal.clear().unwrap();
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
								)
							})
							.unwrap();
						}
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
            terminal.draw(|f| draw_main_ui(f, current_word_index, chunk_size, &words, total_words, speed, words_read, reading_time, bookmarked)).unwrap();
        }
    }

    // Restore terminal state
    terminal::disable_raw_mode().unwrap();
    terminal.backend_mut().execute(LeaveAlternateScreen).unwrap();
}

/* ======= Prompt User =========== */

pub fn prompt_user(prompt: &str) -> String {
    // Temporarily disable raw mode to allow standard input
    terminal::disable_raw_mode().unwrap();
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    terminal::enable_raw_mode().unwrap();

    input.trim().to_string()
}


/* ======= Load FIle UI =========== */


pub fn show_load_file_ui(file_path: &mut String) -> Option<Vec<String>> {
    let mut terminal = {
        terminal::enable_raw_mode().unwrap();
        let backend = CrosstermBackend::new(stdout());
        Terminal::new(backend).unwrap()
    };

    let mut consume_next_event = false; // Debounce flag to prevent double processing

    // Clear the screen
    terminal.clear().unwrap();

    loop {
        // Redraw the UI
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

        // Handle user input
        if let Ok(event::Event::Key(KeyEvent { code, .. })) = event::read() {
            // Skip processing if the debounce flag is set
            if consume_next_event {
                consume_next_event = false; // Reset the flag
                continue; // Skip this event
            }

            match code {
                KeyCode::Enter => {
                    // Try to load the file
                    if let Ok(content) = std::fs::read_to_string(file_path.clone()) {
                        terminal::disable_raw_mode().unwrap();
                        return Some(content.split_whitespace().map(String::from).collect());
                    } else {
                        // Clear the file path if loading fails
                        file_path.clear();
                    }
                }
                KeyCode::Backspace => {
                    file_path.pop(); // Remove the last character
                    consume_next_event = true; // Prevent double processing
                }
                KeyCode::Char(c) => {
                    file_path.push(c); // Add the character to the file path
                    consume_next_event = true; // Prevent double processing
                }
                KeyCode::Esc => {
                    // Exit without loading
                    terminal::disable_raw_mode().unwrap();
                    return None;
                }
                _ => {}
            }
        }
    }
}





/* ======= Preferences UI =========== */




pub fn show_preferences_ui(speed: &mut u64, chunk_size: &mut usize) {
    let mut selected_option = 0; // 0 = Speed, 1 = Chunk Size, 2 = Save, 3 = Cancel
    let options = ["Set Speed", "Set Chunk Size", "Save Preferences", "Cancel"];
    let mut user_input_mode = false; // Tracks if the user is entering a value
    let mut input_value = String::new(); // Tracks the user's input for the entry block
    let mut consume_next_event = false; // Flag to consume the next event (debounce)

    // Open the log file (append mode)
    let mut log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("err.log")
        .unwrap();

    let mut terminal = {
        terminal::enable_raw_mode().unwrap();
        let backend = CrosstermBackend::new(stdout());
        Terminal::new(backend).unwrap()
    };

    terminal.clear().unwrap();

    loop {
        // Log the current state to the file
        writeln!(
            log_file,
            "DEBUG: selected_option = {}, user_input_mode = {}, input_value = '{}'",
            selected_option, user_input_mode, input_value
        )
        .unwrap();

        // Redraw the UI
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

            // Add user entry block at the bottom
            let user_entry_block = Paragraph::new(if user_input_mode {
                format!(
                    "Current Value: {}\nEnter New Value: {}",
                    if selected_option == 0 {
                        speed.to_string()
                    } else if selected_option == 1 {
                        chunk_size.to_string()
                    } else {
                        "".to_string()
                    },
                    input_value
                )
            } else {
                format!(
                    "Current Value: {}",
                    if selected_option == 0 {
                        speed.to_string()
                    } else if selected_option == 1 {
                        chunk_size.to_string()
                    } else {
                        "".to_string()
                    }
                )
            })
            .block(Block::default().borders(Borders::ALL).title("User Entry Block"));
            f.render_widget(user_entry_block, chunks[chunks.len() - 1]);
        }).unwrap();

        // Read user input
        if let Ok(event::Event::Key(KeyEvent { code, .. })) = event::read() {
            // Debounce handling
            if consume_next_event {
                consume_next_event = false; // Reset the flag
                continue; // Skip this event
            }

            if user_input_mode {
                // Log key event to the file
                writeln!(log_file, "DEBUG: Input Mode Key Event = {:?}", code).unwrap();

                // Handle user input in the entry block
                match code {
                    KeyCode::Char(c) => {
                        input_value.push(c); // Append characters
                        writeln!(log_file, "DEBUG: Char input = '{}', input_value = '{}'", c, input_value).unwrap();
                        consume_next_event = true; // Prevent double processing of Char input
                    }
                    KeyCode::Backspace => {
                        input_value.pop(); // Remove last character
                        consume_next_event = true; // Prevent double processing
                    }
                    KeyCode::Enter => {
                        // Save the input value and exit input mode
                        writeln!(log_file, "DEBUG: Saving input value: {}", input_value).unwrap();
                        if selected_option == 0 {
                            *speed = input_value.parse().unwrap_or(*speed);
                        } else if selected_option == 1 {
                            *chunk_size = input_value.parse().unwrap_or(*chunk_size);
                        }
                        save_settings(*speed, *chunk_size);
                        input_value.clear(); // Clear the input buffer
                        user_input_mode = false; // Exit input mode
                        terminal.clear().unwrap(); // Clear the screen
                        consume_next_event = true; // Prevent immediate reprocessing of Enter
                    }
                    KeyCode::Esc => {
                        // Cancel input and return to menu
                        writeln!(log_file, "DEBUG: Cancel input").unwrap();
                        input_value.clear(); // Clear the input buffer
                        user_input_mode = false; // Exit input mode
                        terminal.clear().unwrap(); // Clear the screen
                        consume_next_event = true; // Prevent immediate reprocessing of Esc
                    }
                    _ => {}
                }
                continue; // Skip menu handling while in input mode
            } else {
                // Log key event to the file
                writeln!(log_file, "DEBUG: Menu Mode Key Event = {:?}", code).unwrap();

                // Handle menu navigation
                match code {
                    KeyCode::Up => {
                        if selected_option > 0 {
                            selected_option -= 1;
                            consume_next_event = true; // Prevent double processing of Up
                        }
                    }
                    KeyCode::Down => {
                        if selected_option < options.len() - 1 {
                            selected_option += 1;
                            consume_next_event = true; // Prevent double processing of Down
                        }
                    }
                    KeyCode::Enter => {
                        match selected_option {
                            0 | 1 => {
                                // Enable input mode for Set Speed or Set Chunk Size
                                writeln!(log_file, "DEBUG: Entering input mode").unwrap();
                                user_input_mode = true;
                                input_value.clear(); // Clear the input buffer
                                terminal.clear().unwrap(); // Clear the screen for input
                                consume_next_event = true; // Prevent immediate reprocessing of Enter
                                continue; // Consume Enter key and avoid processing further
                            }
                            2 => {
                                // Save Preferences
                                writeln!(log_file, "DEBUG: Save preferences").unwrap();
                                save_settings(*speed, *chunk_size);
                                break;
                            }
                            3 => break, // Cancel and exit
                            _ => {}
                        }
                    }
                    KeyCode::Esc => break, // Exit preferences on ESC
                    _ => {}
                }
            }
        }
    }

    terminal::disable_raw_mode().unwrap();
}
