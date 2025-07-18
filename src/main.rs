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
        ScrollbarOrientation, ScrollbarState,
    },
};
use std::{
    error::Error,
    fs::{self, DirEntry, Metadata},
    io,
    path::PathBuf,
    time::SystemTime,
};

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
        } else {
            match self.path.extension().and_then(|ext| ext.to_str()) {
                Some("rs") => "🦀",
                Some("py") => "🐍",
                Some("js") | Some("ts") => "⚡",
                Some("html") => "🌐",
                Some("css") => "🎨",
                Some("md") => "📝",
                Some("json") => "🔧",
                Some("toml") | Some("yaml") | Some("yml") => "⚙️",
                Some("png") | Some("jpg") | Some("jpeg") | Some("gif") | Some("svg") => "🖼️",
                Some("mp3") | Some("wav") | Some("flac") => "🎵",
                Some("mp4") | Some("avi") | Some("mov") => "🎬",
                Some("zip") | Some("tar") | Some("gz") | Some("rar") => "📦",
                Some("exe") | Some("app") => "⚙️",
                _ => "📄",
            }
        }
    }

    fn format_size(size: u64, human_readable: bool) -> String {
        if human_readable {
            const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
            let mut size = size as f64;
            let mut unit_index = 0;

            while size >= 1024.0 && unit_index < UNITS.len() - 1 {
                size /= 1024.0;
                unit_index += 1;
            }

            if unit_index == 0 {
                format!("{:.0} {}", size, UNITS[unit_index])
            } else {
                format!("{:.1} {}", size, UNITS[unit_index])
            }
        } else {
            size.to_string()
        }
    }

    fn format_date(&self) -> String {
        match self.modified.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(duration) => {
                let datetime = chrono::DateTime::from_timestamp(duration.as_secs() as i64, 0)
                    .unwrap_or_default();
                datetime.format("%Y-%m-%d %H:%M").to_string()
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
}

impl App {
    fn new(path: PathBuf, show_hidden: bool, human_readable: bool) -> io::Result<Self> {
        let mut app = App {
            files: Vec::new(),
            current_path: path,
            selected_index: 0,
            list_state: ListState::default(),
            scroll_state: ScrollbarState::default(),
            show_hidden,
            human_readable,
            show_help: false,
        };
        app.load_directory()?;
        app.list_state.select(Some(0));
        Ok(app)
    }

    fn load_directory(&mut self) -> io::Result<()> {
        self.files.clear();

        // Add parent directory entry if not at root
        if let Some(parent) = self.current_path.parent() {
            self.files.push(FileItem {
                name: "..".to_string(),
                path: parent.to_path_buf(),
                is_dir: true,
                size: 0,
                modified: SystemTime::UNIX_EPOCH,
                permissions: "drwxr-xr-x".to_string(),
                is_hidden: false,
            });
        }

        let entries = fs::read_dir(&self.current_path)?;
        let mut items: Vec<FileItem> = entries
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| FileItem::from_dir_entry(entry).ok())
            .filter(|item| self.show_hidden || !item.is_hidden)
            .collect();

        // Sort: directories first, then by name
        items.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });

        self.files.extend(items);
        self.selected_index = 0;
        self.list_state.select(Some(0));
        self.scroll_state = ScrollbarState::default().content_length(self.files.len());

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
    let header_text = format!("📁 ls-pretty - {}", app.current_path.display());
    let header = Paragraph::new(header_text)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL).title("Directory"));
    f.render_widget(header, chunks[0]);

    // File list
    let items: Vec<ListItem> = app
        .files
        .iter()
        .map(|file| {
            let icon = file.get_icon();
            let name_style = if file.is_dir {
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD)
            } else if file.is_hidden {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            };

            let size_str = FileItem::format_size(file.size, app.human_readable);
            let date_str = file.format_date();

            let line = Line::from(vec![
                Span::raw(format!("{} ", icon)),
                Span::styled(&file.name, name_style),
                Span::raw(format!(
                    "{:>width$} {} {}",
                    size_str,
                    file.permissions,
                    date_str,
                    width = 10
                )),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Files"))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, chunks[1], &mut app.list_state);

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
    let footer_text = if app.show_hidden {
        "q: quit | ↑/↓: navigate | Enter: open | a: toggle hidden | h: help | Hidden files: ON"
    } else {
        "q: quit | ↑/↓: navigate | Enter: open | a: toggle hidden | h: help | Hidden files: OFF"
    };

    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Controls"));
    f.render_widget(footer, chunks[2]);

    // Help overlay
    if app.show_help {
        let help_text = vec![
            Line::from("🔧 ls-pretty Help"),
            Line::from(""),
            Line::from("Navigation:"),
            Line::from("  ↑/k    - Move up"),
            Line::from("  ↓/j    - Move down"),
            Line::from("  Enter  - Open directory/file"),
            Line::from("  ..     - Go to parent directory"),
            Line::from(""),
            Line::from("Options:"),
            Line::from("  a      - Toggle hidden files"),
            Line::from("  h      - Toggle this help"),
            Line::from("  q/Esc  - Quit"),
            Line::from(""),
            Line::from("Features:"),
            Line::from("  • File type icons"),
            Line::from("  • Size in human readable format"),
            Line::from("  • Permissions display"),
            Line::from("  • Modification dates"),
            Line::from("  • Hidden file filtering"),
            Line::from(""),
            Line::from("Press 'h' or 'Esc' to close this help"),
        ];

        let help_paragraph = Paragraph::new(help_text)
            .block(
                Block::default()
                    .title("Help")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::Yellow)),
            )
            .style(Style::default().fg(Color::White));

        let popup_area = centered_rect(60, 70, size);
        f.render_widget(Clear, popup_area);
        f.render_widget(help_paragraph, popup_area);
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
                    if app.show_help {
                        app.toggle_help();
                    } else {
                        return Ok(());
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if !app.show_help {
                        app.navigate_up();
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if !app.show_help {
                        app.navigate_down();
                    }
                }
                KeyCode::Enter => {
                    if !app.show_help {
                        app.enter_directory()?;
                    }
                }
                KeyCode::Char('a') => {
                    if !app.show_help {
                        app.toggle_hidden()?;
                    }
                }
                KeyCode::Char('h') => {
                    app.toggle_help();
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
