use crossterm::{
    execute,
    terminal::{self,Clear, ClearType},
    cursor::{self, MoveTo},
    event::{self, Event, KeyCode, KeyEvent},
};
use std::io::{stdout, Write};
use clap::{Command, Arg};
use std::fs::File;
use std::io::{Read, Write as IoWrite};
use dirs_next::home_dir;

fn prompt_user(prompt: &str) -> String {
    println!("{}", prompt);

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    // Re-enable raw mode after input
    terminal::enable_raw_mode().unwrap();
    input.trim().to_string()
}

fn set_preferences(mut speed: u64, mut chunk_size: usize) {
    loop {
        terminal::disable_raw_mode().unwrap();

        // Create a mutable handle to stdout
        let mut out = stdout();

        // Clear the terminal and reset cursor
        execute!(out, Clear(ClearType::All), MoveTo(0, 0)).unwrap();

        // Display the options to the user
        println!("Set Preferences:");
        println!("1. Set Speed");
        println!("2. Set Chunk Size");
        println!("3. Save Preferences");
        println!("4. Cancel");

        // Flush the output to ensure it's displayed
        out.flush().unwrap();

        // Prompt the user for a choice
        let choice = prompt_user("Enter your choice (1-4):");
        terminal::enable_raw_mode().unwrap();

        match choice.as_str() {
            "1" => {
                terminal::disable_raw_mode().unwrap();
                let mut out = stdout();
                execute!(out, Clear(ClearType::All)).unwrap();
                let speed_input = prompt_user("Enter the new speed (in words per minute):");
                if let Ok(new_speed) = speed_input.parse::<u64>() {
                    speed = new_speed;
                    println!("Speed updated to {} wpm.", speed);
                } else {
                    println!("Invalid speed value. Please try again.");
                }
            }
            "2" => {
                terminal::disable_raw_mode().unwrap();
                let mut out = stdout();
                execute!(out, Clear(ClearType::All)).unwrap();
                let chunk_input = prompt_user("Enter the new chunk size (number of words):");
                if let Ok(new_chunk_size) = chunk_input.parse::<usize>() {
                    chunk_size = new_chunk_size;
                    println!("Chunk size updated to {} words.", chunk_size);
                } else {
                    println!("Invalid chunk size value. Please try again.");
                }
            }
            "3" => {
                save_settings(speed, chunk_size);
                println!("Preferences saved successfully!");
                break;
            }
            "4" => {
                println!("Preferences update cancelled.");
                break;
            }
            _ => {
                println!("Invalid choice. Please enter a number between 1 and 4.");
            }
        }
    }
}


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

    // Enable terminal raw mode and prepare for output
    let mut out = stdout();
    terminal::enable_raw_mode().unwrap();
    execute!(out, cursor::Hide).unwrap();

    let (cols, rows) = terminal::size().unwrap();
    let mut index = 0;
    let mut paused = false;

    loop {
        if index >= words.len() {
            break;
        }

        // Clear the screen and draw content
        execute!(out, terminal::Clear(ClearType::All)).unwrap();

        // Display the current word(s)
        let chunk = words[index..std::cmp::min(index + chunk_size, words.len())]
            .join(" ");
        let x = (cols / 2) - (chunk.len() as u16 / 2);
        let y = rows / 2;

        execute!(out, MoveTo(x, y)).unwrap();
        print!("{}", chunk);

        // Display the progress bar
        let progress_percentage = (index as f64 / words.len() as f64) * 100.0;
        let progress_bar_length = (cols / 2) as usize; // 50% of screen width
        let filled_length = (progress_bar_length as f64 * progress_percentage / 100.0).round() as usize;
        let empty_length = progress_bar_length - filled_length;
        let progress_bar = format!(
            "[{}{}]",
            "#".repeat(filled_length),
            "-".repeat(empty_length)
        );

        let progress_bar_x = cols / 4; // Centered with 25% padding on each side
        execute!(out, MoveTo(progress_bar_x, rows - 2)).unwrap();
        print!("{}", progress_bar);

        let progress_percentage_text = format!("{:.2}%", progress_percentage);
        let progress_percentage_x = (cols / 2) - (progress_percentage_text.len() as u16 / 2);
        execute!(out, MoveTo(progress_percentage_x, rows - 1)).unwrap();
        print!("{}", progress_percentage_text);

        // Display the menu at the bottom of the screen, centered
        let menu_text = "Up/Down: Adjust Speed | PgUp/PgDn: Adjust Speed by 100 | Space: Pause/Resume | Q: Quit";
        let menu_text2 = "1-9: Set Chunk Size | L: Load File | P: Set Preferences| Left: Skip Back | Right: Skip Forward";
        let menu_text3 = format!("Current: Speed={} WPM | Chunk Size={} words", speed, chunk_size);

        execute!(out, MoveTo(0, rows - 5)).unwrap();
        print!("{:^width$}\n", menu_text, width = cols as usize);
        execute!(out, MoveTo(0, rows - 4)).unwrap();
        print!("{:^width$}\n", menu_text2, width = cols as usize);
        execute!(out, MoveTo(0, rows - 3)).unwrap();
        print!("{:^width$}\n", menu_text3, width = cols as usize);

        out.flush().unwrap();

        if event::poll(std::time::Duration::from_millis(60000 / speed)).unwrap() {
            if let Event::Key(KeyEvent { code, .. }) = event::read().unwrap() {
                match code {
                    KeyCode::Up => speed += 10,
                    KeyCode::Down => if speed > 10 { speed -= 10 },
                    KeyCode::PageUp => speed += 100,
                    KeyCode::PageDown => if speed > 100 { speed -= 100 },
                    KeyCode::Right => index = std::cmp::min(index + chunk_size, words.len() - 1),
                    KeyCode::Left => index = index.saturating_sub(chunk_size),
                    KeyCode::Char('l') => {
                        // Prompt for file and load words
                        // Disable raw mode to handle user input
                        terminal::disable_raw_mode().unwrap();
                        execute!(out, terminal::Clear(ClearType::All)).unwrap();

                        let file = prompt_user("Enter file path:");
                        if let Ok(content) = std::fs::read_to_string(file) {
                            words = content.split_whitespace().map(String::from).collect();
                            index = 0;
                            paused = false; // Reset paused state
                        } else {
                            let _ = prompt_user("Failed to load file. Press Enter to continue.");
                        }
                    }
                    KeyCode::Char('p') => {
                        // Assume `speed` and `chunk_size` are variables already defined in the current scope
                        set_preferences(speed, chunk_size);
                    }
                    KeyCode::Char(' ') => paused = !paused,
                    KeyCode::Char('q') => break,
                    KeyCode::Char(c) if c.is_digit(10) => {
                        chunk_size = c.to_digit(10).unwrap() as usize;
                    }
                    _ => {}
                }
            }
        }

        if !paused {
            index += chunk_size;
        }
    }

    execute!(out, cursor::Show).unwrap();
    terminal::disable_raw_mode().unwrap();

    println!("Program terminated.");
}
