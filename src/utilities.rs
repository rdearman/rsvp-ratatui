use std::io::{self, stdout};
use dirs_next::home_dir;
use std::fs::{self, File};
use std::io::{Read, Write}; // Ensure Write is included

use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use crossterm::{event::{self, KeyCode, KeyEvent}, terminal};

pub fn save_settings(speed: u64, chunk_size: usize) {
    if let Some(home) = home_dir() {
        let settings_path = home.join(".rsvp_settings");
        let mut file = File::create(settings_path).expect("Failed to save settings.");
        writeln!(file, "speed={}", speed).unwrap(); // Ensure Write is in scope
        writeln!(file, "chunk_size={}", chunk_size).unwrap();
    }
}


pub fn load_words_from_file_ui() -> Vec<String> {
    let mut file_path = String::new();
    let mut terminal = {
        terminal::enable_raw_mode().unwrap();
        let backend = CrosstermBackend::new(stdout());
        Terminal::new(backend).unwrap()
    };

    loop {
        terminal.draw(|f| {
            let size = f.size();

            // Main layout
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(20), // Top spacer
                    Constraint::Percentage(60), // Form block
                    Constraint::Percentage(20), // Bottom spacer
                ])
                .split(size);

            // Input form
            let input_block = Paragraph::new(file_path.clone())
                .block(Block::default().borders(Borders::ALL).title("Enter File Path"));
            f.render_widget(input_block, chunks[1]);
        }).unwrap();

        if let Ok(event::Event::Key(KeyEvent { code, .. })) = event::read() {
            match code {
                KeyCode::Enter => {
                    if let Ok(content) = std::fs::read_to_string(&file_path) {
                        terminal::disable_raw_mode().unwrap();
                        return content.split_whitespace().map(String::from).collect();
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
                    return vec![];
                }
                _ => {}
            }
        }
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
