use crossterm::{
    execute,
    terminal::{self, ClearType},
    cursor::{self, MoveTo},
    event::{self, Event, KeyCode, KeyEvent},
};
use std::io::{stdout};
use clap::{Command, Arg};

fn main() {
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

    println!("Input file: {}", input_file);
    println!("Speed: {} words per minute", speed);

    let words = if input_file == "default_help.txt" {
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

        // Display the current word only when not paused
        if !paused {
            execute!(stdout, terminal::Clear(ClearType::All)).unwrap();
            let word = &words[index];
            let x = (cols / 2) - (word.len() as u16 / 2);
            let y = rows / 2;

            execute!(stdout, MoveTo(x, y)).unwrap();
            println!("{}", word);

            // Move to the next word
            index += 1;
        }

        // Wait for the specified time or handle key events
        if event::poll(std::time::Duration::from_millis(60000 / speed)).unwrap() {
            if let Event::Key(KeyEvent { code, .. }) = event::read().unwrap() {
                match code {
                    KeyCode::Up => {
                        // Increase speed
                        speed += 10;
                    }
                    KeyCode::Down => {
                        // Decrease speed
                        if speed > 10 {
                            speed -= 10;
                        }
                    }
                    KeyCode::Char('q') => {
                        // Quit the program
                        break;
                    }
                    KeyCode::Char(' ') => {
                        // Pause or resume
                        paused = !paused;
                    }
                    _ => {}
                }
            }
        }
    }

    execute!(stdout, cursor::Show).unwrap();
    terminal::disable_raw_mode().unwrap();

    println!("Program terminated.");
}
