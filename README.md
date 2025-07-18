# 📁 ls-pretty

A beautiful Terminal User Interface (TUI) file browser written in Rust, designed to make directory navigation both functional and visually appealing.

## ✨ Features

- **🎨 Beautiful TUI Interface**: Interactive file browser with keyboard navigation
- **📱 File Type Icons**: Visual icons for different file types (Rust 🦀, Python 🐍, JavaScript ⚡, etc.)
- **📏 Human-Readable Sizes**: Display file sizes in KB, MB, GB format
- **🔒 Permission Display**: Unix-style permission strings
- **📅 Modification Dates**: Last modified timestamps
- **👻 Hidden File Support**: Toggle visibility of hidden files
- **⌨️ Vim-like Navigation**: Use h/j/k/l or arrow keys
- **🔍 Multiple View Modes**: TUI mode or simple list mode

## 🚀 Installation

```bash
git clone <your-repo>
cd ls-pretty
cargo build --release
```

## 📖 Usage

### TUI Mode (Interactive)
```bash
# Navigate current directory
./target/release/ls-pretty

# Navigate specific directory
./target/release/ls-pretty /path/to/directory

# Show hidden files
./target/release/ls-pretty -a

# Human readable file sizes
./target/release/ls-pretty -H
```

### Simple List Mode
```bash
# Simple list output (non-interactive)
./target/release/ls-pretty -l

# Combine with other options
./target/release/ls-pretty -l -H -a /path/to/directory
```

## ⌨️ TUI Controls

| Key | Action |
|-----|--------|
| `↑/k` | Move selection up |
| `↓/j` | Move selection down |
| `Enter` | Open directory or file |
| `a` | Toggle hidden files |
| `h` | Show/hide help |
| `q/Esc` | Quit application |
| `Ctrl+C` | Force quit |

## 📂 File Type Icons

| Type | Icon | Extensions |
|------|------|------------|
| Directory | 📁 | - |
| Rust | 🦀 | `.rs` |
| Python | 🐍 | `.py` |
| JavaScript/TypeScript | ⚡ | `.js`, `.ts` |
| Web | 🌐 | `.html` |
| Styles | 🎨 | `.css` |
| Documentation | 📝 | `.md` |
| Config | ⚙️ | `.json`, `.toml`, `.yaml`, `.yml` |
| Images | 🖼️ | `.png`, `.jpg`, `.jpeg`, `.gif`, `.svg` |
| Audio | 🎵 | `.mp3`, `.wav`, `.flac` |
| Video | 🎬 | `.mp4`, `.avi`, `.mov` |
| Archives | 📦 | `.zip`, `.tar`, `.gz`, `.rar` |
| Executables | ⚙️ | `.exe`, `.app` |
| Default | 📄 | Other files |

## 🎯 Command Line Options

```
A beautiful TUI file browser

Usage: ls-pretty [OPTIONS] [PATH]

Arguments:
  [PATH]  Directory to browse [default: .]

Options:
  -a, --all             Show hidden files
  -H, --human-readable  Show file sizes in human readable format
  -l, --list           Simple list mode (no TUI)
  -h, --help           Print help
```

## 🛠️ Dependencies

- **ratatui**: Terminal UI library
- **crossterm**: Cross-platform terminal manipulation
- **clap**: Command line argument parsing
- **chrono**: Date and time handling
- **dirs**: Standard directory paths

## 🚧 Requirements

- Rust 1.70+ (uses 2024 edition)
- Terminal with Unicode support for best icon display
- Unix-like system for full permission display (Windows shows simplified permissions)

## 🎨 Screenshots

### TUI Mode
```
┌Directory─────────────────────────────────────────────────────────────────┐
│📁 ls-pretty - /Users/username/projects/ls-pretty                        │
└──────────────────────────────────────────────────────────────────────────┘
┌Files─────────────────────────────────────────────────────────────────────┐
│▶ 📁 ..                    0 B drwxr-xr-x 1970-01-01 00:00               │
│  📁 src                  96 B drwxr-xr-x 2025-07-18 09:42               │
│  📁 target              160 B drwxr-xr-x 2025-07-18 09:42               │
│  📄 Cargo.lock       25.0 KB -rw-r--r-- 2025-07-18 09:43               │
│  ⚙️ Cargo.toml          252 B -rw-r--r-- 2025-07-18 09:43               │
└──────────────────────────────────────────────────────────────────────────┘
┌Controls──────────────────────────────────────────────────────────────────┐
│q: quit | ↑/↓: navigate | Enter: open | a: toggle hidden | h: help       │
└──────────────────────────────────────────────────────────────────────────┘
```

### List Mode
```
📁 Directory: /Users/username/projects/ls-pretty
────────────────────────────────────────────────────────────────────────────────
📁 ..                                    0 B drwxr-xr-x 1970-01-01 00:00
📁 src                                  96 B drwxr-xr-x 2025-07-18 09:42
📁 target                              160 B drwxr-xr-x 2025-07-18 09:42
📄 Cargo.lock                        25.0 KB -rw-r--r-- 2025-07-18 09:43
⚙️ Cargo.toml                          252 B -rw-r--r-- 2025-07-18 09:43
────────────────────────────────────────────────────────────────────────────────
Total files: 5
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## 📄 License

This project is open source. Feel free to use, modify, and distribute as needed.

## 🔮 Future Features

- [ ] File search functionality
- [ ] Sort options (size, date, type)
- [ ] File operations (copy, move, delete)
- [ ] Bookmarks for frequent directories
- [ ] Color themes
- [ ] Configuration file support
- [ ] Git integration (show git status)