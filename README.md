<div align="center">

# ⚡ Blaze Terminal

**A blazingly fast, modern terminal emulator built with Rust**

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![Dioxus](https://img.shields.io/badge/dioxus-0.7.1-blue?style=for-the-badge)
![License](https://img.shields.io/badge/license-MIT-green?style=for-the-badge)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey?style=for-the-badge)

</div>

---

## 🎯 Overview

Blaze Terminal is a high-performance, cross-platform terminal emulator that combines the speed of Rust with the elegance of modern UI frameworks. Built with Dioxus 0.7, it offers both a native desktop application for real command execution and a web-based showcase for demonstration purposes.

## ✨ Features

<table>
<tr>
<td>

### 🚀 **Performance**
- Native Rust implementation
- Minimal memory footprint
- Instant command execution
- Smooth scrolling and rendering

</td>
<td>

### 🎨 **User Experience**  
- Modern, clean interface
- Custom window controls
- Color-coded output types
- Command history navigation

</td>
</tr>
<tr>
<td>

### 🔧 **Functionality**
- Built-in command set
- System command integration
- Directory navigation
- Error handling & feedback

</td>
<td>

### 🌐 **Cross-Platform**
- Desktop application
- Web demo showcase
- Responsive design
- Platform-specific optimizations

</td>
</tr>
</table>

## 📦 Installation

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Dioxus CLI](https://dioxuslabs.com/learn/0.7/getting_started/installation)

```bash
# Install Dioxus CLI
curl -sSL https://dioxus.dev/install.sh | sh
```

### Build from Source

```bash
# Clone the repository
git clone https://github.com/your-username/blaze-terminal.git
cd blaze-terminal

# Run desktop application
dx serve --platform desktop

# Or run web showcase  
dx serve --platform web
```

## 🚀 Quick Start

### Desktop Application

```bash
dx serve --platform desktop
```

The desktop app provides a full terminal experience with:
- Real command execution
- File system operations
- Custom window controls (minimize, maximize, close)
- Complete terminal functionality

### Web Showcase

```bash
dx serve --platform web
```

The web version includes:
- Interactive demo terminal
- Command reference pages
- Feature showcase
- Simulated command responses

## 📚 Commands Reference

### Built-in Commands

| Command | Description | Example |
|---------|-------------|---------|
| `help` | Display available commands | `help` |
| `clear` / `cls` | Clear terminal screen | `clear` |
| `cd <directory>` | Change working directory | `cd Documents` |
| `pwd` | Print working directory | `pwd` |
| `exit` | Close the terminal | `exit` |

### System Commands (Desktop Only)

| Command | Description | Example |
|---------|-------------|---------|
| `ls` / `dir` | List directory contents | `ls` or `dir` |
| `echo <text>` | Print text to terminal | `echo "Hello World"` |
| `mkdir <name>` | Create directory | `mkdir new-folder` |
| `rm` / `del <path>` | Delete file or directory | `rm file.txt` |
| `mv <from> <to>` | Move or rename | `mv old.txt new.txt` |
| `cat` / `type <file>` | Display file contents | `cat readme.txt` |
| `grep <pattern> <file>` | Search text in file | `grep "TODO" notes.txt` |
| `vim <file>` | Open file in Vim editor | `vim config.txt` |
| `whoami` | Display current user | `whoami` |

## 🏗️ Architecture

```
blaze-terminal/
├── 📁 assets/              # Static assets (CSS, images, icons)
│   ├── main.css           # Main stylesheet
│   ├── tailwind.css       # Tailwind CSS file
│   └── branding/          # Brand assets
├── 📁 src/
│   ├── 📄 main.rs         # Application entry point
│   ├── 📄 state.rs        # Terminal state management
│   ├── 📁 components/     # Reusable UI components
│   │   ├── mod.rs
│   │   └── terminal.rs    # Terminal component (desktop & web)
│   └── 📁 views/          # Web pages and routing
│       ├── mod.rs         # Route definitions
│       ├── home.rs        # Landing page
│       ├── commands.rs    # Command reference
│       └── demo.rs        # Interactive demo
├── 📄 Cargo.toml          # Rust dependencies
├── 📄 Dioxus.toml         # Dioxus configuration  
└── 📄 README.md           # Project documentation
```

## 🛠️ Technology Stack

- **Language**: [Rust](https://www.rust-lang.org/) 2021 Edition
- **UI Framework**: [Dioxus](https://dioxuslabs.com/) 0.7.1
- **Async Runtime**: [Tokio](https://tokio.rs/) (desktop only)
- **Styling**: [Tailwind CSS](https://tailwindcss.com/) (auto-configured)
- **Routing**: Dioxus Router (web only)

## 🎮 Usage Examples

### Basic Navigation
```bash
# Change to Documents folder
> cd Documents

# List files in current directory  
> ls

# Create a new folder
> mkdir projects

# Navigate to the new folder
> cd projects
```

### File Operations
```bash
# Create a file with content
> echo "Hello, Blaze!" > hello.txt

# Display file contents
> cat hello.txt

# Search for text in files
> grep "Blaze" hello.txt
```

## 🔧 Development

### Features Flags

The project uses Cargo features to control platform-specific code:

```toml
[features]
default = ["desktop"]
web = ["dioxus/web"]           # Web platform support
desktop = ["dioxus/desktop"]   # Desktop platform support
```

### Building for Different Platforms

```bash
# Desktop release build
cargo build --release --features desktop

# Web build  
dx build --platform web --release
```

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) for details.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 🐛 Issues & Support

- 🐛 [Report bugs](https://github.com/your-username/blaze-terminal/issues)
- 💡 [Request features](https://github.com/your-username/blaze-terminal/issues)
- ❓ [Ask questions](https://github.com/your-username/blaze-terminal/discussions)

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Dioxus Team](https://github.com/DioxusLabs/dioxus) for the amazing UI framework
- [Rust Community](https://www.rust-lang.org/community) for the incredible ecosystem
- All contributors who help make Blaze Terminal better

---

<div align="center">

**Built with ❤️ and ⚡ by the Blaze Terminal team**

[⭐ Star us on GitHub](https://github.com/your-username/blaze-terminal) • [🌐 Try the Web Demo](https://your-demo-url.com) • [📖 Documentation](https://docs.blaze-terminal.com)

</div>


