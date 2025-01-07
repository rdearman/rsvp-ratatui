use crossterm::{
    execute,
    terminal::{self, ClearType},
    cursor::{self, MoveTo},
    event::{self, Event, KeyCode, KeyEvent},
};
use std::io::{stdout, Write};
use clap::{Command, Arg};
use std::fs::{self, File};
use std::io::{Read, Write as IoWrite};
use std::path::PathBuf;
use dirs_next::home_dir; // Use dirs-next crate for user's home directory


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
fn load_settings() -> (u64, usize) {
    if let Some(home) = home_dir() {
        let settings_path = home.join(".rsvp_settings");
        if settings_path.exists() {
            let mut file = File::open(settings_path).expect("Failed to open settings file.");
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();

            let mut speed = 250;
            let mut chunk_size = 1;

            for line in contents.lines() {
                if line.starts_with("speed=") {
                    speed = line[6..].parse().unwrap_or(250);
                } else if line.starts_with("chunk_size=") {
                    chunk_size = line[11..].parse().unwrap_or(1);
                }
            }
            return (speed, chunk_size);
        }
    }
    (250, 1) // Default settings
}

fn prompt_user(prompt: &str) -> String {
    terminal::disable_raw_mode().unwrap();
    execute!(stdout(), terminal::Clear(ClearType::All)).unwrap();
    println!("{}", prompt);

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    terminal::enable_raw_mode().unwrap();
    input.trim().to_string()
}

fn main() {
    let matches = Command::new("RSVP")
        .version("1.0")
        .author("Rick Dearman")
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
                .default_value("250")
                .help("Speed in words per minute"),
        )
        .get_matches();

    let input_file = matches.get_one::<String>("input").map(String::as_str).unwrap_or("default_help.txt");

    // Load settings from file
    let (mut speed, mut chunk_size) = load_settings();

    if let Some(arg_speed) = matches.get_one::<String>("speed") {
        speed = arg_speed.parse().expect("Speed must be a number");
    }

    let mut words = if input_file == "default_help.txt" {
        vec![
            "Welcome to RSVP!".to_string(),
            "This program displays one word at a time.".to_string(),
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

    let mut stdout = stdout();
    terminal::enable_raw_mode().unwrap();
    execute!(stdout, cursor::Hide).unwrap();
    let (cols, rows) = terminal::size().unwrap();

    let mut index = 0;
    let mut paused = false;

    loop {
        if index >= words.len() {
            break;
        }

        execute!(stdout, terminal::Clear(ClearType::All)).unwrap();

        // Display the current word(s)
        let chunk = words[index..std::cmp::min(index + chunk_size, words.len())]
            .join(" ");
        let x = (cols / 2) - (chunk.len() as u16 / 2);
        let y = rows / 2;

        execute!(stdout, MoveTo(x, y)).unwrap();
        print!("{}", chunk);

        // Display the speed below the word
        let speed_text = format!("Speed: {} WPM", speed);
        let speed_x = (cols / 2) - (speed_text.len() as u16 / 2);
        execute!(stdout, MoveTo(speed_x, (rows / 2) + 2)).unwrap();
        print!("{}", speed_text);

        // Always display the bottom menu
        let menu_text = "[Up: +10] [Down: -10] [PgUp: +100] [PgDn: -100] [Space: Pause/Resume] [Q: Quit]";
        let menu_text2 = "[Load File: L] [Set Speed: S] [Skip Forward: Right] [Skip Back: Left] [Chunk Size: C]";
        execute!(stdout, MoveTo(0, rows - 4)).unwrap();
        print!("{:^width$}", menu_text, width = cols as usize);
        execute!(stdout, MoveTo(0, rows - 3)).unwrap();
        print!("{:^width$}", menu_text2, width = cols as usize);

        stdout.flush().unwrap();

        if event::poll(std::time::Duration::from_millis(60000 / speed)).unwrap() {
            if let Event::Key(KeyEvent { code, .. }) = event::read().unwrap() {
                match code {
                    KeyCode::Up => speed += 10,
                    KeyCode::Down => if speed > 10 { speed -= 10 },
                    KeyCode::PageUp => speed += 100,
                    KeyCode::PageDown => if speed > 100 { speed -= 100 },
                    KeyCode::Right => index = std::cmp::min(index + chunk_size, words.len() - 1),
                    KeyCode::Left => index = index.saturating_sub(chunk_size),
                    KeyCode::Char(' ') => paused = !paused,
                    KeyCode::Char('l') => {
                        let file = prompt_user("Enter file path:");
                        if let Ok(content) = std::fs::read_to_string(file) {
                            words = content.split_whitespace().map(String::from).collect();
                            index = 0;
                            paused = false;
                        } else {
                            let _ = prompt_user("Failed to load file. Press Enter to continue.");
                        }
                    }
                    KeyCode::Char('s') => {
                        let input = prompt_user("Enter new speed (WPM):");
                        if let Ok(new_speed) = input.parse::<u64>() {
                            speed = new_speed;
                            save_settings(speed, chunk_size); // Save settings
                        } else {
                            let _ = prompt_user("Invalid speed. Press Enter to continue.");
                        }
                    }
                    KeyCode::Char('c') => {
                        let input = prompt_user("Enter chunk size (number of words):");
                        if let Ok(new_chunk) = input.parse::<usize>() {
                            chunk_size = std::cmp::max(1, new_chunk);
                            save_settings(speed, chunk_size); // Save settings
                        } else {
                            let _ = prompt_user("Invalid chunk size. Press Enter to continue.");
                        }
                    }
                    KeyCode::Char('q') => break,
                    _ => {}
                }
            }
        }

        if !paused {
            index += chunk_size;
        }
    }

    execute!(stdout, cursor::Show).unwrap();
    terminal::disable_raw_mode().unwrap();

    // Save settings on exit
    save_settings(speed, chunk_size);

    println!("Program terminated.");
}
