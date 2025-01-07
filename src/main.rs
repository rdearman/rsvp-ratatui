use crossterm::{
    execute,
    terminal::{self, ClearType},
    cursor::{self, MoveTo},
};
use std::io::{stdout, Write};

fn main() {
    let words = vec!["Hello", "World", "This", "is", "Crossterm"];
    let mut stdout = stdout();

    // Enable raw mode
    terminal::enable_raw_mode().unwrap();

    // Hide the cursor
    execute!(stdout, cursor::Hide).unwrap();

    // Get terminal size
    let (cols, rows) = terminal::size().unwrap();

    for word in words {
        // Clear the screen
        execute!(stdout, terminal::Clear(ClearType::All)).unwrap();

        // Calculate the position to center the word
        let x = (cols / 2) - (word.len() as u16 / 2);
        let y = rows / 2;

        // Move cursor and write the word
        execute!(stdout, MoveTo(x, y)).unwrap();
        println!("{}", word);

        // Wait for a moment
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    // Show the cursor again
    execute!(stdout, cursor::Show).unwrap();

    // Disable raw mode
    terminal::disable_raw_mode().unwrap();
}
