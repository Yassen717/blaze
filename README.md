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

### Download from GitHub Releases (v0.1.1)

If you just want to use Blaze (no source build required), download the latest packaged binary from **GitHub Releases**.

1. Open your repository **Releases** page.
2. Select release tag **`v0.1.1`**.
3. Download the asset for your platform.
4. Extract/install and launch `blaze`.

Direct download page:

https://github.com/Yassen717/blaze/releases/tag/v0.1.1

### Publishing v0.1.1 (Maintainers)

Use this quick flow to publish `blaze` **0.1.1**:

```bash
# 1) Ensure version is correct
# Cargo.toml -> version = "0.1.1"

# 2) Build release artifact(s)
dx build --platform desktop --release

# 3) Tag and push
git tag v0.1.1
git push origin v0.1.1
```

Then create a GitHub Release for tag `v0.1.1` and upload the generated desktop artifacts from your release output.

Project packaging is configured to emit release artifacts under:

- `target/packager`
- `target/dx/blaze/release/windows/app`

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
- Read-only file system operations by default
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
| `curl <url> ...` | Fetch a URL (requires `curl` installed) | `curl https://example.com` |
| `wget <url> ...` | Fetch a URL (requires `wget` installed) | `wget https://example.com` |
| `cat` / `type <file>` | Display file contents (`type` is Windows-only alias) | `cat readme.txt` |
| `grep <pattern> <file>` | Search text in file | `grep "TODO" notes.txt` |
| `ipconfig` / `ip` | Show network config (Windows) | `ipconfig` |
| `ifconfig` / `ip` | Show network config (Linux/macOS) | `ifconfig` |
| `vim <file>` | Not supported (interactive TTY required) | `vim config.txt` |
| `whoami` | Display current user | `whoami` |

### Optional Mutating Commands (Desktop + `unsafe-fs`)

These commands are intentionally disabled by default and require the `unsafe-fs` feature.

| Command | Description | Example |
|---------|-------------|---------|
| `mkdir <name>` | Create directory | `mkdir new-folder` |
| `rm` / `del <path>` | Delete file or directory | `rm file.txt` |
| `mv <from> <to>` | Move or rename | `mv old.txt new.txt` |

## 🏗️ Architecture

```
blaze-terminal/
├── 📁 assets/              # Static assets (CSS, images, icons)
│   ├── main.css           # Main stylesheet
│   ├── tailwind.css       # Tailwind CSS file
│   └── branding/          # Brand assets
├── 📁 src/
│   ├── 📄 main.rs         # Application entry point
│   ├── 📁 components/     # Reusable UI components
│   │   ├── mod.rs
│   ├── 📁 terminal/       # Terminal domain module
│   │   ├── mod.rs
│   │   ├── components.rs  # Desktop/Web terminal UI components
│   │   ├── state.rs       # Terminal line state types
│   │   ├── utils.rs       # Shared helpers (arg parsing, line trimming)
│   │   └── 📁 commands/
│   │       ├── mod.rs
│   │   │   ├── 📁 desktop/
│   │   │   │   ├── mod.rs # Desktop command dispatcher
│   │   │   │   ├── fs.rs  # Filesystem command handlers
│   │   │   │   └── process.rs # Process/network command handlers
│   │       └── web.rs     # Web demo command simulation logic
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
# Print text
> echo Hello, Blaze!

# Display file contents (pick any real file on disk)
> cat README.md

# Search for text in files
> grep Blaze README.md
```

## 🔧 Development

### Features Flags

The project uses Cargo features to control platform-specific code:

```toml
[features]
default = ["desktop"]
web = ["dioxus/web"]           # Web platform support
desktop = ["dioxus/desktop"]   # Desktop platform support
safe-mode = []                  # Disable destructive commands (rm/del/mv/mkdir)
unsafe-fs = []                  # Opt-in mutating filesystem commands (mkdir/rm/del/mv)
```

### Windows Process Behavior

On Windows desktop builds, external commands are launched with `CREATE_NO_WINDOW` to avoid flashing console popups for short-lived commands (for example `curl`, `wget`, and `ipconfig`). Output is still captured and shown inside Blaze.

### Building for Different Platforms

```bash
# Desktop release build
cargo build --release --features desktop

# Web build (recommended via Dioxus CLI)
dx build --platform web --release

# Web build (via Cargo)
# Note: desktop is the default feature, so disable defaults for wasm builds.
cargo build --release --no-default-features --features web --target wasm32-unknown-unknown
```

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) for details.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 🐛 Issues & Support

- 🐛 [Report bugs](https://github.com/Yassen717/blaze/issues)
- 💡 [Request features](https://github.com/Yassen717/blaze/issues)
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

[⭐ Star us on GitHub](https://github.com/Yassen717/blaze) • [🌐 Try the Web Demo](https://your-demo-url.com) • [📖 Documentation](https://docs.blaze-terminal.com)

</div>


