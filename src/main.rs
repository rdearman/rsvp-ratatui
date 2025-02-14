mod interface;
mod utilities;
use clap::{Arg, Command};
use crate::utilities::{load_settings, save_settings,file_selector_ui, read_file_content};
use std::collections::HashMap;
use serde_json::{json, Value};
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
        .or_else(|| file_selector_ui());

    if let Some(file_path) = input_file {
        let absolute_path = fs::canonicalize(Path::new(&file_path))
            .unwrap_or_else(|_| Path::new(&file_path).to_path_buf());

        let absolute_path_str = absolute_path.to_string_lossy().to_string();

        let words = read_file_content(&absolute_path_str);
        let total_words = words.len();

        // ✅ Fix: Restore book data handling
        // let book_settings = book_data.get(&absolute_path_str).and_then(|b| b.as_object());
        let book_settings = book_data.entry(absolute_path_str.clone()).or_insert_with(|| json!({
            "bookmarks": [],
            "speed": global_speed,
            "chunk_size": global_chunk_size,
            "last_position": 0
        }));


        let speed = matches
            .get_one::<String>("speed")
            .and_then(|s| s.parse().ok())
            .or_else(|| book_settings.as_object().and_then(|b| b.get("speed")?.as_u64()))
            .unwrap_or(global_speed);

        let chunk_size = matches
            .get_one::<String>("chunk_size")
            .and_then(|cs| cs.parse().ok())
            .or_else(|| book_settings.as_object().and_then(|b| b.get("chunk_size")?.as_u64()).map(|cs| cs as usize))
            .unwrap_or(global_chunk_size);

        let _last_position = book_settings
            .as_object().and_then(|b| b.get("last_position")?.as_u64())
            .unwrap_or(0) as usize;

        // ✅ Fix: Ensure correct arguments to run_ui()
        let final_position = interface::run_ui(speed, chunk_size, total_words, words, &mut book_data, global_speed, global_chunk_size);
        
        book_data.entry(absolute_path_str.clone()).or_insert_with(|| json!({}));
        if let Some(book) = book_data.get_mut(&absolute_path_str) {
            book["last_position"] = json!(final_position);
            book["speed"] = json!(speed);
            book["chunk_size"] = json!(chunk_size);
        }
        save_settings(global_speed, global_chunk_size, book_data, None, None);
    }
}
