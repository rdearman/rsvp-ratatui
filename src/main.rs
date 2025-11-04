mod interface;
mod utilities;
use clap::{Arg, Command};
use crate::utilities::{load_settings, save_settings, read_file_content, read_file_sentences};
use crate::interface::DisplayMode;
use serde_json::json;
use std::fs;
use std::path::Path;


fn main() {
    // Load settings from JSON
    let (global_speed, global_chunk_size, mut book_data) = load_settings(); // ✅ Fix: Ensure book_data is loaded

    let matches = Command::new("RSVP")
        .version("1.2.0")
        .author("Your Name <you@example.com>")
        .about("Displays one word at a time in the terminal")
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .num_args(1)
                .help("The input file containing words"),
        )
        .arg(
            Arg::new("speed")
                .short('s')
                .long("speed")
                .num_args(1)
                .help("Speed in words per minute (overrides saved preference)"),
        )
        .arg(
            Arg::new("chunk_size")
                .short('c')
                .long("chunk-size")
                .num_args(1)
                .help("Number of words per chunk (overrides saved preference)"),
        )
        .get_matches();

    let input_file = matches.get_one::<String>("input").cloned()
        .or_else(|| utilities::load_file_menu_ui(&book_data));

    if let Some(file_path) = input_file {
        let absolute_path = fs::canonicalize(Path::new(&file_path))
            .unwrap_or_else(|_| Path::new(&file_path).to_path_buf());

        let absolute_path_str = absolute_path.to_string_lossy().to_string();

        let book_settings = book_data.entry(absolute_path_str.clone()).or_insert_with(|| json!({
            "bookmarks": [],
            "speed": global_speed,
            "chunk_size": global_chunk_size,
            "last_position": 0
        }));

        let chunk_size = matches
            .get_one::<String>("chunk_size")
            .and_then(|cs| cs.parse().ok())
            .or_else(|| book_settings.as_object().and_then(|b| b.get("chunk_size")?.as_u64()).map(|cs| cs as usize))
            .unwrap_or(global_chunk_size);

        let display_mode = book_settings.as_object()
            .and_then(|b| b.get("display_mode"))
            .and_then(|dm| dm.as_str())
            .map(|s| {
                if s == "sentence" {
                    DisplayMode::Sentence
                } else {
                    DisplayMode::WordChunk(chunk_size)
                }
            })
            .unwrap_or(DisplayMode::WordChunk(chunk_size));

        let words = match display_mode {
            DisplayMode::WordChunk(_) => read_file_content(&absolute_path_str),
            DisplayMode::Sentence => read_file_sentences(&absolute_path_str),
        };
        let total_words = words.len();

        let speed = matches
            .get_one::<String>("speed")
            .and_then(|s| s.parse().ok())
            .or_else(|| book_settings.as_object().and_then(|b| b.get("speed")?.as_u64()))
            .unwrap_or(global_speed);

        let _last_position = book_settings
            .as_object().and_then(|b| b.get("last_position")?.as_u64())
            .unwrap_or(0) as usize;

        // ✅ Fix: Ensure correct arguments to run_ui()
        // let final_position = interface::run_ui(speed, chunk_size, total_words, words, &mut book_data, absolute_path_str.clone());        
        let final_position = interface::run_ui(
            speed,
            display_mode,
            total_words,
            words,
            &mut book_data,
            global_speed,          // ✅ Add missing global_speed
            global_chunk_size,     // ✅ Add missing global_chunk_size
            absolute_path_str.clone(), // ✅ Correct file_path position
            _last_position,
        );

        book_data.entry(absolute_path_str.clone()).or_insert_with(|| json!({}));
        if let Some(book) = book_data.get_mut(&absolute_path_str) {
            book["last_position"] = json!(final_position);
            book["speed"] = json!(speed);
            if let DisplayMode::WordChunk(size) = display_mode {
                book["chunk_size"] = json!(size);
            } else {
                book["chunk_size"] = json!(global_chunk_size);
            }
            book["display_mode"] = match display_mode {
                DisplayMode::Sentence => json!("sentence"),
                DisplayMode::WordChunk(_) => json!("word_chunk"),
            };
        }
        save_settings(global_speed, global_chunk_size, book_data, None, None);
    }
}
