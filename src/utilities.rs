use std::fs::{self, File};
use std::io::Write;
use dirs_next::home_dir;

/// Save the user's settings to a file in their home directory
fn save_settings(speed: u64, chunk_size: usize) {
    if let Some(home) = home_dir() {
        let settings_path = home.join(".rsvp_settings");
        let mut file = File::create(settings_path).expect("Failed to save settings.");
        writeln!(file, "speed={}", speed).unwrap();
        writeln!(file, "chunk_size={}", chunk_size).unwrap();
    }
}

/// Load settings from the user's home directory
fn load_settings() -> (Option<u64>, Option<usize>) {
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

fn load_words_from_file() -> Vec<String> {
    loop {
        //println!("Debug: Entering load_words_from_file loop.");

        // Reset terminal state before taking input
        reset_terminal_state();

        println!("Enter file path:");
        let mut file_path = String::new();

        match std::io::stdin().read_line(&mut file_path) {
            Ok(_) => {
                file_path = file_path.trim().to_string();
                // println!("Debug: File path entered: {}", file_path);

                match std::fs::read_to_string(&file_path) {
                    Ok(content) => {
                        //println!("Debug: File loaded successfully.");
                        let words = content.split_whitespace().map(String::from).collect();
                        println!("File loaded successfully!");
                        return words;
                    }
                    Err(err) => {
                        println!("Debug: Failed to load file: {}.", err);

                        // Reset terminal state before retry prompt
                        reset_terminal_state();

                        println!("Retry? (y/n):");
                        let mut retry = String::new();
                        match std::io::stdin().read_line(&mut retry) {
                            Ok(_) => {
                                let retry = retry.trim().to_lowercase();
                                //println!("Debug: Retry response: {}", retry);

                                if retry != "y" {
                                    println!("Returning to the main menu.");
                                    return vec![];
                                }
                            }
                            Err(e) => {
                                println!("Debug: Failed to capture retry response: {}", e);
                                return vec![];
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("Debug: Failed to read file path: {}", e);
                return vec![];
            }
        }
    }
}