#![allow(unused_mut)]
use crate::utilities::get_adaptive_chunk_size;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Alignment},
    widgets::{Wrap, Block, Borders, Paragraph, Gauge},
    text::{Span, Line, Text},
    Terminal,
};
use ratatui::style::{Color, Style, Modifier};
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
use crate::utilities;
use std::collections::HashMap;
//use crate::json;
use serde_json::json;use serde_json::Value;
use tts::{Tts};



#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DisplayMode {
    WordChunk(usize),
    Sentence,
}

fn draw_main_ui(
    f: &mut Frame,
    current_word_index: usize,
    display_mode: DisplayMode,
    words: &[String],
    total_words: usize,
    speed: u64,
    words_read: usize,
    reading_time: f64,
    preferences_mode: bool,
    bookmark_mode: bool,
    pause_mode: bool,
    bookmarks_list: &[(usize, String)],
    selected_bookmark: usize,
    file_path: &str, // ✅ Add missing file_path
    smart_mode: bool,
    tts_enabled: bool,
) {
    // panic!("DEBUG: Passed current_word_index = {}", current_word_index);
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

    let quick_keys_text = "[Q]uit | [Space] pause/resume | [L]oad File | [W]eb | [P]references | [B]ookmark | [S]entence Mode | [↑] +10 | [↓] -10 | [PgUp] +100 | [PgDn] -100 | [1-9] chunk size ";
    let quick_keys = Paragraph::new(quick_keys_text)
        .block(Block::default().borders(Borders::ALL).title("Menu Keys"))
        .style(Style::default().fg(SCRTEXT).bg(BGRND));
    f.render_widget(quick_keys, chunks[0]);

  
    // **PREFERENCES UI (IN TOP SPACER)**
    if preferences_mode {
        let preferences_text = format!(
            "Preferences:\nSpeed: {} WPM  [↑] +10 | [↓] -10\nChunk Size: {} [←] -1 | [→] +1\n[Enter] Save | [Esc] Cancel",
            speed,
            if let DisplayMode::WordChunk(size) = display_mode { size.to_string() } else { "Sentence".to_string() }
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
        let chunk_size = if let DisplayMode::WordChunk(size) = display_mode { size } else { 1 };
        // Define window for context (20 words before and 20 words after the chunk)
        let before_start = current_word_index.saturating_sub(20);
        let chunk_end = (current_word_index + chunk_size).min(words.len());
        let after_end = (chunk_end + 20).min(words.len());

        // Get surrounding context
        let before_text = words[before_start..current_word_index].join(" ");
        let current_chunk = words[current_word_index..chunk_end].join(" ").to_uppercase(); // ✅ Show the full chunk
        let after_text = words[chunk_end..after_end].join(" ");
        let margin = "          "; // Define a left and right margin (spaces)

        // Define styling
        let chunk_style = Style::default().fg(TXT).bg(Color::Black).add_modifier(Modifier::BOLD); // ✅ Set chunk colour

        let context_text = Text::from(vec![
            Line::from(Span::styled("[Context]", Style::default().fg(TXT))), // Title
            Line::from(""), // Empty line
            Line::from(vec![
                Span::raw(format!("{} ", margin)),         // Left margin
                Span::raw(before_text),                    // Words before the chunk
                Span::styled(format!(" [{}] ", current_chunk), chunk_style), // Highlighted chunk
                Span::raw(after_text),                     // Words after the chunk
                Span::raw(margin),                         // Right margin
            ])
        ]);

        let context_block = Paragraph::new(context_text)
            .block(Block::default().borders(Borders::ALL).title("Paused"))
            .alignment(Alignment::Center) // ✅ Centers text
            .wrap(Wrap { trim: true }) // ✅ Enables word wrapping
            .style(Style::default().bg(Color::Black));

        f.render_widget(context_block, chunks[3]);
    } else if bookmark_mode {
        // Clear panel before displaying bookmarks
        let mut bookmark_items = vec!["=> Create Bookmark".to_string()];
        // for (i, (index, preview)) in bookmarks_list.iter().enumerate() {
        //     let selected = if i + 1 == selected_bookmark { "=>" } else { "  " };
        //     bookmark_items.push(format!("{} Word #{} ({})", selected, index, preview));
        // }
        let max_display = 10; // ✅ Limit display to 10 bookmarks at a time
        let start_index = selected_bookmark.saturating_sub(max_display / 2);
        let end_index = (start_index + max_display).min(bookmarks_list.len());

        for (i, (index, preview)) in bookmarks_list[start_index..end_index].iter().enumerate() {
            let selected = if i + start_index == selected_bookmark { "=>" } else { "  " };
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
        match display_mode {
            DisplayMode::WordChunk(chunk_size) => {
                let display_chunk_size = if smart_mode {
                    get_adaptive_chunk_size(words, current_word_index, chunk_size)
                } else {
                    chunk_size
                };
                words[current_word_index..current_word_index + display_chunk_size.min(words.len() - current_word_index)].join(" ")
            }
            DisplayMode::Sentence => {
                // For now, just display the current word.
                // We will implement sentence splitting later.
                words[current_word_index].clone()
            }
        }
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

    // Use the `file_path` that is passed to the function
    let file_path = file_path.to_string();

    let left_stats_text = format!(
        "\nFile: {}\nWords Read This Session: {}\nTotal Words: {} of {}\nReading Time: {:.2} seconds\nCurrent Position: {}",
        file_path.to_string(), words_read, words_read, total_words, reading_time, current_word_index
    );
    
    let left_stats = Paragraph::new(left_stats_text)
        .block(Block::default().borders(Borders::ALL).title("Reading Statistics"))
        .style(Style::default().fg(SCRTEXT).bg(BGRND));
    f.render_widget(left_stats, stats_split[0]);

    // **Right Stats**
    let right_stats_text = format!(
        "\nSpeed: {} WPM\nDisplay Mode: {}\nSmart Mode: {}\nTTS: {}",
        speed,
        match display_mode {
            DisplayMode::WordChunk(size) => format!("Chunk ({})", size),
            DisplayMode::Sentence => "Sentence".to_string(),
        },
        if smart_mode { "On" } else { "Off" },
        if tts_enabled { "On" } else { "Off" }
    );
    let right_stats = Paragraph::new(right_stats_text)
        .block(Block::default().borders(Borders::ALL).title("Settings"))
        .style(Style::default().fg(SCRTEXT).bg(BGRND));
    f.render_widget(right_stats, stats_split[1]);

    // **Progress Bar**
    let progress_ratio = if total_words > 0 {
        current_word_index as f64 / total_words as f64
    } else {
        0.0
    };
    // let progress_ratio = words_read as f64 / total_words as f64;
    // println!("DEBUG: words_read = {}, total_words = {}, progress_ratio = {}", words_read, total_words, progress_ratio);
    let progress_bar = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Progress"))
        .gauge_style(Style::default().fg(Color::Green).bg(BGRND))
        .ratio(progress_ratio);
    f.render_widget(progress_bar, stats_chunks[1]);
}


pub fn run_ui(


    mut speed: u64,


    mut display_mode: DisplayMode,


    mut total_words: usize,


    mut words: Vec<String>,


    book_data: &mut HashMap<String, Value>,


    global_speed: u64,


    global_chunk_size: usize,


    mut file_path: String,


    mut current_word_index: usize,


) -> usize {


    let mut chunk_size = if let DisplayMode::WordChunk(size) = display_mode { size } else { 1 };





    let mut preferences_mode = false;


    let mut bookmark_mode = false;


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


    let mut file_path = file_path.clone(); // Ensure we're using the correct file





    let mut selected_bookmark = 0;


    let mut pause_mode = false;


    let mut smart_mode = false;


    let mut tts_enabled = false;


    let mut tts = Tts::new(tts::Backends::SpeechDispatcher).unwrap();





    terminal


        .draw(|f| {


            draw_main_ui(


                f,


                current_word_index,


                display_mode,


                &words,


                total_words,


                speed,


                words_read,


                reading_time,


                preferences_mode,


                bookmark_mode,


                pause_mode,


                &bookmarks_list,


                selected_bookmark,


                &file_path,


                smart_mode,


                tts_enabled,


            )


        })


        .unwrap();





    loop {


        if event::poll(Duration::from_millis(10)).unwrap() {


            if let Event::Key(KeyEvent { code, .. }) = event::read().unwrap() {


                if consume_next_event {


                    consume_next_event = false;


                    continue;


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


                                let preview = words


                                    .get(current_word_index..(current_word_index + 5).min(words.len()))


                                    .unwrap_or(&[])


                                    .join(" ");


                                bookmarks_list.push((current_word_index, preview.clone()));





                                let file_path = file_path.clone();


                                let book_entry = book_data.entry(file_path.clone()).or_insert_with(|| {


                                    json!({


                                        "bookmarks": [],


                                        "speed": speed,


                                        "chunk_size": chunk_size,


                                        "last_position": 0


                                    })


                                });





                                if !book_entry["bookmarks"].is_array() {


                                    book_entry["bookmarks"] = json!([]);


                                }





                                if let Some(bookmarks) = book_entry["bookmarks"].as_array_mut() {


                                    bookmarks.push(json!({ "position": current_word_index, "preview": preview.clone() }));


                                }


                                if let DisplayMode::WordChunk(size) = display_mode {


                                    save_settings(speed, size, book_data.clone(), None, None);


                                } else {


                                    save_settings(speed, 1, book_data.clone(), None, None);


                                }


                            } else {


                                current_word_index = bookmarks_list[selected_bookmark - 1].0;


                            }


                            bookmark_mode = false;


                        }


                        KeyCode::Esc => bookmark_mode = false,


                        _ => {}


                    }


                } else if preferences_mode {


                    match code {


                        KeyCode::Up => speed += 10,


                        KeyCode::Down => speed = (speed.saturating_sub(10)).max(1),


                        KeyCode::Right => {


                            if let DisplayMode::WordChunk(size) = &mut display_mode {


                                *size += 1;


                            }


                        }


                        KeyCode::Left => {


                            if let DisplayMode::WordChunk(size) = &mut display_mode {


                                *size = size.saturating_sub(1);


                            }


                        }


                        KeyCode::Enter => {


                            word_delay = Duration::from_millis(60000 / speed);


                            if let DisplayMode::WordChunk(size) = display_mode {


                                save_settings(speed, size, book_data.clone(), None, None);


                            } else {


                                save_settings(speed, 1, book_data.clone(), None, None);


                            }


                            preferences_mode = false;


                        }


                        KeyCode::Esc => preferences_mode = false,


                        _ => {}


                    }


                } else {


                    match code {


                        KeyCode::Char(' ') => {


                            pause_mode = !pause_mode;


                            if pause_mode {


                                let _ = tts.stop();


                            }


                        }


                        KeyCode::Char('p') => preferences_mode = true,


                        KeyCode::Char('m') => smart_mode = !smart_mode,


                        KeyCode::Char('t') => tts_enabled = !tts_enabled,


                        KeyCode::Char('s') => {


                            display_mode = if display_mode == DisplayMode::Sentence {


                                DisplayMode::WordChunk(1)


                            } else {


                                DisplayMode::Sentence


                            };


                            words = match display_mode {


                                DisplayMode::WordChunk(_) => utilities::read_file_content(&file_path),


                                DisplayMode::Sentence => utilities::read_file_sentences(&file_path),


                            };


                            total_words = words.len();


                            current_word_index = 0;


                        }


                        KeyCode::Char('w') => {
                            if let Some(url) = utilities::get_url_ui() {
                                if let Ok(content) = utilities::get_content_from_url(&url) {
                                    words = content.split_whitespace().map(String::from).collect();
                                    total_words = words.len();
                                    current_word_index = 0;
                                    file_path = url;
                                }
                            }
                            terminal.clear().unwrap();
                            terminal::enable_raw_mode().unwrap();
                        }


                        KeyCode::Char('l') => {


                            match utilities::load_file_menu_ui(book_data) {


                                Some(selected_file) => {


                                    words = utilities::read_file_content(&selected_file);


                                    total_words = words.len();


                                    file_path = selected_file.clone();





                                    let book_entry =


                                        book_data.entry(selected_file.clone()).or_insert_with(|| {


                                            json!({


                                                "bookmarks": [],


                                                "speed": global_speed,


                                                "chunk_size": global_chunk_size,


                                                "last_position": 0


                                            })


                                        });





                                    speed = book_entry["speed"].as_u64().unwrap_or(global_speed);


                                    chunk_size = book_entry["chunk_size"].as_u64().unwrap_or(global_chunk_size as u64) as usize;


                                    display_mode = DisplayMode::WordChunk(chunk_size);


                                    current_word_index = book_entry["last_position"].as_u64().unwrap_or(0) as usize;





                                    bookmarks_list.clear();


                                    if let Some(bookmarks) = book_entry["bookmarks"].as_array() {


                                        bookmarks_list.extend(bookmarks.iter().filter_map(|bm| {


                                            Some((


                                                bm.get("position")?.as_u64()? as usize,


                                                bm.get("preview")?.as_str()?.to_string(),


                                            ))


                                        }));


                                    }


                                }


                                None => {}


                            }
                            terminal.clear().unwrap();
                            terminal::enable_raw_mode().unwrap();


                            terminal


                                .draw(|f| {


                                    draw_main_ui(


                                        f,


                                        current_word_index,


                                        display_mode,


                                        &words,


                                        total_words,


                                        speed,


                                        words_read,


                                        reading_time,


                                        preferences_mode,


                                        bookmark_mode,


                                        pause_mode,


                                        &bookmarks_list,


                                        selected_bookmark,


                                        &file_path,


                                        smart_mode,


                                        tts_enabled,


                                    )


                                })


                                .unwrap();


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


                            let _ = tts.stop();


                            terminal::disable_raw_mode().unwrap();


                            terminal.backend_mut().execute(LeaveAlternateScreen).unwrap();


                            terminal.clear().unwrap();


                            break;


                        }


                        KeyCode::Up => {


                            speed += 10;


                            word_delay = Duration::from_millis(60000 / speed);


                        }


                        KeyCode::Down => {


                            speed = (speed.saturating_sub(10)).max(1);


                            word_delay = Duration::from_millis(60000 / speed);


                        }


                        KeyCode::PageUp => {


                            speed += 100;


                            word_delay = Duration::from_millis(60000 / speed);


                        }


                        KeyCode::PageDown => {


                            speed = (speed.saturating_sub(100)).max(1);


                            word_delay = Duration::from_millis(60000 / speed);


                        }


                        KeyCode::Right => {


                            if let DisplayMode::WordChunk(size) = display_mode {


                                current_word_index = (current_word_index + size).min(words.len());


                            } else {


                                current_word_index = (current_word_index + 1).min(words.len());


                            }


                        }


                        KeyCode::Left => {


                            if let DisplayMode::WordChunk(size) = display_mode {


                                current_word_index = current_word_index.saturating_sub(size);


                            } else {


                                current_word_index = current_word_index.saturating_sub(1);


                            }


                        }


                        KeyCode::Char(c) if c.is_digit(10) => {


                            display_mode = DisplayMode::WordChunk(c.to_digit(10).unwrap() as usize);


                        }


                        _ => {}


                    }


                }





                terminal


                    .draw(|f| {


                        draw_main_ui(


                            f,


                            current_word_index,


                            display_mode,


                            &words,


                            total_words,


                            speed,


                            words_read,


                            reading_time,


                            preferences_mode,


                            bookmark_mode,


                            pause_mode,


                            &bookmarks_list,


                            selected_bookmark,


                            &file_path,


                            smart_mode,


                            tts_enabled,


                        )


                    })


                    .unwrap();


            }


        }





        if !pause_mode && !preferences_mode && last_update.elapsed() >= word_delay {


            last_update = Instant::now();


            if current_word_index < words.len() {


                let advance_chunk_size = if smart_mode {


                    match display_mode {


                        DisplayMode::WordChunk(chunk_size) => get_adaptive_chunk_size(&words, current_word_index, chunk_size),


                        DisplayMode::Sentence => 1, // TODO


                    }


                } else {


                    match display_mode {


                        DisplayMode::WordChunk(chunk_size) => chunk_size,


                        DisplayMode::Sentence => 1, // TODO


                    }


                };


                current_word_index += advance_chunk_size;


                words_read += advance_chunk_size;


                reading_time += word_delay.as_secs_f64();





                if tts_enabled {


                    let word_display = match display_mode {


                        DisplayMode::WordChunk(chunk_size) => {


                            let display_chunk_size = if smart_mode {


                                get_adaptive_chunk_size(&words, current_word_index, chunk_size)


                            } else {


                                chunk_size


                            };


                            words[current_word_index..current_word_index + display_chunk_size.min(words.len() - current_word_index)]


                                .join(" ")


                        }


                        DisplayMode::Sentence => words[current_word_index].clone(),


                    };


                    let _ = tts.speak(word_display, true);


                }


            } else {


                // Don't break here. Just stop advancing.


                // The user can still quit or load a new file.


            }





            terminal


                .draw(|f| {


                    draw_main_ui(


                        f,


                        current_word_index,


                        display_mode,


                        &words,


                        total_words,


                        speed,


                        words_read,


                        reading_time,


                        preferences_mode,


                        bookmark_mode,


                        pause_mode,


                        &bookmarks_list,


                        selected_bookmark,


                        &file_path,


                        smart_mode,


                        tts_enabled,


                    )


                })


                .unwrap();


        }


    }





    let _ = tts.stop();





    terminal::disable_raw_mode().unwrap();





    terminal.backend_mut().execute(LeaveAlternateScreen).unwrap();





    return current_word_index;


}




