use crossterm::{
    execute,
    terminal::{self, ClearType},
    cursor::{self, MoveTo},
    event::{self, Event, KeyCode, KeyEvent},
};
use std::io::{stdout, Write};
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

        // Redraw the current word and UI when not paused or when paused state changes
        if !paused {
            execute!(stdout, terminal::Clear(ClearType::All)).unwrap();

            // Display the current word
            let word = &words[index];
            let x = (cols / 2) - (word.len() as u16 / 2);
            let y = rows / 2;

            execute!(stdout, MoveTo(x, y)).unwrap();
            print!("{}", word);

            // Display the speed below the word
            let speed_text = format!("Speed: {} WPM", speed);
            let speed_x = (cols / 2) - (speed_text.len() as u16 / 2);
            execute!(stdout, MoveTo(speed_x, (rows / 2) + 2)).unwrap();
            print!("{}", speed_text);

            // Always display the bottom menu
            let menu_text = "[Up: +10] [Down: -10] [PgUp: +100] [PgDn: -100] [Space: Pause/Resume] [Q: Quit]";
            let menu_text2 = "[Load File: L] [Set Speed: S] [Skip Forward: Right] [Skip Back: Left] [Chunk Size: (1=default)] [Q: Quit]";	    
            execute!(stdout, MoveTo(0, rows - 2)).unwrap();
            print!("{:^width$}", menu_text, width = cols as usize);
            execute!(stdout, MoveTo(0, rows - 1)).unwrap();
            print!("{:^width$}", menu_text2, width = cols as usize);

            // Flush the output
            stdout.flush().unwrap();
        }

        // Wait for user input or timeout
        if event::poll(std::time::Duration::from_millis(60000 / speed)).unwrap() {
            if let Event::Key(KeyEvent { code, .. }) = event::read().unwrap() {
                match code {
                    KeyCode::Up => {
                        // Increase speed in increments of 10
                        speed += 10;
                        paused = false; // Ensure UI updates immediately after a speed change
                    }
                    KeyCode::Down => {
                        // Decrease speed in increments of 10
                        if speed > 10 {
                            speed -= 10;
                        }
                        paused = false; // Ensure UI updates immediately after a speed change
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

        // Only move to the next word if not paused
        if !paused {
            index += 1;
        }
    }

    // Restore the terminal to its normal state
    execute!(stdout, cursor::Show).unwrap();
    terminal::disable_raw_mode().unwrap();

    println!("Program terminated.");
}
