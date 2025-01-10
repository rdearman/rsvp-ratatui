mod interface;
mod utilities;

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
    let mut speed: u64 = matches
        .get_one::<String>("speed")
        .and_then(|s| s.parse().ok())
        .or(saved_speed)
        .unwrap_or(250); // Default speed

    let mut chunk_size: usize = matches
        .get_one::<String>("chunk_size")
        .and_then(|cs| cs.parse().ok())
        .or(saved_chunk_size)
        .unwrap_or(1); // Default chunk size
		
    // Save updated settings
    save_settings(speed, chunk_size);

    // Get the input file or default to help text
    let input_file = matches.get_one::<String>("input").map(String::as_str).unwrap_or("default_help.txt");
    let mut words = if input_file == "default_help.txt" {
        vec![
            "Welcome to RSVP!".to_string(),
            "This program displays one word at a time in the terminal.".to_string(),
            "Use the up and down arrows to adjust speed.".to_string(),
            "Press space to pause or resume.".to_string(),
            "Press 'q' to quit.".to_string(),
        ]
    } else {
        std::fs::read_to_string(input_file)
            .expect("Failed to read input file")
            .split_whitespace()
            .map(String::from)
            .collect::<Vec<_>>()
    };		
		
    // Launch the UI
    interface::run_ui(speed, chunk_size, total_words);
}
