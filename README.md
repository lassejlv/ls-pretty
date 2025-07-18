# 📁 ls-pretty

A powerful Terminal User Interface (TUI) file browser with integrated text editor and terminal, written in Rust. Experience beautiful file navigation, syntax-highlighted editing, and seamless terminal integration all in one application.

## ✨ Key Features

### 🎨 **Beautiful TUI Interface**
- Interactive file browser with intuitive keyboard navigation
- Elegant design with icons, colors, and visual feedback
- Responsive layout with multiple dizzzzzzzzzzzzText Editor**
### 📝 **Integrated Text Editor**
- **Syntax highlighting** for 20+ programming languages
- **Real-time editing** with blinking cursor and line numbers
- **Current line highlighting** with dark background
- **Advanced search** with Ctrl+F and match highlighting
- **Multi-cursor editing** for simultaneous edits
- **Save functionality** with Ctrl+S
- **Unsaved changes protection** with smart alerts
- **View/Edit mode toggle** for seamless workflow

### 🚀 **Go Language Server & Autocomplete**
- **Integrated Go LSP** with `gopls` language server support
- **Automatic autocomplete** that triggers as you type in Go files
- **Manual autocomplete** with Ctrl+Space for Go files
- **Real-time completion suggestions** with function signatures
- **Smart context-aware completions** from Go standard library
- **Seamless LSP integration** that starts automatically for .go files
- **Tab completion** to accept suggestions
- **Visual completion popup** with detailed function information
- **Fixed vim-style navigation** - 'k' and 'j' keys work properly in edit mode

### 💻 **Built-in Terminal**
- **Integrated terminal** at bottom of screen (Ctrl+T)
- **Real pseudo-terminal** with full shell support
- **Current directory context** - starts where you're browsing
- **Command execution** with live output display
- **Graceful fallback** if PTY unavailable

### 🔍 **Advanced File Management**
- **Recursive file finder** with Ctrl+O for instant navigation
- **Smart file filtering** with real-time search
- **Cross-directory file access** without leaving the interface

### 🎯 **Enhanced Navigation**
- **Line numbers** in both view and edit modes
- **Cursor navigation** with arrow keys (↑↓←→)
- **Smart scrolling** to keep cursor visible
- **30+ line viewport** for better content visibility
- **Vim-like controls** (hjkl) plus arrow keys

## 📋 **Prerequisites**

### For Go Language Server Support:
```bash
# Install Go language server (gopls)
go install golang.org/x/tools/gopls@latest

# Ensure gopls is in your PATH
export PATH=$PATH:$(go env GOPATH)/bin

# Verify installation
gopls version
```

## 🔍 **How to Know if Go LSP is Working**

### Visual Indicators:
1. **Header Status**: When editing Go files, look for status in the header:
   - `🟢 LSP` = Language server running and ready
   - `🟡 LSP` = Language server starting up  
   - `🔴 LSP` = Language server failed or not installed
   - `⚪ LSP` = Language server not started

2. **Footer Messages**: The footer shows current LSP status:
   - `"🟢 LSP ready - Ctrl+Space for autocomplete"` = Working perfectly
   - `"🔴 LSP failed - Press Ctrl+Space for details"` = Problem detected
   - `"🟡 LSP starting..."` = Server initializing

3. **Autocomplete Behavior**:
   - **Working**: Press `Ctrl+Space` → Shows completion popup with Go functions
   - **Not Working**: Press `Ctrl+Space` → Shows error message about installation

### Troubleshooting Steps:
```bash
# 1. Check if gopls is installed
which gopls

# 2. If not found, install it
go install golang.org/x/tools/gopls@latest

# 3. Verify Go environment
go env GOPATH
go env GOROOT

# 4. Make sure GOPATH/bin is in your PATH
echo $PATH | grep "$(go env GOPATH)/bin"
```

### Test the Integration:
1. Open the included `test.go` file
2. Press `Enter` to view the file
3. Press `Ctrl+E` to enter edit mode
4. Look for LSP status indicators in header: `📁 /path | 🐹 Go 🟢 LSP Ready`
5. Type `fmt.` and press `Ctrl+Space`
6. You should see autocomplete suggestions with function signatures

## 🚀 Installation

```bash
git clone <your-repo>
cd ls-pretty
cargo build --release
```

## 📖 Usage

### Interactive TUI Mode

#### **Go Development Features:**
```bash
# Open a Go file for editing
cargo run .
# Navigate to a .go file and press Enter
# Press Ctrl+E to enter edit mode
# Press Ctrl+Space for autocomplete suggestions
# Use ↑↓ to navigate completions, Tab to accept
```

#### **Go Autocomplete Demo:**
1. Open the included `test.go` file
2. Enter edit mode (Ctrl+E)  
3. **Check LSP status** in header - should show `🟢 LSP`
4. Type `fmt.` and press `Ctrl+Space`
5. **If working**: See popup with `fmt.Println`, `fmt.Printf`, etc.
6. **If not working**: See error message about gopls installation
7. Navigate suggestions with ↑↓ arrows, press Tab to insert
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
| `Ctrl+E` | Toggle between view/edit modes |
| `↑↓←→` | Navigate cursor (edit) / scroll (view) |
| `Tab` | Insert 4 spaces for indentation |
| `Ctrl+F` | Open search mode |
| `F3` / `Shift+F3` | Next/previous search match |
| `Ctrl+D` | Toggle multi-cursor mode |
| `Alt+Enter` | Add cursor at position (multi-cursor mode) |
| `Ctrl+S` | Save file changes |
| `Ctrl+Z` | Revert all changes to original |
| `Enter` | New line at cursor |
| `Backspace` | Delete character before cursor |
| `Esc` | Close file (with unsaved changes protection) |

### Go Language Server (LSP) & Autocomplete
| Key | Action |
|-----|--------|
| `Ctrl+Space` | Trigger autocomplete suggestions (Go files) |
| `↑↓` | Navigate through autocomplete suggestions |
| `Tab` | Accept selected autocomplete suggestion |
| `Esc` | Close autocomplete popup or LSP status |
| `Backspace` | Hide autocomplete and delete character |

**LSP Status Indicators:**
- **Header**: `📁 /path | 🐹 Go 🟢 LSP Ready` (shows current LSP state)
- **Footer**: Dynamic messages about LSP status and capabilities
- **🟢 Green**: LSP running and ready for autocomplete
- **🟡 Yellow**: LSP starting up or initializing  
- **🔴 Red**: LSP failed - check gopls installation

### File Finder
| Key | Action |
|-----|--------|
| `Ctrl+O` | Open recursive file finder |
| `Type` | Filter files by name |
| `↑↓` | Navigate through results |
| `Enter` | Open selected file |
| `Esc` | Close file finder |

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
| `R` | Revert to original and close |
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
- **Advanced search** with match highlighting and navigation
- **Multi-cursor editing** for simultaneous text manipulation
- **Tab support** with 4-space indentation
- **Revert functionality** with Ctrl+Z to undo all changes
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

### File Management Features
- **Recursive file finder** scans entire project directory
- **Smart filtering** with real-time search as you type
- **Instant navigation** to any file without manual browsing
- **Cross-directory access** from anywhere in the project
- **Intelligent exclusions** (skips .git, target, node_modules)

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

### Text Editor with Advanced Features
```
┌ main.rs (EDITING - UNSAVED) ─────────────────────────────────────────────┐
│  1 use std::collections::HashMap;                                        │
│  2 use clap::Parser;                                                     │
│  3                                                                       │
│  4 fn main() {                                                           │
│  5     println!("Hello, world!");█    // <- Current cursor              │
│  6     println!("Search this!");      // <- Highlighted search match    │
│  7 }                                   // <- Multi-cursor position       │
│  8                                                                       │
└──────────────────────────────────────────────────────────────────────────┘
│EDIT: Ctrl+F search, Ctrl+O finder, Ctrl+D multi-cursor | Cursor: 5:27│2 cursors│

┌ File Finder: demo ───────────────────────────────────────────────────────┐
│ demo.rs (src/demo.rs)                                                    │
│ advanced_features_demo.rs (advanced_features_demo.rs)                   │
│ test_file.txt (test_file.txt)                                           │
└──────────────────────────────────────────────────────────────────────────┘
│Type to filter, ↑↓ to navigate, Enter to open, Esc to close              │
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
✅ **Professional editing**: Real syntax highlighting, advanced search, multi-cursor support, and tab indentation  
✅ **Instant navigation**: Recursive file finder lets you jump to any file instantly  
✅ **Advanced search**: Find and navigate through code with highlighted matches  
✅ **Multi-cursor magic**: Edit multiple locations simultaneously for powerful refactoring  
✅ **Safe editing**: Revert changes with Ctrl+Z, unsaved changes protection with multiple options  
✅ **Terminal integration**: Execute commands in context with live output  
✅ **Beautiful design**: Carefully crafted UI with icons, colors, and smooth interactions  
✅ **Developer-focused**: Built by developers, for developers, with attention to workflow efficiency

Whether you're exploring codebases, editing configuration files, or running quick commands, ls-pretty provides a seamless, productive experience that feels natural and powerful.

**Try it today and transform your terminal file management! 🚀**