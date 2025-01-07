use crossterm::{
    execute,
    terminal::{self, ClearType},
    cursor::{self, MoveTo},
    event::{self, Event, KeyCode, KeyEvent},
};
use std::io::{stdout, Write};
use clap::{Command, Arg};

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
    let mut speed: u64 = matches
        .get_one::<String>("speed")
        .unwrap()
        .parse()
        .expect("Speed must be a number");

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
    let mut chunk_size = 1;

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

        // Calculate and display the progress bar
        let progress_percentage = (index * 100 / words.len()) as u16;
        let progress_bar_width = cols / 2; // 50% of the screen width
        let left_margin = (cols - progress_bar_width) / 2; // 25% blank space on each side
        let progress_filled = progress_percentage as usize * progress_bar_width as usize / 100;
        let progress_bar = format!(
            "[{}{}]",
            "#".repeat(progress_filled),
            "-".repeat((progress_bar_width as usize).saturating_sub(progress_filled))
        );

        execute!(stdout, MoveTo(left_margin, rows - 2)).unwrap();
        print!("{}", progress_bar);

        execute!(stdout, MoveTo(left_margin, rows - 1)).unwrap();
        print!("Progress: {}%", progress_percentage);

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
                        } else {
                            let _ = prompt_user("Invalid speed. Press Enter to continue.");
                        }
                    }
                    KeyCode::Char('c') => {
                        let input = prompt_user("Enter chunk size (number of words):");
                        if let Ok(new_chunk) = input.parse::<usize>() {
                            chunk_size = std::cmp::max(1, new_chunk);
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
    println!("Program terminated.");
}
