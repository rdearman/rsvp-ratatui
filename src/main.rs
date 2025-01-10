use crossterm::{
    execute,
    terminal::{self, disable_raw_mode, enable_raw_mode},
    event::{self, Event, KeyCode},
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Layout, Constraint, Direction};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::text::Text;
use ratatui::Terminal;
use std::io::{stdout, Write};
use clap::{Command, Arg};
use std::fs::File;
use std::io::{Read, Write as IoWrite};
use dirs_next::home_dir;



fn reset_terminal_state() {
    if let Err(err) = disable_raw_mode() {
        eprintln!("Error disabling raw mode: {}", err);
    }

    if let Err(err) = stdout().flush() {
        eprintln!("Error flushing stdout: {}", err);
    }
}

fn save_settings(speed: u64, chunk_size: usize) {
    if let Some(home) = home_dir() {
        let settings_path = home.join(".rsvp_settings");
        let mut file = File::create(settings_path).expect("Failed to save settings.");
        writeln!(file, "speed={}", speed).unwrap();
        writeln!(file, "chunk_size={}", chunk_size).unwrap();
    }
}

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (saved_speed, saved_chunk_size) = load_settings();

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

    let mut speed: u64 = matches
        .get_one::<String>("speed")
        .and_then(|s| s.parse().ok())
        .or(saved_speed)
        .unwrap_or(250);

    let mut chunk_size: usize = matches
        .get_one::<String>("chunk_size")
        .and_then(|cs| cs.parse().ok())
        .or(saved_chunk_size)
        .unwrap_or(1);

    save_settings(speed, chunk_size);

    let input_file = matches.get_one::<String>("input").map(String::as_str).unwrap_or("default_help.txt");
    let words = if input_file == "default_help.txt" {
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

    enable_raw_mode()?;
    let stdout = stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;
    let mut index = 0;
    let mut paused = false;

    loop {
        terminal.draw(|f| {
            let size = f.size();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(80),
                    Constraint::Percentage(20),
                ].as_ref())
                .split(size);

            let word_chunk = &words[index..std::cmp::min(index + chunk_size, words.len())]
                .join(" ");

            let word_paragraph = Paragraph::new(word_chunk.clone())
                .block(Block::default().borders(Borders::ALL).title("Word Display"));

            f.render_widget(word_paragraph, chunks[0]);

            let control_text = Paragraph::new(
                Text::from("[Up/Down: Adjust Speed | Space: Pause | Q: Quit]"),
            )
            .block(Block::default().borders(Borders::ALL).title("Controls"));

            f.render_widget(control_text, chunks[1]);
        })?;

        if event::poll(std::time::Duration::from_millis(60000 / speed))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Up => speed += 10,
                    KeyCode::Down => if speed > 10 { speed -= 10 },
                    KeyCode::Right => index = std::cmp::min(index + chunk_size, words.len() - 1),
                    KeyCode::Left => index = index.saturating_sub(chunk_size),
                    KeyCode::Char(' ') => paused = !paused,
                    KeyCode::Char('q') => break,
                    _ => {}
                }
            }
        }

        if !paused {
            index += chunk_size;
            if index >= words.len() {
                break;
            }
        }
    }

    disable_raw_mode()?;
    Ok(())
}
