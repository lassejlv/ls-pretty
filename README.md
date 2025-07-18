# 📁 ls-pretty

A powerful Terminal User Interface (TUI) file browser with integrated text editor and terminal, written in Rust. Experience beautiful file navigation, syntax-highlighted editing, and seamless terminal integration all in one application.

## ✨ Key Features

### 🎨 **Beautiful TUI Interface**
- Interactive file browser with intuitive keyboard navigation
- Elegant design with icons, colors, and visual feedback
- Responsive layout with multiple display modes

### 📝 **Integrated Text Editor**
- **Syntax highlighting** for 20+ programming languages
- **Real-time editing** with blinking cursor and line numbers
- **Current line highlighting** with dark background
- **Save functionality** with Ctrl+S
- **Unsaved changes protection** with smart alerts
- **View/Edit mode toggle** for seamless workflow

### 💻 **Built-in Terminal**
- **Integrated terminal** at bottom of screen (Ctrl+T)
- **Real pseudo-terminal** with full shell support
- **Current directory context** - starts where you're browsing
- **Command execution** with live output display
- **Graceful fallback** if PTY unavailable

### 🎯 **Enhanced Navigation**
- **Line numbers** in both view and edit modes
- **Cursor navigation** with arrow keys (↑↓←→)
- **Smart scrolling** to keep cursor visible
- **30+ line viewport** for better content visibility
- **Vim-like controls** (hjkl) plus arrow keys

## 🚀 Installation

```bash
git clone <your-repo>
cd ls-pretty
cargo build --release
```

## 📖 Usage

### Interactive TUI Mode
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
# Non-interactive list output
./target/release/ls-pretty -l

# Combine options
./target/release/ls-pretty -l -H -a /path/to/directory
```

## ⌨️ Controls

### File Browser
| Key | Action |
|-----|--------|
| `↑/k` | Move selection up |
| `↓/j` | Move selection down |
| `Enter` | Open directory or view/edit file |
| `a` | Toggle hidden files |
| `h` | Show/hide help |
| `Ctrl+T` | Toggle integrated terminal |
| `q/Esc` | Quit application |

### Text Editor
| Key | Action |
|-----|--------|
| `E` | Toggle between view/edit modes |
| `↑↓←→` | Navigate cursor (edit) / scroll (view) |
| `Ctrl+S` | Save file changes |
| `Enter` | New line at cursor |
| `Backspace` | Delete character before cursor |
| `Esc` | Close file (with unsaved changes protection) |

### Terminal
| Key | Action |
|-----|--------|
| `Ctrl+T` | Open/close terminal |
| `Type + Enter` | Execute commands |
| `Ctrl+C` | Send interrupt to running command |
| `↑↓` | Navigate in terminal mode |

### Unsaved Changes Dialog
| Key | Action |
|-----|--------|
| `S` | Save and close |
| `D` | Discard changes and close |
| `C` | Cancel (continue editing) |

## 📂 File Type Support

### Icons & Recognition
| Type | Icon | Extensions |
|------|------|------------|
| Directory | 📁 | - |
| Rust | 🦀 | `.rs` |
| Python | 🐍 | `.py` |
| JavaScript/TypeScript | 📜 | `.js`, `.ts` |
| Web | 🌐 | `.html` |
| Styles | 🎨 | `.css` |
| JSON | 📄 | `.json` |
| Markdown | 📝 | `.md` |
| Text | 📃 | `.txt` |
| Images | 🖼️ | `.png`, `.jpg`, `.jpeg`, `.gif` |
| Audio | 🎵 | `.mp3`, `.wav`, `.flac` |
| Video | 🎬 | `.mp4`, `.avi`, `.mkv` |

### Syntax Highlighting Support
**Programming Languages:**
- Rust, Python, JavaScript, TypeScript, Java, Go, C/C++, PHP, Ruby, Lua, Perl

**Web Technologies:**
- HTML, CSS, JSON, XML, YAML, TOML

**Scripts & Config:**
- Shell scripts (.sh, .bash, .zsh, .fish), Makefiles, Dockerfiles

**Documentation:**
- Markdown, Plain text, CSV

**And many more!**

## 🎯 Advanced Features

### Text Editor Capabilities
- **Blinking cursor** (500ms interval) for precise editing
- **Current line highlighting** with dark background
- **Line numbers** with smart width calculation  
- **Syntax highlighting** in both view and edit modes
- **Real-time change tracking** with visual indicators
- **Auto-scrolling** to keep cursor visible
- **Cross-platform save** with Ctrl+S
- **Data loss prevention** with unsaved changes alerts

### Terminal Integration
- **Pseudo-terminal (PTY)** for full shell experience
- **Current working directory** sync with file browser
- **Live command output** with scrollable history
- **Background process support** with proper signal handling
- **Robust error handling** with fallback modes

### UI Enhancements
- **30+ line display** for better productivity
- **Smart status bar** showing cursor position and file stats
- **Context-aware help** system
- **Professional color scheme** with syntax highlighting themes
- **Responsive layout** adapting to terminal size

## 🛠️ Dependencies

```toml
[dependencies]
ratatui = "0.24"          # Terminal UI framework
crossterm = "0.27"        # Cross-platform terminal control
clap = "4.0"              # Command line parsing
chrono = "0.4"            # Date/time handling
syntect = "5.1"           # Syntax highlighting engine
portable-pty = "0.8"      # Pseudo-terminal support
anyhow = "1.0"            # Error handling
```

## 🚧 Requirements

- **Rust 1.70+** (uses 2024 edition)
- **Unicode terminal** for optimal icon display
- **True color support** for syntax highlighting
- **Unix-like system** for full PTY support (graceful fallback on Windows)

## 🎨 Screenshots

### File Browser with Syntax Highlighting
```
┌Directory─────────────────────────────────────────────────────────────────┐
│📁 /Users/dev/ls-pretty                                                   │
└──────────────────────────────────────────────────────────────────────────┘
┌Files─────────────────────────────────────────────────────────────────────┐
│➤ 📁 ..                          0B drwxr-xr-x 2024-01-01 00:00          │
│  📁 src                        96B drwxr-xr-x 2024-01-15 10:30          │
│  🦀 main.rs                  15.2K -rw-r--r-- 2024-01-15 10:45          │
│  📄 Cargo.toml                 512B -rw-r--r-- 2024-01-15 09:20          │
│  📝 README.md                 8.1K -rw-r--r-- 2024-01-15 10:50          │
└──────────────────────────────────────────────────────────────────────────┘
┌Terminal (Ctrl+T to close)───────────────────────────────────────────────┐
│$ ls -la                                                                  │
│total 128                                                                 │
│drwxr-xr-x  5 user staff  160 Jan 15 10:50 .                            │
│drwxr-xr-x 10 user staff  320 Jan 15 09:15 ..                           │
│-rw-r--r--  1 user staff  512 Jan 15 09:20 Cargo.toml                   │
│$ █                                                                       │
└──────────────────────────────────────────────────────────────────────────┘
┌Controls──────────────────────────────────────────────────────────────────┐
│↑↓ Navigate  Enter Open  E Edit  Ctrl+T Terminal  h Help  q Quit         │
└──────────────────────────────────────────────────────────────────────────┘
```

### Text Editor Mode
```
┌ main.rs (EDITING - UNSAVED) ─────────────────────────────────────────────┐
│  1 use std::collections::HashMap;                                        │
│  2 use clap::Parser;                                                     │
│  3                                                                       │
│  4 fn main() {                                                           │
│  5     println!("Hello, world!");█                                      │
│  6 }                                                                     │
│  7                                                                       │
│  8                                                                       │
└──────────────────────────────────────────────────────────────────────────┘
│EDIT MODE: Type/↑↓←→ navigate, Ctrl+S save, E view, Esc close | Cursor: 5:27│
```

## 🔮 Architecture

### Core Components
- **File Browser Engine**: Efficient directory traversal with metadata caching
- **Syntax Highlighting**: Powered by syntect with 100+ language definitions
- **Terminal Emulator**: Full PTY implementation with signal handling
- **Text Editor**: Custom editor with cursor management and change tracking
- **UI Framework**: Built on ratatui with custom widgets and layouts

### Performance Features
- **Lazy loading** for large directories
- **Efficient rendering** with minimal screen updates
- **Memory management** for large files and long terminal sessions
- **Async I/O** for responsive user experience

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes with tests
4. Commit your changes (`git commit -m 'Add amazing feature'`)
5. Push to the branch (`git push origin feature/amazing-feature`)
6. Open a Pull Request

### Development Setup
```bash
git clone <your-repo>
cd ls-pretty
cargo build
cargo test
cargo run -- --help
```

## 📄 License

This project is open source. Feel free to use, modify, and distribute as needed.

## 🎉 What Makes ls-pretty Special

**ls-pretty** isn't just another file browser—it's a complete development environment in your terminal:

✅ **All-in-one workflow**: Browse files, edit code, run commands—all without leaving the interface
✅ **Professional editing**: Real syntax highlighting, cursor navigation, and save functionality  
✅ **Terminal integration**: Execute commands in context with live output
✅ **Beautiful design**: Carefully crafted UI with icons, colors, and smooth interactions
✅ **Developer-focused**: Built by developers, for developers, with attention to workflow efficiency

Whether you're exploring codebases, editing configuration files, or running quick commands, ls-pretty provides a seamless, productive experience that feels natural and powerful.

**Try it today and transform your terminal file management! 🚀**