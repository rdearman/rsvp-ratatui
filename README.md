
# RSVP Program

Welcome to **RSVP** (Rapid Serial Visual Presentation)! This program allows users to read text files by displaying words or chunks of words one at a time in the terminal at a user-defined speed. Designed for both programmers and non-programmers, RSVP provides an efficient way to speed-read and manage text content.

---

## 📋 Current Features

- **Word-by-Word Display**: Displays words sequentially in the terminal.
- **Customisable Speed**: Adjust the speed (words per minute) in real time using keyboard shortcuts.
- **Chunk Size Control**: Choose how many words to display at a time.
- **Progress Tracking**: Shows the number of words read and the total word count.
- **Persistent Preferences**: Saves speed and chunk size settings for future sessions.
- **Intuitive Controls**: Use keyboard shortcuts for seamless navigation:
  - `[Q]` to quit
  - `[Space]` to pause or resume
  - `[↑]`/`[↓]` to increase or decrease speed by 10 WPM
  - `[PgUp]`/`[PgDn]` to adjust speed by 100 WPM
  - `[1-9]` to set chunk size
  - `[←]`/`[→]` to skip backward or forward by 5 words

---

## 🚀 How to Get Started

### For Non-Programmers

1. **Download and Install:**
   - Install a Rust toolchain by visiting [Rust's official site](https://rust-lang.org/).
   - Follow the instructions to install `rustup` and `cargo`.

2. **Download the Program:**
   - Visit the [GitHub repository](https://github.com/your_repo_link).
   - Click "Code" > "Download ZIP" and extract it to a folder on your computer.

3. **Build and Run:**
   - Open a terminal in the folder where the program is located.
   - Run the following commands:
     ```bash
     cargo build --release
     ./target/release/rsvp --help
     ```

4. **Load a File:**
   - Save your text file to a location on your computer.
   - Use the following command to start reading:
     ```bash
     ./target/release/rsvp -i path/to/your/file.txt
     ```

### For Programmers

1. **Clone the Repository:**
   ```bash
   git clone https://github.com/your_repo_link.git
   cd rsvp
   ```

2. **Build and Run:**
   ```bash
   cargo build --release
   ./target/release/rsvp -i path/to/your/file.txt
   ```

3. **Customisation:**
   - Modify the source code as needed.
   - Build the project with `cargo build` to apply changes.

---

## 📜 Planned Features

- **Save Reading Progress:** Automatically save your position in the file and allow resuming from where you left off.
- **Improved File Management:** Add a file selector interface for easier file loading.
- **Enhanced Visualisation:** Display formatted text or highlight keywords for better readability.
- **Mobile/GUI Version:** Expand to a graphical interface or mobile app.

---

## 🛠 Technical Details

- **Language:** Rust
- **Dependencies:**
  - `clap` for command-line argument parsing
  - `ratatui` for terminal-based UI
  - `crossterm` for terminal event handling
- **Persistence:** Saves preferences to a hidden file in the user's home directory (`~/.rsvp_settings`).

---

## 🤝 Contributions

This project is primarily for personal use and learning, but feedback and suggestions are welcome. Feel free to fork the repository or submit issues via GitHub.

---

Enjoy rapid reading with RSVP!
