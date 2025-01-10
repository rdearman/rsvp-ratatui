# RSVP (Rapid Serial Visual Presentation)

RSVP is a terminal-based application that displays text one word at a time, helping users practice speed reading and improve reading efficiency. Users can adjust the reading speed, chunk size (number of words displayed at once), and navigate through text with intuitive controls.

This version of RSVP is a fork of the original application and has been updated to use the Ratatui crate instead of Crossterm. This change enables better display capabilities and more options for customization, making the application more versatile and visually appealing.

## Features

- Display one or more words at a time in the terminal.
- Adjustable speed in words per minute (WPM).
- Configurable chunk size (1-9 words).
- Interactive preferences menu to save and modify settings.
- Real-time progress bar indicating reading completion percentage.
- Supports file input for custom text or default usage instructions.

## Installation

1. **Clone the Repository**:
   ```bash
   git clone https://github.com/rdearman/rsvp-ratatui.git
   cd rsvp-ratatui
   ```

2. **Build the Project**:
   Make sure you have Rust installed. Then, build the project using Cargo:
   ```bash
   cargo build --release
   ```

3. **Run the Application**:
   ```bash
   cargo run -- -i <input_file> -s <speed_in_wpm>
   ```

   Example:
   ```bash
   cargo run -- -i sample.txt -s 300
   ```

## Usage

### Controls
- `[Up]` / `[Down]`: Increase or decrease speed by 10 WPM.
- `[PgUp]` / `[PgDn]`: Increase or decrease speed by 100 WPM.
- `[Right]` / `[Left]`: Skip forward or backward by the current chunk size.
- `[1-9]`: Set chunk size directly to a number (1-9 words).
- `[Space]`: Pause or resume the text display.
- `[C]`: Change chunk size (via number prompt).
- `[S]`: Change speed (via WPM prompt).
- `[P]`: Open preferences menu.
- `[Q]`: Quit the program.

### Preferences Menu
- Modify reading speed and chunk size interactively.
- Save settings to `~/.rsvp_settings` for future sessions.

### Default Help Text
If no file is provided, the program will display a default instructional text to guide users.

## Future Enhancements

- **Word Highlighting**: Highlight key parts of the text (e.g., nouns, verbs) to improve comprehension.
- **Custom Themes**: Add options for light, dark, and high-contrast themes.
- **Keyboard Shortcuts Display**: Dynamically update and show shortcuts relevant to the current state (e.g., hide `[PgUp]` if not applicable).
- **Dynamic Progress Updates**: Allow users to toggle a percentage-only progress indicator instead of the full bar.
- **Multi-File Input**: Enable loading multiple files and navigating between them.
- **Session Summary**: Show total reading time, average speed, and other statistics upon completion.
- **Audio Cues**: Add optional sound feedback for pacing and transitions.
- **Language Support**: Include localization for non-English languages.
- **Word Splitting**: Break long words intelligently if they exceed the screen width.
- **Text Search**: Allow users to search for specific words or phrases.
- **Bookmarking**: Save progress and resume later from the last-read position.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request to suggest or implement new features.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/) and the [ratatui](https://crates.io/crates/ratatui) library.
- Inspired by traditional RSVP reading applications.
