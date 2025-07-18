use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
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
use std::{
    error::Error,
    fs::{self, DirEntry, Metadata},
    io,
    path::PathBuf,
    time::SystemTime,
};
use syntect::{easy::HighlightLines, highlighting::ThemeSet, parsing::SyntaxSet};

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
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

impl App {
    fn new(path: PathBuf, show_hidden: bool, human_readable: bool) -> io::Result<Self> {
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
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
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

    fn enter_directory(&mut self) -> io::Result<()> {
        if let Some(selected_file) = self.files.get(self.selected_index) {
            if selected_file.is_dir {
                self.current_path = selected_file.path.clone();
                self.load_directory()?;
            } else {
                // Try to open as text file
                self.open_file()?;
            }
        }
        Ok(())
    }

    fn toggle_hidden(&mut self) -> io::Result<()> {
        self.show_hidden = !self.show_hidden;
        self.load_directory()
    }

    fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    fn open_file(&mut self) -> io::Result<()> {
        if let Some(selected_file) = self.files.get(self.selected_index) {
            if !selected_file.is_dir && self.is_text_file(selected_file) {
                match fs::read_to_string(&selected_file.path) {
                    Ok(content) => {
                        self.file_content = content;
                        self.show_file_content = true;
                        self.file_content_scroll = 0;
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
        self.show_file_content = false;
        self.file_content.clear();
        self.file_content_scroll = 0;
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

    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // File list
            Constraint::Length(3), // Footer
        ])
        .split(size);

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
            horizontal: 0,
        }),
        &mut app.scroll_state,
    );

    // Footer
    let footer_text = if app.show_help {
        "Help: ↑↓/jk=Navigate  Enter=Open  a=Toggle hidden  h=Help  q/Esc=Quit"
    } else {
        "Press 'h' for help  |  ↑↓ Navigate  Enter Open  q Quit"
    };
    let footer = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Gray));
    f.render_widget(footer, chunks[2]);

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
            Line::from("  q/Esc   - Quit or close popup"),
            Line::from(""),
            Line::from("File viewing:"),
            Line::from("  Text files will open with syntax highlighting"),
            Line::from("  Use ↑↓ to scroll in file view"),
            Line::from("  Press Esc to close file view"),
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

        let highlighted_lines = render_highlighted_content(app);
        let content = Paragraph::new(highlighted_lines)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .wrap(Wrap { trim: false });

        f.render_widget(content, popup_area);

        // Show scroll indicator
        let total_lines = app.file_content.lines().count();
        if total_lines > 20 {
            let scroll_info = format!(
                "Lines {}-{} of {} (↑↓ to scroll, Esc to close)",
                app.file_content_scroll + 1,
                (app.file_content_scroll + 20).min(total_lines),
                total_lines
            );
            let info_area = ratatui::layout::Rect {
                x: popup_area.x + 2,
                y: popup_area.y + popup_area.height - 2,
                width: popup_area.width - 4,
                height: 1,
            };
            f.render_widget(
                Paragraph::new(scroll_info).style(Style::default().fg(Color::Gray)),
                info_area,
            );
        }
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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    if app.show_file_content {
                        app.close_file();
                    } else if app.show_help {
                        app.toggle_help();
                    } else {
                        return Ok(());
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if app.show_file_content {
                        app.scroll_file_up();
                    } else if !app.show_help {
                        app.navigate_up();
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if app.show_file_content {
                        app.scroll_file_down();
                    } else if !app.show_help {
                        app.navigate_down();
                    }
                }
                KeyCode::Enter => {
                    if !app.show_help && !app.show_file_content {
                        app.enter_directory()?;
                    }
                }
                KeyCode::Char('a') => {
                    if !app.show_help && !app.show_file_content {
                        app.toggle_hidden()?;
                    }
                }
                KeyCode::Char('h') => {
                    if !app.show_file_content {
                        app.toggle_help();
                    }
                }
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Ok(());
                }
                _ => {}
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

fn main() -> Result<(), Box<dyn Error>> {
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
