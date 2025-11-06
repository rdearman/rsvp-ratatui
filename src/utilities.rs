#![allow(unused_mut)]
use unicode_segmentation::UnicodeSegmentation;
use std::fs;
use std::io::{Read, stdout};
use dirs_next::home_dir;
use crossterm::event::{self, KeyCode, KeyEvent};
use crossterm::terminal;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use ratatui::style::{Style, Color};
use ratatui::Terminal;
use epub::doc::EpubDoc;
use pdf_extract::extract_text;
use scraper::{Html, Selector};
use pulldown_cmark::{Parser, Options, Event};
use zip::read::ZipArchive;
use xml::reader::{EventReader, XmlEvent};
use std::collections::HashMap;
use serde_json::{json, Value};
//use std::io::{Write, Read};
use std::fs::{File, read_dir};
use std::io::Write ;

/// List of supported file types
const SUPPORTED_FILE_TYPES: &[&str] = &["pdf",  "docx", "odt", "txt", "html", "htm", "md"]; // Removed "epub" because it was crashing

pub fn browse_files_ui() -> Option<String> {
    let mut current_dir = std::env::current_dir().expect("Failed to get current directory");
    let mut file_entries = get_file_entries(&current_dir);
    let mut selected_index = 0;

    let backend = ratatui::backend::CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend).unwrap();

    struct RawModeGuard;
    impl Drop for RawModeGuard {
        fn drop(&mut self) {
            let _ = terminal::disable_raw_mode();
        }
    }
    let _guard = RawModeGuard;

    terminal::enable_raw_mode().unwrap();
    terminal.clear().unwrap();

    let max_display = 45; // Maximum number of visible entries
    let mut start_index = 0; // Track the starting index for scrolling

    loop {
        terminal.draw(|f| {
            let size = f.area();

            // Adjust the displayed range to keep the selected item visible
            if selected_index < start_index {
                start_index = selected_index;
            } else if selected_index >= start_index + max_display {
                start_index = selected_index + 1 - max_display;
            }

            let end_index = (start_index + max_display).min(file_entries.len());

            let items: Vec<ListItem> = file_entries[start_index..end_index]
                .iter()
                .enumerate()
                .map(|(i, entry)| {
                    let actual_index = i + start_index;
                    if actual_index == selected_index {
                        ListItem::new(format!("=> {}", entry))
                            .style(Style::default().fg(Color::Black).bg(Color::Yellow))
                    } else {
                        ListItem::new(entry.clone())
                    }
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title(format!("Select a File - {:?}", current_dir)));

            f.render_widget(list, size);
        }).unwrap();

        if let Ok(event::Event::Key(KeyEvent { code, .. })) = event::read() {
            match code {
                KeyCode::Up => {
                    if selected_index > 0 {
                        selected_index -= 1;
                    }
                }
                KeyCode::Down => {
                    if selected_index < file_entries.len() - 1 {
                        selected_index += 1;
                    }
                }
                KeyCode::Enter => {
                    let selected = &file_entries[selected_index];
                    let selected_path = current_dir.join(selected);

                    if selected == ".." {
                        if let Some(parent) = current_dir.parent() {
                            current_dir = parent.to_path_buf();
                            file_entries = get_file_entries(&current_dir);
                            selected_index = 0;
                        }
                    } else if selected_path.is_dir() {
                        current_dir = selected_path;
                        file_entries = get_file_entries(&current_dir);
                        selected_index = 0;
                    } else {
                        // ✅ Clear UI properly before returning file
                        terminal.clear().unwrap();
                        terminal::disable_raw_mode().unwrap();
//                        return Some(selected_path.to_string_lossy().into_owned());
                        if let Ok(absolute_path) = fs::canonicalize(&selected_path) {
                            return Some(absolute_path.to_string_lossy().into_owned());
                        } else {
                            return Some(selected_path.to_string_lossy().into_owned());
                        }


                    }
                }
                KeyCode::Esc => {
                    // ✅ Properly restore the UI when exiting
                    terminal::disable_raw_mode().unwrap();
                    return None;
                }
                _ => {}
            }
        }
    }
}



fn get_file_entries(dir: &std::path::Path) -> Vec<String> {
    let mut entries = Vec::new();

    // Always add ".." to move up a directory
    if dir.parent().is_some() {
        entries.push("..".to_string());
    }

    if let Ok(read_dir) = read_dir(dir) {
        for entry in read_dir.flatten() {
            let file_name = entry.file_name().to_string_lossy().into_owned();
            let path = entry.path();

            if path.is_dir() {
                // Always add directories
                entries.push(file_name);
            } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                // Only add files with supported extensions
                if SUPPORTED_FILE_TYPES.contains(&ext) {
                    entries.push(file_name);
                }
            }
        }
    }

    entries
}



/* supports: PDF, docx, txt, html, MD */
pub fn get_content(file_path: &str) -> String {
    if file_path.ends_with(".pdf") {
        // Extract text from PDF
        match extract_text(file_path) {
            Ok(text) => text,
            Err(e) => {
                eprintln!("Failed to extract text from PDF '{}': {}", file_path, e);
                String::new()
            },
        }
    } else if file_path.ends_with(".epub") {
        // Extract text from EPUB
        match EpubDoc::new(file_path) {
            Ok(mut doc) => {
                let mut text = String::new();
                while let Ok(page) = doc.get_current_str() {
                    text.push_str(&page);
                    let _ = doc.go_next();
                }
                text
            }
            Err(_) => panic!("Failed to extract text from EPUB"),
        }
    } else if file_path.ends_with(".docx") {
        // Extract text from DOCX
        let file = File::open(file_path).expect("Failed to open DOCX file");
        let mut archive = ZipArchive::new(file).expect("Failed to read DOCX ZIP structure");

        let mut text = String::new();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).expect("Failed to read DOCX entry");
            let file_name = file.name().to_string();

            if file_name == "word/document.xml" {
                let mut content = String::new();
                file.read_to_string(&mut content).expect("Failed to extract XML");

                // Strip XML tags (a basic approach)
                text = content
                    .replace("<w:t>", "")
                    .replace("</w:t>", " ")
                    .replace("<w:p>", "\n")
                    .replace("</w:p>", "\n");
                break;
            }
        }
        text
    } else if file_path.ends_with(".html") || file_path.ends_with(".htm") {
        // Extract text from HTML
        let content = fs::read_to_string(file_path).expect("Failed to read HTML file");
        let document = Html::parse_document(&content);
        let selector = Selector::parse("body").unwrap();
        document
            .select(&selector)
            .map(|e| e.text().collect::<Vec<_>>().join(" "))
            .collect::<Vec<String>>()
            .join(" ")
    } else if file_path.ends_with(".md") {
        // Extract text from Markdown
        let content = fs::read_to_string(file_path).expect("Failed to read Markdown file");
        let parser = Parser::new_ext(&content, Options::all());
        parser
            .filter_map(|event| match event {
                Event::Text(t) => Some(t.to_string()),
                _ => None,
            })
            .collect::<Vec<String>>()
            .join(" ")
    } else if file_path.ends_with(".odt") {
        // Extract text from Open Document Format
        let file = File::open(file_path).expect("Failed to open ODT file");
        let mut archive = ZipArchive::new(file).expect("Failed to read ODT ZIP structure");

        let mut text = String::new();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).expect("Failed to read ODT entry");
            let file_name = file.name().to_string();

            if file_name == "content.xml" {
                let mut content = String::new();
                file.read_to_string(&mut content).expect("Failed to extract XML");

                let xml_parser = EventReader::from_str(&content);
                for event in xml_parser {
                    if let Ok(XmlEvent::Characters(chars)) = event {
                        text.push_str(&chars);
                        text.push(' ');
                    }
                }
                break;
            }
        }
        text
    } else {
        // Default to plain text files
        fs::read_to_string(file_path)
            .expect("Failed to read input file")
    }
}

pub fn read_file_content(file_path: &str) -> Vec<String> {
    get_content(file_path)
        .split_whitespace()
        .map(String::from)
        .collect::<Vec<_>>()
}

pub fn read_file_sentences(file_path: &str) -> Vec<String> {
    get_content(file_path)
        .unicode_sentences()
        .map(String::from)
        .collect::<Vec<_>>()
}


pub fn load_file_menu_ui(book_data: &HashMap<String, Value>) -> Option<String> {
    let mut menu_options = vec!["Browse Files".to_string()];
    let mut recent_files: Vec<String> = book_data.keys().cloned().collect();
    recent_files.sort(); // Sort alphabetically for now

    if !recent_files.is_empty() {
        menu_options.push("Recent Files".to_string());
    }

    let backend = ratatui::backend::CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend).unwrap();

    struct RawModeGuard;
    impl Drop for RawModeGuard {
        fn drop(&mut self) {
            let _ = terminal::disable_raw_mode();
        }
    }
    let _guard = RawModeGuard;

    terminal::enable_raw_mode().unwrap();
    terminal.clear().unwrap();

    let mut selected_index = 0;
    let mut in_recent_files_menu = false;
    let mut selected_recent_file_index = 0;

    loop {
        terminal.draw(|f| {
            let size = f.area();
            let items: Vec<ListItem> = if in_recent_files_menu {
                recent_files
                    .iter()
                    .enumerate()
                    .map(|(i, entry)| {
                        if i == selected_recent_file_index {
                            ListItem::new(format!("=> {}", entry))
                                .style(Style::default().fg(Color::Black).bg(Color::Yellow))
                        } else {
                            ListItem::new(entry.clone())
                        }
                    })
                    .collect()
            } else {
                menu_options
                    .iter()
                    .enumerate()
                    .map(|(i, entry)| {
                        if i == selected_index {
                            ListItem::new(format!("=> {}", entry))
                                .style(Style::default().fg(Color::Black).bg(Color::Yellow))
                        } else {
                            ListItem::new(entry.clone())
                        }
                    })
                    .collect()
            };

            let title = if in_recent_files_menu { "Recent Files" } else { "Load File" };
            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title(title));

            f.render_widget(list, size);
        }).unwrap();

        if let Ok(event::Event::Key(KeyEvent { code, .. })) = event::read() {
            match code {
                KeyCode::Up => {
                    if in_recent_files_menu {
                        if selected_recent_file_index > 0 {
                            selected_recent_file_index -= 1;
                        }
                    } else {
                        if selected_index > 0 {
                            selected_index -= 1;
                        }
                    }
                }
                KeyCode::Down => {
                    if in_recent_files_menu {
                        if selected_recent_file_index < recent_files.len() - 1 {
                            selected_recent_file_index += 1;
                        }
                    } else {
                        if selected_index < menu_options.len() - 1 {
                            selected_index += 1;
                        }
                    }
                }
                KeyCode::Enter => {
                    if in_recent_files_menu {
                        terminal.clear().unwrap();
                        terminal::disable_raw_mode().unwrap();
                        return Some(recent_files[selected_recent_file_index].clone());
                    } else {
                        match menu_options[selected_index].as_str() {
                            "Browse Files" => {
                                terminal.clear().unwrap();
                                terminal::disable_raw_mode().unwrap();
                                return browse_files_ui();
                            }
                            "Recent Files" => {
                                in_recent_files_menu = true;
                                selected_recent_file_index = 0;
                            }
                            _ => {}
                        }
                    }
                }
                KeyCode::Esc => {
                    if in_recent_files_menu {
                        in_recent_files_menu = false;
                    } else {
                        terminal::disable_raw_mode().unwrap();
                        return None;
                    }
                }
                _ => {}
            }
        }
    }
}

/// Save settings to a JSON file

/// Load settings from the user's home directory
pub fn load_settings() -> (u64, usize, HashMap<String, Value>) {
    let mut speed = 300; // Default speed
    let mut chunk_size = 1; // Default chunk size
    let mut book_data: HashMap<String, Value> = HashMap::new();

    if let Some(home) = home_dir() {
        let settings_path = home.join(".rsvp_settings.json");
        if settings_path.exists() {
            let mut file = File::open(settings_path).expect("Failed to open settings file.");
            let mut content = String::new();
            if file.read_to_string(&mut content).is_ok() {
                if let Ok(json_data) = serde_json::from_str::<Value>(&content) {
                    if let Some(global) = json_data.get("global") {
                        if let Some(s) = global.get("speed").and_then(|v| v.as_u64()) {
                            speed = s;
                        }
                        if let Some(cs) = global.get("chunk_size").and_then(|v| v.as_u64()) {
                            chunk_size = cs as usize;
                        }
                    }
                    if let Some(books) = json_data.get("books").and_then(|v| v.as_object()) {
                        for (key, value) in books {
                            book_data.insert(key.clone(), value.clone());
                        }
                    }
                }
            }
        }
    }

    (speed, chunk_size, book_data)
}


pub fn save_settings(
    speed: u64,
    chunk_size: usize,
    mut book_data: HashMap<String, Value>,
    max_saved_books: Option<u64>,
    max_bookmarks_per_book: Option<u64>,
) {
    if let Some(home) = home_dir() {
        let settings_path = home.join(".rsvp_settings.json");

        // Load existing settings if they exist
        let mut global_settings = json!({
            "speed": speed,
            "chunk_size": chunk_size,
            "max_saved_books": 10, // Default
            "max_bookmarks_per_book": 10 // Default
        });

        if settings_path.exists() {
            if let Ok(mut file) = File::open(&settings_path) {
                let mut content = String::new();
                if file.read_to_string(&mut content).is_ok() {
                    if let Ok(json_data) = serde_json::from_str::<Value>(&content) {
                        if let Some(global) = json_data.get("global") {
                            global_settings["max_saved_books"] =
                                json!(max_saved_books.unwrap_or(global.get("max_saved_books").and_then(|v| v.as_u64()).unwrap_or(10)));

                            global_settings["max_bookmarks_per_book"] =
                                json!(max_bookmarks_per_book.unwrap_or(global.get("max_bookmarks_per_book").and_then(|v| v.as_u64()).unwrap_or(10)));
                        }

                        // ✅ Ensure bookmarks are retained when saving
                        if let Some(books) = json_data.get("books").and_then(|v| v.as_object()) {
                            for (key, value) in books {
                                if let Some(book) = book_data.get_mut(key) {
                                    if let Some(bookmarks) = value.get("bookmarks") {
                                        if book["bookmarks"].is_array() {
                                            // ✅ Ensure new bookmarks are merged with existing ones
                                            if let Some(existing_bookmarks) = book["bookmarks"].as_array_mut() {
                                                for bm in bookmarks.as_array().unwrap() {
                                                    if !existing_bookmarks.contains(bm) {
                                                        existing_bookmarks.push(bm.clone());
                                                    }
                                                }
                                            }
                                        } else {
                                            book["bookmarks"] = bookmarks.clone();
                                        }
                                    }
                                } else {
                                    // ✅ If book entry does not exist, create it with bookmarks
                                    book_data.insert(key.clone(), json!({ "bookmarks": value.get("bookmarks").cloned().unwrap_or(json!([])) }));
                                }
                            }
                        }
                    }
                }
            }
        }

        let settings = json!({
            "global": global_settings,
            "books": book_data
        });

        // ✅ Write to the JSON file
        if let Ok(mut file) = File::create(settings_path) {
            if let Ok(json_str) = serde_json::to_string_pretty(&settings) {
                let _ = file.write_all(json_str.as_bytes());
            }
        }
    }
}

/// Adjusts the chunk size based on the length of the words.
pub fn get_adaptive_chunk_size(
    words: &[String],
    current_index: usize,
    base_chunk_size: usize,
) -> usize {
    if current_index >= words.len() {
        return base_chunk_size;
    }

    let mut total_chars = 0;
    let mut num_words = 0;

    // A simple heuristic: try to fit a certain number of characters in a chunk.
    // Let's aim for an average of 5 characters per word in the base chunk size.
    let target_chars = base_chunk_size * 5;

    for i in 0..base_chunk_size * 2 { // Check up to twice the base chunk size
        if current_index + i < words.len() {
            total_chars += words[current_index + i].len();
            num_words += 1;
            if total_chars > target_chars {
                break;
            }
        } else {
            break;
        }
    }

    num_words.max(1) // Always return at least 1.
}

pub fn get_content_from_url(url: &str) -> Result<String, reqwest::Error> {
    let body = reqwest::blocking::get(url)?.text()?;
    let document = Html::parse_document(&body);
    let selector = Selector::parse("body").unwrap();
    let text = document
        .select(&selector)
        .map(|e| e.text().collect::<Vec<_>>().join(" "))
        .collect::<Vec<String>>()
        .join(" ");
    Ok(text)
}

pub fn get_url_ui() -> Option<String> {
    let backend = ratatui::backend::CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend).unwrap();

    struct RawModeGuard;
    impl Drop for RawModeGuard {
        fn drop(&mut self) {
            let _ = terminal::disable_raw_mode();
        }
    }
    let _guard = RawModeGuard;

    terminal::enable_raw_mode().unwrap();
    terminal.clear().unwrap();

    let mut url = String::new();

    loop {
        terminal.draw(|f| {
            let size = f.area();
            let input = Paragraph::new(url.as_str())
                .block(Block::default().borders(Borders::ALL).title("Enter URL"));
            f.render_widget(input, size);
        }).unwrap();

        if let Ok(event::Event::Key(KeyEvent { code, .. })) = event::read() {
            match code {
                KeyCode::Enter => {
                    terminal.clear().unwrap();
                    terminal::disable_raw_mode().unwrap();
                    return Some(url);
                }
                KeyCode::Char(c) => {
                    url.push(c);
                }
                KeyCode::Backspace => {
                    url.pop();
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



// pub fn file_selector_ui() -> Option<String> {
//     let mut current_dir = std::env::current_dir().expect("Failed to get current directory");
//     let mut file_entries = get_file_entries(&current_dir);
//     let mut selected_index = 0;""

//     let backend = ratatui::backend::CrosstermBackend::new(stdout());
//     let mut terminal = Terminal::new(backend).unwrap();

//     struct RawModeGuard;
//     impl Drop for RawModeGuard {
//         fn drop(&mut self) {
//             let _ = terminal::disable_raw_mode();
//         }
//     }
//     let _guard = RawModeGuard;

//     terminal::enable_raw_mode().unwrap();
//     terminal.clear().unwrap();

//     loop {
//         terminal.draw(|f| {
//             let size = f.area();
//             let items: Vec<ListItem> = file_entries
//                 .iter()
//                 .enumerate()
//                 .map(|(i, entry)| {
//                     if i == selected_index {
//                         ListItem::new(format!("=> {}", entry)).style(Style::default().fg(Color::Black).bg(Color::Yellow))
//                     } else {
//                         ListItem::new(entry.clone())
//                     }
//                 })
//                 .collect();

//             let list = List::new(items)
//                 .block(Block::default().borders(Borders::ALL).title(format!("Select a File - {:?}", current_dir)));

//             f.render_widget(list, size);
//         }).unwrap();

//         if let Ok(event::Event::Key(KeyEvent { code, .. })) = event::read() {
//             match code {
//                 KeyCode::Up => {
//                     if selected_index > 0 {
//                         selected_index -= 1;
//                     }
//                 }
//                 KeyCode::Down => {
//                     if selected_index < file_entries.len() - 1 {
//                         selected_index += 1;
//                     }
//                 }
//                 KeyCode::Enter => {
//                     let selected = &file_entries[selected_index];
//                     let selected_path = current_dir.join(selected);

//                     if selected == ".." {
//                         if let Some(parent) = current_dir.parent() {
//                             current_dir = parent.to_path_buf();
//                             file_entries = get_file_entries(&current_dir);
//                             selected_index = 0;
//                         }
//                     } else if selected_path.is_dir() {
//                         current_dir = selected_path;
//                         file_entries = get_file_entries(&current_dir);
//                         selected_index = 0;
//                     } else {
//                         // Close UI before returning the file
//                         terminal.clear().unwrap();
//                         terminal::disable_raw_mode().unwrap();
//                         return Some(selected_path.to_string_lossy().into_owned());
//                     }
//                 }
//                 KeyCode::Esc => {
//                     terminal.clear().unwrap();
//                     terminal::disable_raw_mode().unwrap();
//                     return None;
//                 }
//                 _ => {}
//             }
//         }
//     }
// }
