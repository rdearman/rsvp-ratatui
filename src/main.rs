mod interface;
mod utilities;
use clap::{Arg, Command};
use crate::utilities::{load_settings, save_settings,file_selector_ui, read_file_content};

fn main() {
    // Load saved preferences
    let (saved_speed, saved_chunk_size) = load_settings();

    // Parse command-line arguments
    let matches = Command::new("RSVP")
        .version("1.0")
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

    // Determine speed and chunk size with priority: Command-line > Saved Preferences > Defaults
    let speed: u64 = matches
        .get_one::<String>("speed")
        .and_then(|s| s.parse().ok())
        .or(saved_speed)
        .unwrap_or(250); // Default speed

    let chunk_size: usize = matches
        .get_one::<String>("chunk_size")
        .and_then(|cs| cs.parse().ok())
        .or(saved_chunk_size)
        .unwrap_or(1); // Default chunk size

    // Save updated settings
    save_settings(speed, chunk_size);

    let words = if let Some(input_file) = matches.get_one::<String>("input") {
        read_file_content(input_file)
    } else {
        file_selector_ui()
            .map(|s| s.split_whitespace().map(String::from).collect::<Vec<_>>())
            .unwrap_or_else(Vec::new)
    };

    let total_words = words.len();

    // Pass words to the UI
    interface::run_ui(speed, chunk_size, total_words, words);
}
