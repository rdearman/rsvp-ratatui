#![allow(unused_mut)]
use std::fs;
use std::io::{Read, Write, Cursor, stdout};
use std::fs::{File, read_dir};
use dirs_next::home_dir;
use crossterm::event::{self, KeyCode, KeyEvent};
use crossterm::terminal;
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::style::{Style, Color};
use ratatui::Terminal;
use epub::doc::EpubDoc;
use pdf_extract::extract_text;
use scraper::{Html, Selector};
use pulldown_cmark::{Parser, Options, Event};
use zip::read::ZipArchive;
use xml::reader::{EventReader, XmlEvent};
use ratatui::layout::{Constraint, Direction, Layout};

/// List of supported file types
const SUPPORTED_FILE_TYPES: &[&str] = &["pdf",  "docx", "odt", "txt", "html", "htm", "md"]; // Removed "epub" because it was crashing


pub fn file_selector_ui() -> Option<String> {
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

    loop {
        terminal.draw(|f| {
            let size = f.area();
            let items: Vec<ListItem> = file_entries
                .iter()
                .enumerate()
                .map(|(i, entry)| {
                    if i == selected_index {
                        ListItem::new(format!("=> {}", entry)).style(Style::default().fg(Color::Black).bg(Color::Yellow))
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
                        // Close UI before returning the file
                        terminal.clear().unwrap();
                        terminal::disable_raw_mode().unwrap();
                        return Some(selected_path.to_string_lossy().into_owned());
                    }
                }
                KeyCode::Esc => {
                    terminal.clear().unwrap();
                    terminal::disable_raw_mode().unwrap();
                    return None;
                }
                _ => {}
            }
        }
    }
}


/// Helper function to get entries of a directory
fn get_file_entries(dir: &std::path::Path) -> Vec<String> {
    let mut entries = Vec::new();

    // Add ".." option to move up a directory
    if dir.parent().is_some() {
        entries.push("..".to_string());
    }

    if let Ok(read_dir) = read_dir(dir) {
        for entry in read_dir.flatten() {
            let file_name = entry.file_name().to_string_lossy().into_owned();
            entries.push(file_name);
        }
    }

    entries
}




pub fn save_settings(speed: u64, chunk_size: usize) {
    if let Some(home) = home_dir() {
        let settings_path = home.join(".rsvp_settings");
        let mut file = File::create(settings_path).expect("Failed to save settings.");
        writeln!(file, "speed={}", speed).unwrap(); // Ensure Write is in scope
        writeln!(file, "chunk_size={}", chunk_size).unwrap();
    }
}



/// Load settings from the user's home directory
pub fn load_settings() -> (Option<u64>, Option<usize>) {
    if let Some(home) = home_dir() {
        let settings_path = home.join(".rsvp_settings");
        if settings_path.exists() {
            let mut file = File::open(settings_path).expect("Failed to open settings file.");
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();

            let mut speed = None;
            let mut chunk_size = None;

            for line in content.lines() {
                if line.starts_with("speed=") {
                    speed = line[6..].parse::<u64>().ok();
                } else if line.starts_with("chunk_size=") {
                    chunk_size = line[11..].parse::<usize>().ok();
                }
            }

            return (speed, chunk_size);
        }
    }
    (None, None)
}



/* supports: PDF, docx, txt, html, MD */
pub fn read_file_content(file_path: &str) -> Vec<String> {
    if file_path.ends_with(".pdf") {
        // Extract text from PDF
        match extract_text(file_path) {
            Ok(text) => text.split_whitespace().map(String::from).collect(),
            Err(_) => panic!("Failed to extract text from PDF"),
        }
    } else if file_path.ends_with(".epub") {
        // Extract text from EPUB
        match EpubDoc::new(file_path) {
            Ok(mut doc) => {
                let mut text = String::new();
                while let Ok(page) = doc.get_current_str() {
                    text.push_str(&page);
                    doc.go_next();
                }
                text.split_whitespace().map(String::from).collect()
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

    text.split_whitespace().map(String::from).collect()
    } else if file_path.ends_with(".html") || file_path.ends_with(".htm") {
        // Extract text from HTML
        let content = fs::read_to_string(file_path).expect("Failed to read HTML file");
        let document = Html::parse_document(&content);
        let selector = Selector::parse("body").unwrap();
        let text = document
            .select(&selector)
            .map(|e| e.text().collect::<Vec<_>>().join(" "))
            .collect::<Vec<String>>()
            .join(" ");
        text.split_whitespace().map(String::from).collect()
    } else if file_path.ends_with(".md") {
        // Extract text from Markdown
        let content = fs::read_to_string(file_path).expect("Failed to read Markdown file");
        let parser = Parser::new_ext(&content, Options::all());
        let text = parser
            .filter_map(|event| match event {
                Event::Text(t) => Some(t.to_string()),
                _ => None,
            })
            .collect::<Vec<String>>()
            .join(" ");
        text.split_whitespace().map(String::from).collect()
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

        text.split_whitespace().map(String::from).collect()

    } else {
        // Default to plain text files
        fs::read_to_string(file_path)
            .expect("Failed to read input file")
            .split_whitespace()
            .map(String::from)
            .collect::<Vec<_>>()
    }

}


