use anyhow::Result as AppResult;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, poll},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use portable_pty::{CommandBuilder, MasterPty, PtySize};
use ratatui::{
    Frame, Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, List, ListItem, ListState, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Wrap,
    },
};
use std::io::{Read, Write};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::{
    fs::{self, DirEntry, Metadata},
    io,
    path::PathBuf,
    time::SystemTime,
};
use syntect::{easy::HighlightLines, highlighting::ThemeSet, parsing::SyntaxSet};

#[derive(Debug, Clone, Copy)]
enum CursorDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Parser)]
#[command(name = "ls-pretty")]
#[command(about = "A beautiful TUI file browser")]
struct Args {
    /// Directory to browse
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Show hidden files
    #[arg(short = 'a', long)]
    all: bool,

    /// Show file sizes in human readable format
    #[arg(short = 'H', long)]
    human_readable: bool,

    /// Simple list mode (no TUI)
    #[arg(short = 'l', long)]
    list: bool,
}

#[derive(Clone)]
struct FileItem {
    name: String,
    path: PathBuf,
    is_dir: bool,
    size: u64,
    modified: SystemTime,
    permissions: String,
    is_hidden: bool,
}

impl FileItem {
    fn from_dir_entry(entry: DirEntry) -> io::Result<Self> {
        let metadata = entry.metadata()?;
        let name = entry.file_name().to_string_lossy().to_string();
        let is_hidden = name.starts_with('.');

        Ok(FileItem {
            name: name.clone(),
            path: entry.path(),
            is_dir: metadata.is_dir(),
            size: metadata.len(),
            modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            permissions: format_permissions(&metadata),
            is_hidden,
        })
    }

    fn get_icon(&self) -> &'static str {
        if self.is_dir {
            "📁"
        } else if let Some(ext) = self.path.extension() {
            match ext.to_str().unwrap_or("").to_lowercase().as_str() {
                "rs" => "🦀",
                "py" => "🐍",
                "js" | "ts" => "📜",
                "html" => "🌐",
                "css" => "🎨",
                "json" => "📄",
                "md" => "📝",
                "txt" => "📃",
                "png" | "jpg" | "jpeg" | "gif" => "🖼️",
                "mp3" | "wav" | "flac" => "🎵",
                "mp4" | "avi" | "mkv" => "🎬",
                _ => "📄",
            }
        } else {
            "📄"
        }
    }

    fn format_size(size: u64, human_readable: bool) -> String {
        if human_readable {
            const UNITS: &[&str] = &["B", "K", "M", "G", "T"];
            let mut size = size as f64;
            let mut unit_index = 0;

            while size >= 1024.0 && unit_index < UNITS.len() - 1 {
                size /= 1024.0;
                unit_index += 1;
            }

            if unit_index == 0 {
                format!("{:.0}{}", size, UNITS[unit_index])
            } else {
                format!("{:.1}{}", size, UNITS[unit_index])
            }
        } else {
            size.to_string()
        }
    }

    fn format_date(&self) -> String {
        match self.modified.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(duration) => {
                let timestamp = duration.as_secs();
                chrono::DateTime::from_timestamp(timestamp as i64, 0)
                    .unwrap_or_default()
                    .format("%Y-%m-%d %H:%M")
                    .to_string()
            }
            Err(_) => "Unknown".to_string(),
        }
    }
}

struct App {
    files: Vec<FileItem>,
    current_path: PathBuf,
    selected_index: usize,
    list_state: ListState,
    scroll_state: ScrollbarState,
    show_hidden: bool,
    human_readable: bool,
    show_help: bool,
    show_file_content: bool,
    file_content: String,
    file_content_scroll: usize,
    file_editing_mode: bool,
    file_has_unsaved_changes: bool,
    original_file_content: String,
    show_unsaved_alert: bool,
    cursor_position: usize,
    cursor_line: usize,
    cursor_col: usize,
    cursor_blink_state: bool,
    cursor_blink_timer: std::time::Instant,
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    show_terminal: bool,
    terminal_output: Arc<Mutex<String>>,
    terminal_input: String,
    terminal_pty: Option<Box<dyn MasterPty + Send>>,
    terminal_receiver: Option<std::sync::mpsc::Receiver<String>>,
}

impl App {
    fn new(path: PathBuf, show_hidden: bool, human_readable: bool) -> AppResult<Self> {
        let mut app = Self {
            files: Vec::new(),
            current_path: path,
            selected_index: 0,
            list_state: ListState::default(),
            scroll_state: ScrollbarState::new(0),
            show_hidden,
            human_readable,
            show_help: false,
            show_file_content: false,
            file_content: String::new(),
            file_content_scroll: 0,
            file_editing_mode: false,
            file_has_unsaved_changes: false,
            original_file_content: String::new(),
            show_unsaved_alert: false,
            cursor_position: 0,
            cursor_line: 0,
            cursor_col: 0,
            cursor_blink_state: true,
            cursor_blink_timer: std::time::Instant::now(),
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            show_terminal: false,
            terminal_output: Arc::new(Mutex::new(String::new())),
            terminal_input: String::new(),
            terminal_pty: None,
            terminal_receiver: None,
        };
        app.load_directory()?;
        app.list_state.select(Some(0));
        Ok(app)
    }

    fn load_directory(&mut self) -> io::Result<()> {
        self.files.clear();
        self.selected_index = 0;

        let entries = fs::read_dir(&self.current_path)?;
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(file_item) = FileItem::from_dir_entry(entry) {
                    if self.show_hidden || !file_item.is_hidden {
                        self.files.push(file_item);
                    }
                }
            }
        }

        // Sort: directories first, then files, both alphabetically
        self.files.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });

        // Add parent directory entry if not at root
        if let Some(parent) = self.current_path.parent() {
            let parent_item = FileItem {
                name: "..".to_string(),
                path: parent.to_path_buf(),
                is_dir: true,
                size: 0,
                modified: SystemTime::UNIX_EPOCH,
                permissions: "drwxrwxrwx".to_string(),
                is_hidden: false,
            };
            self.files.insert(0, parent_item);
        }

        // Update scroll state
        self.scroll_state = self.scroll_state.content_length(self.files.len());
        self.list_state.select(Some(0));

        Ok(())
    }

    fn navigate_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.list_state.select(Some(self.selected_index));
            self.scroll_state = self.scroll_state.position(self.selected_index);
        }
    }

    fn navigate_down(&mut self) {
        if self.selected_index < self.files.len().saturating_sub(1) {
            self.selected_index += 1;
            self.list_state.select(Some(self.selected_index));
            self.scroll_state = self.scroll_state.position(self.selected_index);
        }
    }

    fn enter_directory(&mut self) -> AppResult<()> {
        if let Some(selected_file) = self.files.get(self.selected_index) {
            if selected_file.is_dir {
                self.current_path = selected_file.path.clone();
                self.load_directory()?;
            } else {
                // Try to open as text file
                self.open_file().map_err(anyhow::Error::from)?;
            }
        }
        Ok(())
    }

    fn toggle_hidden(&mut self) -> AppResult<()> {
        self.show_hidden = !self.show_hidden;
        self.load_directory().map_err(anyhow::Error::from)
    }

    fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    fn open_file(&mut self) -> io::Result<()> {
        if let Some(selected_file) = self.files.get(self.selected_index) {
            if !selected_file.is_dir && self.is_text_file(selected_file) {
                match fs::read_to_string(&selected_file.path) {
                    Ok(content) => {
                        self.file_content = content.clone();
                        self.original_file_content = content;
                        self.show_file_content = true;
                        self.file_content_scroll = 0;
                        self.file_editing_mode = false;
                        self.file_has_unsaved_changes = false;
                        self.update_cursor_position();
                    }
                    Err(_) => {
                        // If file can't be read as text, do nothing
                    }
                }
            }
        }
        Ok(())
    }

    fn close_file(&mut self) {
        if self.file_has_unsaved_changes {
            self.show_unsaved_alert = true;
        } else {
            self.actually_close_file();
        }
    }

    fn actually_close_file(&mut self) {
        self.show_file_content = false;
        self.file_content.clear();
        self.file_content_scroll = 0;
        self.file_editing_mode = false;
        self.file_has_unsaved_changes = false;
        self.original_file_content.clear();
        self.show_unsaved_alert = false;
        self.cursor_position = 0;
        self.cursor_line = 0;
        self.cursor_col = 0;
        self.cursor_blink_state = true;
        self.cursor_blink_timer = std::time::Instant::now();
    }

    fn toggle_edit_mode(&mut self) {
        self.file_editing_mode = !self.file_editing_mode;
    }

    fn save_file(&mut self) -> AppResult<()> {
        if let Some(selected_file) = self.files.get(self.selected_index) {
            if !selected_file.is_dir && self.file_has_unsaved_changes {
                fs::write(&selected_file.path, &self.file_content)?;
                self.original_file_content = self.file_content.clone();
                self.file_has_unsaved_changes = false;
            }
        }
        Ok(())
    }

    fn handle_file_edit(&mut self, ch: char) {
        let chars: Vec<char> = self.file_content.chars().collect();
        let mut new_chars = chars.clone();

        match ch {
            '\n' => {
                new_chars.insert(self.cursor_position, '\n');
                self.cursor_position += 1;
                self.cursor_line += 1;
                self.cursor_col = 0;
            }
            '\u{8}' | '\u{7f}' => {
                // Backspace
                if self.cursor_position > 0 {
                    new_chars.remove(self.cursor_position - 1);
                    self.cursor_position -= 1;
                    if self.cursor_col > 0 {
                        self.cursor_col -= 1;
                    } else if self.cursor_line > 0 {
                        self.cursor_line -= 1;
                        // Find the length of the previous line
                        let lines: Vec<&str> = self.file_content.lines().collect();
                        if self.cursor_line < lines.len() {
                            self.cursor_col = lines[self.cursor_line].len();
                        }
                    }
                }
            }
            c if c.is_control() => {
                // Ignore other control characters
            }
            _ => {
                new_chars.insert(self.cursor_position, ch);
                self.cursor_position += 1;
                self.cursor_col += 1;
            }
        }

        self.file_content = new_chars.into_iter().collect();
        self.file_has_unsaved_changes = self.file_content != self.original_file_content;

        // Auto-scroll to keep cursor visible
        let visible_lines = 30; // Show more lines
        if self.cursor_line >= self.file_content_scroll + visible_lines {
            self.file_content_scroll = self.cursor_line.saturating_sub(visible_lines - 1);
        } else if self.cursor_line < self.file_content_scroll {
            self.file_content_scroll = self.cursor_line;
        }
    }

    fn update_cursor_position(&mut self) {
        self.cursor_position = 0;
        self.cursor_line = 0;
        self.cursor_col = 0;
        self.cursor_blink_state = true;
        self.cursor_blink_timer = std::time::Instant::now();
    }

    fn update_cursor_blink(&mut self) {
        if self.cursor_blink_timer.elapsed().as_millis() > 500 {
            self.cursor_blink_state = !self.cursor_blink_state;
            self.cursor_blink_timer = std::time::Instant::now();
        }
    }

    fn handle_cursor_movement(&mut self, direction: CursorDirection) {
        let lines: Vec<&str> = self.file_content.lines().collect();

        match direction {
            CursorDirection::Up => {
                if self.cursor_line > 0 {
                    self.cursor_line -= 1;
                    let line_len = if self.cursor_line < lines.len() {
                        lines[self.cursor_line].len()
                    } else {
                        0
                    };
                    self.cursor_col = self.cursor_col.min(line_len);
                    self.recalculate_cursor_position();
                }
            }
            CursorDirection::Down => {
                if self.cursor_line < lines.len().saturating_sub(1) {
                    self.cursor_line += 1;
                    let line_len = if self.cursor_line < lines.len() {
                        lines[self.cursor_line].len()
                    } else {
                        0
                    };
                    self.cursor_col = self.cursor_col.min(line_len);
                    self.recalculate_cursor_position();
                }
            }
            CursorDirection::Left => {
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                    self.cursor_position -= 1;
                } else if self.cursor_line > 0 {
                    self.cursor_line -= 1;
                    self.cursor_col = if self.cursor_line < lines.len() {
                        lines[self.cursor_line].len()
                    } else {
                        0
                    };
                    self.cursor_position -= 1;
                }
            }
            CursorDirection::Right => {
                let current_line_len = if self.cursor_line < lines.len() {
                    lines[self.cursor_line].len()
                } else {
                    0
                };

                if self.cursor_col < current_line_len {
                    self.cursor_col += 1;
                    self.cursor_position += 1;
                } else if self.cursor_line < lines.len().saturating_sub(1) {
                    self.cursor_line += 1;
                    self.cursor_col = 0;
                    self.cursor_position += 1;
                }
            }
        }

        // Auto-scroll to keep cursor visible
        let visible_lines = 30;
        if self.cursor_line >= self.file_content_scroll + visible_lines {
            self.file_content_scroll = self.cursor_line.saturating_sub(visible_lines - 1);
        } else if self.cursor_line < self.file_content_scroll {
            self.file_content_scroll = self.cursor_line;
        }
    }

    fn recalculate_cursor_position(&mut self) {
        let lines: Vec<&str> = self.file_content.lines().collect();
        let mut pos = 0;

        for i in 0..self.cursor_line.min(lines.len()) {
            pos += lines[i].len() + 1; // +1 for newline
        }
        pos += self.cursor_col;

        self.cursor_position = pos.min(self.file_content.len());
    }

    fn discard_changes(&mut self) {
        self.file_content = self.original_file_content.clone();
        self.file_has_unsaved_changes = false;
        self.show_unsaved_alert = false;
        self.actually_close_file();
    }

    fn scroll_file_up(&mut self) {
        if self.file_content_scroll > 0 {
            self.file_content_scroll -= 1;
        }
    }

    fn scroll_file_down(&mut self) {
        let lines_count = self.file_content.lines().count();
        if self.file_content_scroll < lines_count.saturating_sub(1) {
            self.file_content_scroll += 1;
        }
    }

    fn is_text_file(&self, file: &FileItem) -> bool {
        if file.is_dir {
            return false;
        }

        if let Some(ext) = file.path.extension() {
            if let Some(ext_str) = ext.to_str() {
                matches!(
                    ext_str.to_lowercase().as_str(),
                    "txt"
                        | "md"
                        | "rs"
                        | "py"
                        | "js"
                        | "ts"
                        | "html"
                        | "css"
                        | "json"
                        | "xml"
                        | "yaml"
                        | "yml"
                        | "toml"
                        | "cfg"
                        | "conf"
                        | "log"
                        | "sh"
                        | "bash"
                        | "zsh"
                        | "fish"
                        | "c"
                        | "cpp"
                        | "h"
                        | "hpp"
                        | "java"
                        | "go"
                        | "php"
                        | "rb"
                        | "pl"
                        | "lua"
                        | "vim"
                        | "sql"
                        | "csv"
                )
            } else {
                false
            }
        } else {
            // Check if filename suggests it's a text file
            let name = file.name.to_lowercase();
            matches!(
                name.as_str(),
                "readme"
                    | "license"
                    | "changelog"
                    | "makefile"
                    | "dockerfile"
                    | "gitignore"
                    | "gitattributes"
                    | "editorconfig"
            )
        }
    }

    fn toggle_terminal(&mut self) -> AppResult<()> {
        if self.show_terminal {
            // Close terminal
            self.show_terminal = false;
            self.terminal_pty = None;
            self.terminal_receiver = None;
            if let Ok(mut output) = self.terminal_output.lock() {
                output.clear();
            }
            self.terminal_input.clear();
        } else {
            // Open terminal
            self.open_terminal()?;
        }
        Ok(())
    }

    fn open_terminal(&mut self) -> AppResult<()> {
        // Try to create pseudo-terminal, but don't fail the whole app if it doesn't work
        match self.try_create_pty() {
            Ok(_) => {
                self.show_terminal = true;
                if let Ok(mut output) = self.terminal_output.lock() {
                    output.push_str("Terminal initialized successfully.\n");
                    output.push_str(&format!(
                        "Working directory: {}\n",
                        self.current_path.display()
                    ));
                }
            }
            Err(e) => {
                // Fallback to simple command execution
                if let Ok(mut output) = self.terminal_output.lock() {
                    output.push_str("Failed to create pseudo-terminal, using fallback mode.\n");
                    output.push_str(&format!("Error: {}\n", e));
                    output.push_str("You can still navigate files normally.\n");
                }
                self.show_terminal = true;
            }
        }
        Ok(())
    }

    fn try_create_pty(&mut self) -> AppResult<()> {
        let pty_system = portable_pty::native_pty_system();
        let pty_size = PtySize {
            rows: 8,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        };

        // Determine shell command
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        let mut cmd = CommandBuilder::new(&shell);
        cmd.cwd(&self.current_path);

        let pty_pair = pty_system.openpty(pty_size)?;
        let _child = pty_pair.slave.spawn_command(cmd)?;

        // Setup reader thread
        let mut reader = pty_pair.master.try_clone_reader()?;
        let terminal_output = Arc::clone(&self.terminal_output);
        let (sender, receiver) = mpsc::channel();

        thread::spawn(move || {
            let mut buffer = [0u8; 1024];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        let text = String::from_utf8_lossy(&buffer[..n]);
                        if let Ok(mut output) = terminal_output.lock() {
                            output.push_str(&text);
                            // Keep only last 100 lines to prevent memory issues
                            let lines: Vec<&str> = output.lines().collect();
                            if lines.len() > 100 {
                                *output = lines[lines.len() - 100..].join("\n");
                            }
                        }
                        let _ = sender.send(text.to_string());
                    }
                    Err(_) => break,
                }
            }
        });

        self.terminal_pty = Some(pty_pair.master);
        self.terminal_receiver = Some(receiver);

        Ok(())
    }

    fn send_to_terminal(&mut self, input: &str) -> AppResult<()> {
        if let Some(ref pty) = self.terminal_pty {
            match pty.take_writer() {
                Ok(mut writer) => {
                    let _ = writer.write_all(input.as_bytes());
                    let _ = writer.flush();
                }
                Err(_) => {
                    // Fallback: just echo the input to the output
                    if let Ok(mut output) = self.terminal_output.lock() {
                        output.push_str(input);
                    }
                }
            }
        } else {
            // No PTY available, just echo to output
            if let Ok(mut output) = self.terminal_output.lock() {
                output.push_str("(no terminal) ");
                output.push_str(input);
            }
        }
        Ok(())
    }

    fn handle_terminal_input(&mut self, ch: char) -> AppResult<()> {
        match ch {
            '\r' | '\n' => {
                // Send the current input plus newline to terminal
                let input = format!("{}\n", self.terminal_input);
                let _ = self.send_to_terminal(&input);
                self.terminal_input.clear();
            }
            '\u{8}' | '\u{7f}' => {
                // Backspace
                if !self.terminal_input.is_empty() {
                    self.terminal_input.pop();
                    let _ = self.send_to_terminal("\u{8} \u{8}"); // Backspace, space, backspace
                }
            }
            _ => {
                self.terminal_input.push(ch);
                let _ = self.send_to_terminal(&ch.to_string());
            }
        }
        Ok(())
    }
}

fn format_permissions(metadata: &Metadata) -> String {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = metadata.permissions().mode();
        let mut perms = String::new();

        // File type
        perms.push(if metadata.is_dir() { 'd' } else { '-' });

        // Owner permissions
        perms.push(if mode & 0o400 != 0 { 'r' } else { '-' });
        perms.push(if mode & 0o200 != 0 { 'w' } else { '-' });
        perms.push(if mode & 0o100 != 0 { 'x' } else { '-' });

        // Group permissions
        perms.push(if mode & 0o040 != 0 { 'r' } else { '-' });
        perms.push(if mode & 0o020 != 0 { 'w' } else { '-' });
        perms.push(if mode & 0o010 != 0 { 'x' } else { '-' });

        // Others permissions
        perms.push(if mode & 0o004 != 0 { 'r' } else { '-' });
        perms.push(if mode & 0o002 != 0 { 'w' } else { '-' });
        perms.push(if mode & 0o001 != 0 { 'x' } else { '-' });

        perms
    }

    #[cfg(not(unix))]
    {
        if metadata.permissions().readonly() {
            "r--r--r--".to_string()
        } else {
            "rw-rw-rw-".to_string()
        }
    }
}

fn render_highlighted_content(app: &App) -> Vec<Line> {
    if app.file_content.is_empty() {
        return vec![Line::from("File is empty or could not be read")];
    }

    let selected_file = &app.files[app.selected_index];
    let syntax = app
        .syntax_set
        .find_syntax_for_file(&selected_file.path)
        .ok()
        .flatten()
        .unwrap_or_else(|| app.syntax_set.find_syntax_plain_text());

    let theme = &app.theme_set.themes["base16-ocean.dark"];
    let mut highlighter = HighlightLines::new(syntax, theme);

    let mut lines = Vec::new();
    let content_lines: Vec<&str> = app.file_content.lines().collect();

    // Apply scrolling - show up to 20 lines at a time
    let visible_lines = content_lines.iter().skip(app.file_content_scroll).take(20);

    for line in visible_lines {
        match highlighter.highlight_line(line, &app.syntax_set) {
            Ok(highlighted) => {
                let mut spans = Vec::new();
                for (style, text) in highlighted {
                    let fg_color = style.foreground;
                    let color = Color::Rgb(fg_color.r, fg_color.g, fg_color.b);
                    let mut modifier = Modifier::empty();
                    if style
                        .font_style
                        .contains(syntect::highlighting::FontStyle::BOLD)
                    {
                        modifier |= Modifier::BOLD;
                    }
                    if style
                        .font_style
                        .contains(syntect::highlighting::FontStyle::ITALIC)
                    {
                        modifier |= Modifier::ITALIC;
                    }
                    if style
                        .font_style
                        .contains(syntect::highlighting::FontStyle::UNDERLINE)
                    {
                        modifier |= Modifier::UNDERLINED;
                    }
                    spans.push(Span::styled(
                        text,
                        Style::default().fg(color).add_modifier(modifier),
                    ));
                }
                lines.push(Line::from(spans));
            }
            Err(_) => {
                lines.push(Line::from(*line));
            }
        }
    }

    if lines.is_empty() {
        lines.push(Line::from("No content to display"));
    }

    lines
}

fn ui(f: &mut Frame, app: &mut App) {
    let size = f.size();

    // Create main layout - adjust based on terminal visibility
    let chunks = if app.show_terminal {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Min(0),     // File list
                Constraint::Length(12), // Terminal
                Constraint::Length(3),  // Footer
            ])
            .split(size)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // File list
                Constraint::Length(3), // Footer
            ])
            .split(size)
    };

    // Header
    let header = Paragraph::new(format!("📁 {}", app.current_path.display()))
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(header, chunks[0]);

    // File list
    let items: Vec<ListItem> = app
        .files
        .iter()
        .map(|file| {
            let icon = file.get_icon();
            let size_str = FileItem::format_size(file.size, app.human_readable);
            let date_str = file.format_date();

            let style = if file.is_dir {
                Style::default().fg(Color::Blue)
            } else if app.is_text_file(file) {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::White)
            };

            let content = format!(
                "{} {:30} {:>10} {} {}",
                icon, file.name, size_str, file.permissions, date_str
            );
            ListItem::new(content).style(style)
        })
        .collect();

    let files_list = List::new(items)
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black))
        .highlight_symbol("➤ ");

    f.render_stateful_widget(files_list, chunks[1], &mut app.list_state);

    // Scrollbar
    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));
    f.render_stateful_widget(
        scrollbar,
        chunks[1].inner(&Margin {
            vertical: 1,
            horizontal: 1,
        }),
        &mut app.scroll_state,
    );

    // Terminal (if enabled, show in its own section)
    if app.show_terminal {
        // Get terminal output
        let terminal_content = if let Ok(output) = app.terminal_output.lock() {
            output.clone()
        } else {
            "Terminal output unavailable".to_string()
        };

        // Show last 8 lines for bottom terminal
        let lines: Vec<&str> = terminal_content.lines().collect();
        let visible_lines = if lines.len() > 8 {
            &lines[lines.len() - 8..]
        } else {
            &lines
        };

        let mut terminal_lines: Vec<Line> =
            visible_lines.iter().map(|line| Line::from(*line)).collect();

        // Add current input line
        let input_line = format!("$ {}", app.terminal_input);
        terminal_lines.push(Line::from(input_line));

        let terminal_widget = Paragraph::new(terminal_lines)
            .block(
                Block::default()
                    .title(" Terminal (Ctrl+T to close) ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green)),
            )
            .wrap(Wrap { trim: false });

        f.render_widget(terminal_widget, chunks[2]);
    }

    // Footer
    let footer_text = if app.show_help {
        "Help: ↑↓/jk=Navigate  Enter=Open  a=Toggle hidden  h=Help  Ctrl+T=Terminal  q/Esc=Quit"
    } else if app.show_terminal {
        "Terminal active - Type commands and press Enter  |  Ctrl+T to close  |  q to quit"
    } else {
        "Press 'h' for help  |  ↑↓ Navigate  Enter Open  Ctrl+T Terminal  q Quit"
    };
    let footer = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Gray));

    let footer_chunk = if app.show_terminal {
        chunks[3]
    } else {
        chunks[2]
    };
    f.render_widget(footer, footer_chunk);

    // Help popup
    if app.show_help {
        let popup_area = centered_rect(60, 50, size);
        f.render_widget(Clear, popup_area);
        let help_text = vec![
            Line::from("File Browser Help"),
            Line::from(""),
            Line::from("Navigation:"),
            Line::from("  ↑/k     - Move up"),
            Line::from("  ↓/j     - Move down"),
            Line::from("  Enter   - Enter directory or view file"),
            Line::from(""),
            Line::from("Commands:"),
            Line::from("  a       - Toggle hidden files"),
            Line::from("  h       - Toggle this help"),
            Line::from("  Ctrl+T  - Toggle integrated terminal"),
            Line::from("  q/Esc   - Quit or close popup"),
            Line::from(""),
            Line::from("File viewing and editing:"),
            Line::from("  Text files open with syntax highlighting"),
            Line::from("  Press E to toggle edit mode"),
            Line::from("  Ctrl+S to save changes"),
            Line::from("  View mode: ↑↓ to scroll"),
            Line::from("  Edit mode: ↑↓←→ to move cursor"),
            Line::from("  Edit mode: Type to insert, Backspace to delete"),
            Line::from("  Press Esc to close file view"),
            Line::from(""),
            Line::from("Terminal:"),
            Line::from("  Opens at bottom of screen"),
            Line::from("  Type commands and press Enter"),
            Line::from("  Ctrl+T to close terminal"),
        ];
        let help_popup = Paragraph::new(help_text)
            .block(
                Block::default()
                    .title(" Help ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green)),
            )
            .wrap(Wrap { trim: false });
        f.render_widget(help_popup, popup_area);
    }

    // File content popup
    if app.show_file_content {
        let popup_area = centered_rect(85, 85, size);
        f.render_widget(Clear, popup_area);

        let selected_file = &app.files[app.selected_index];
        let title = format!(" {} ", selected_file.name);

        let content = if app.file_editing_mode {
            // In editing mode, show raw text with cursor
            let content_lines: Vec<&str> = app.file_content.lines().collect();
            let visible_lines = content_lines.iter().skip(app.file_content_scroll).take(30);

            let mut lines: Vec<Line> = Vec::new();
            for (line_idx, line_text) in visible_lines.enumerate() {
                let actual_line_idx = line_idx + app.file_content_scroll;

                if actual_line_idx == app.cursor_line {
                    // This line contains the cursor
                    let mut spans = Vec::new();
                    let line_chars: Vec<char> = line_text.chars().collect();

                    for (col_idx, ch) in line_chars.iter().enumerate() {
                        if col_idx == app.cursor_col && app.cursor_blink_state {
                            // Insert cursor before this character
                            spans.push(Span::styled("█", Style::default().fg(Color::White)));
                        }
                        spans.push(Span::raw(ch.to_string()));
                    }

                    // If cursor is at end of line
                    if app.cursor_col >= line_chars.len() && app.cursor_blink_state {
                        spans.push(Span::styled("█", Style::default().fg(Color::White)));
                    }

                    lines.push(Line::from(spans));
                } else {
                    lines.push(Line::from(*line_text));
                }
            }

            let edit_title = if app.file_has_unsaved_changes {
                format!(" {} (EDITING - UNSAVED) ", selected_file.name)
            } else {
                format!(" {} (EDITING) ", selected_file.name)
            };

            Paragraph::new(lines)
                .block(
                    Block::default()
                        .title(edit_title)
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(if app.file_has_unsaved_changes {
                            Color::Red
                        } else {
                            Color::Cyan
                        })),
                )
                .wrap(Wrap { trim: false })
        } else {
            // In viewing mode, show syntax highlighted content
            let highlighted_lines = render_highlighted_content(app);
            Paragraph::new(highlighted_lines)
                .block(
                    Block::default()
                        .title(title)
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Yellow)),
                )
                .wrap(Wrap { trim: false })
        };

        f.render_widget(content, popup_area);

        // Show scroll indicator
        let total_lines = app.file_content.lines().count();
        let help_text = if app.file_editing_mode {
            if total_lines > 30 {
                format!(
                    "Lines {}-{} of {} | EDIT: Type/↑↓←→ navigate, Ctrl+S save, E view, Esc close | Cursor: {}:{}",
                    app.file_content_scroll + 1,
                    (app.file_content_scroll + 30).min(total_lines),
                    total_lines,
                    app.cursor_line + 1,
                    app.cursor_col + 1
                )
            } else {
                format!(
                    "EDIT MODE: Type/↑↓←→ navigate, Ctrl+S save, E view, Esc close | Cursor: {}:{}",
                    app.cursor_line + 1,
                    app.cursor_col + 1
                )
            }
        } else {
            if total_lines > 30 {
                format!(
                    "Lines {}-{} of {} | VIEW MODE: ↑↓ to scroll, E to edit, Esc to close",
                    app.file_content_scroll + 1,
                    (app.file_content_scroll + 30).min(total_lines),
                    total_lines
                )
            } else {
                "VIEW MODE: E to edit, Esc to close".to_string()
            }
        };

        let info_area = ratatui::layout::Rect {
            x: popup_area.x + 2,
            y: popup_area.y + popup_area.height - 2,
            width: popup_area.width - 4,
            height: 1,
        };
        f.render_widget(
            Paragraph::new(help_text).style(Style::default().fg(Color::Gray)),
            info_area,
        );
    }

    // Unsaved changes alert
    if app.show_unsaved_alert {
        let popup_area = centered_rect(50, 30, size);
        f.render_widget(Clear, popup_area);

        let alert_text = vec![
            Line::from(""),
            Line::from("You have unsaved changes!"),
            Line::from(""),
            Line::from("Press:"),
            Line::from("  S - Save and close"),
            Line::from("  D - Discard changes and close"),
            Line::from("  C - Cancel (continue editing)"),
        ];

        let alert = Paragraph::new(alert_text)
            .block(
                Block::default()
                    .title(" Unsaved Changes ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red)),
            )
            .style(Style::default().fg(Color::White));

        f.render_widget(alert, popup_area);
    }
}

fn centered_rect(
    percent_x: u16,
    percent_y: u16,
    r: ratatui::layout::Rect,
) -> ratatui::layout::Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> AppResult<()> {
    loop {
        // Update cursor blink state
        app.update_cursor_blink();

        terminal.draw(|f| ui(f, &mut app))?;

        // Use poll to check for events with timeout for cursor blinking
        if poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        if app.show_unsaved_alert {
                            app.show_unsaved_alert = false;
                        } else if app.show_terminal {
                            app.toggle_terminal()?;
                        } else if app.show_file_content {
                            app.close_file();
                        } else if app.show_help {
                            app.toggle_help();
                        } else {
                            return Ok(());
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if app.show_unsaved_alert {
                            // Don't navigate when alert is shown
                        } else if app.show_terminal {
                            // In terminal mode, don't handle up/down
                        } else if app.show_file_content && app.file_editing_mode {
                            app.handle_cursor_movement(CursorDirection::Up);
                        } else if app.show_file_content && !app.file_editing_mode {
                            app.scroll_file_up();
                        } else if !app.show_help && !app.show_file_content {
                            app.navigate_up();
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if app.show_unsaved_alert {
                            // Don't navigate when alert is shown
                        } else if app.show_terminal {
                            // In terminal mode, don't handle up/down
                        } else if app.show_file_content && app.file_editing_mode {
                            app.handle_cursor_movement(CursorDirection::Down);
                        } else if app.show_file_content && !app.file_editing_mode {
                            app.scroll_file_down();
                        } else if !app.show_help && !app.show_file_content {
                            app.navigate_down();
                        }
                    }
                    KeyCode::Enter => {
                        if app.show_unsaved_alert {
                            // Don't handle enter when alert is shown
                        } else if app.show_terminal {
                            app.handle_terminal_input('\n')?;
                        } else if app.file_editing_mode {
                            app.handle_file_edit('\n');
                        } else if !app.show_help && !app.show_file_content {
                            if app.file_has_unsaved_changes {
                                app.show_unsaved_alert = true;
                            } else {
                                app.enter_directory()?;
                            }
                        }
                    }
                    KeyCode::Left => {
                        if app.file_editing_mode && !app.show_unsaved_alert {
                            app.handle_cursor_movement(CursorDirection::Left);
                        }
                    }
                    KeyCode::Right => {
                        if app.file_editing_mode && !app.show_unsaved_alert {
                            app.handle_cursor_movement(CursorDirection::Right);
                        }
                    }
                    KeyCode::Char('a') => {
                        if app.show_unsaved_alert {
                            // Don't handle 'a' when alert is shown
                        } else if app.show_terminal {
                            app.handle_terminal_input('a')?;
                        } else if app.file_editing_mode {
                            app.handle_file_edit('a');
                        } else if !app.show_help && !app.show_file_content {
                            app.toggle_hidden()?;
                        }
                    }
                    KeyCode::Char('h') => {
                        if app.show_unsaved_alert {
                            // Don't handle 'h' when alert is shown
                        } else if app.show_terminal {
                            app.handle_terminal_input('h')?;
                        } else if app.file_editing_mode {
                            app.handle_file_edit('h');
                        } else if !app.show_file_content {
                            app.toggle_help();
                        }
                    }
                    KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        if !app.show_unsaved_alert {
                            app.toggle_terminal()?;
                        }
                    }
                    KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        if app.show_file_content && app.file_editing_mode {
                            app.save_file()?;
                        } else if app.show_unsaved_alert {
                            app.save_file()?;
                            app.actually_close_file();
                        }
                    }
                    KeyCode::Char('e') => {
                        if app.show_file_content && !app.show_unsaved_alert {
                            app.toggle_edit_mode();
                        }
                    }

                    KeyCode::Char('d') => {
                        if app.show_unsaved_alert {
                            app.discard_changes();
                        } else if app.file_editing_mode {
                            app.handle_file_edit('d');
                        }
                    }
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        if app.show_unsaved_alert {
                            // Don't quit when alert is shown
                        } else if app.show_terminal {
                            let _ = app.send_to_terminal("\u{3}"); // Send Ctrl+C to terminal
                        } else {
                            return Ok(());
                        }
                    }
                    KeyCode::Backspace => {
                        if app.show_unsaved_alert {
                            // Don't handle backspace when alert is shown
                        } else if app.show_terminal {
                            app.handle_terminal_input('\u{8}')?;
                        } else if app.file_editing_mode {
                            app.handle_file_edit('\u{8}');
                        }
                    }
                    KeyCode::Char(c) => {
                        if app.show_unsaved_alert {
                            match c {
                                's' => {
                                    app.save_file()?;
                                    app.actually_close_file();
                                }
                                'd' => {
                                    app.discard_changes();
                                }
                                'c' => {
                                    app.show_unsaved_alert = false;
                                }
                                _ => {}
                            }
                        } else if app.show_terminal {
                            app.handle_terminal_input(c)?;
                        } else if app.file_editing_mode {
                            app.handle_file_edit(c);
                        }
                        // Don't handle other characters when not in terminal or edit mode
                        // This prevents accidental exits
                    }
                    _ => {}
                }
            }
        }
    }
}

fn print_simple_list(app: &App) {
    println!("📁 Directory: {}", app.current_path.display());
    println!("{}", "─".repeat(80));

    for file in &app.files {
        let icon = file.get_icon();
        let size_str = FileItem::format_size(file.size, app.human_readable);
        let date_str = file.format_date();

        println!(
            "{} {:30} {:>10} {} {}",
            icon, file.name, size_str, file.permissions, date_str
        );
    }

    println!("{}", "─".repeat(80));
    println!("Total files: {}", app.files.len());
}

fn main() -> AppResult<()> {
    let args = Args::parse();

    // Resolve the path
    let path = if args.path.is_absolute() {
        args.path
    } else {
        std::env::current_dir()?.join(args.path)
    };

    if !path.exists() {
        eprintln!("Error: Path '{}' does not exist", path.display());
        std::process::exit(1);
    }

    if !path.is_dir() {
        eprintln!("Error: Path '{}' is not a directory", path.display());
        std::process::exit(1);
    }

    // Create app
    let app = App::new(path, args.all, args.human_readable)?;

    if args.list {
        // Simple list mode
        print_simple_list(&app);
        return Ok(());
    }

    // Setup terminal for TUI mode
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run TUI
    let res = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}
