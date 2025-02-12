use std::io::{Read, Write, stdout};
use std::fs::{File, read_dir};
use dirs_next::home_dir;
use crossterm::event::{self, KeyCode, KeyEvent};
use crossterm::terminal;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::style::{Style, Color};
use ratatui::Terminal;
use crate::utilities::terminal::enable_raw_mode;
use crate::utilities::terminal::disable_raw_mode;


/// File Selector UI Function
pub fn file_selector_ui() -> Option<String> {
    let mut current_dir = std::env::current_dir().expect("Failed to get current directory");
    let mut file_entries = get_file_entries(&current_dir);
    let mut selected_index = 0;

    enable_raw_mode().unwrap();
    let backend = ratatui::backend::CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend).unwrap();

    struct RawModeGuard;
    impl Drop for RawModeGuard {
        fn drop(&mut self) {
            let _ = disable_raw_mode();
        }
    }
    let _guard = RawModeGuard;

    // Clear the screen before drawing the UI
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
                .block(Block::default().borders(Borders::ALL).title("Select a File"));

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

                    if selected_path.is_dir() {
                        current_dir = selected_path;
                        file_entries = get_file_entries(&current_dir);
                        selected_index = 0;
                    } else {
                        terminal.clear().unwrap();
                        return Some(selected_path.to_string_lossy().into_owned());
                    }
                }
                KeyCode::Esc => {
                    terminal.clear().unwrap();
                    return None;
                }
                _ => {}
            }
        }
    }
}




/// Helper function to get entries of a directory
fn get_file_entries(dir: &std::path::Path) -> Vec<String> {
    read_dir(dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect()
}




/* ==================================== */

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
