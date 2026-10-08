<div align="center">

# ⚡ Blaze Terminal

**A blazingly fast, modern terminal emulator built with Rust**

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![Dioxus](https://img.shields.io/badge/dioxus-0.7.4-blue?style=for-the-badge)
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
- Full shell access through PTY
- ANSI/VT100 rendering
- Interactive TUI and REPL support
- Real-time keyboard forwarding

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

### Download from GitHub Releases

If you just want to use Blaze (no source build required), download the latest packaged binary from **GitHub Releases**.

1. Open the repository **Releases** page.
2. Open the **latest release**.
3. Download the asset for your platform.
4. Extract/install and launch `blaze`.

Direct download page:

https://github.com/Yassen717/blaze/releases/latest

### Publishing a Release (Maintainers)

Use this quick flow to publish the version in `Cargo.toml` (shown as `<version>` below):

```bash
# 1) Ensure version is correct
# Cargo.toml -> version = "<version>"

# 2) Build release artifact(s)
dx build --platform desktop --release

# 3) Tag and push
git tag v<version>
git push origin v<version>
```

Then create a GitHub Release for tag `v<version>` and upload the generated desktop artifacts from your release output.

Project packaging is configured to emit release artifacts under:

- `target/packager`
- `target/dx/blaze/release/windows/app`

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Dioxus CLI](https://dioxuslabs.com/learn/0.7/getting_started/#install-the-dioxus-cli)

```bash
# Install Dioxus CLI
curl -sSL https://dioxus.dev/install.sh | sh
```

### Build from Source

```bash
# Clone the repository
git clone https://github.com/Yassen717/blaze.git
cd blaze

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
- A real shell running inside a PTY
- ANSI/VT rendering with interactive app support
- Keyboard forwarding (Ctrl+C, Ctrl+D, arrows, function keys)
- Custom window controls (minimize, maximize, close)

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

### Desktop App (Real PTY Terminal)

The desktop build runs a real system shell inside a PTY. Blaze no longer uses
a command allowlist, so command behavior is owned by your shell.

Examples that work in desktop builds:

| Category | Example |
|---------|---------|
| Git tooling | `git log --oneline` |
| Rust tooling | `cargo test` |
| Interactive REPL | `python` |
| Full-screen TUI | `vim src/main.rs` |
| Remote sessions | `ssh user@host` |
| Pipes and redirects | `cargo test | findstr error` |

### Web Demo (Simulated)

The web build keeps a simulated command set for showcase purposes. Use
`help` in the demo to see supported commands.

## 🏗️ Architecture

```
blaze/
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
│   │   ├── components.rs  # Desktop PTY terminal + web demo component
│   │   ├── state.rs       # Terminal line state types (web demo)
│   │   ├── screen.rs      # PTY screen snapshot model
│   │   ├── utils.rs       # Shared helpers (line trimming)
│   │   ├── 📁 pty/
│   │   │   ├── mod.rs     # PTY session backend
│   │   │   ├── keys.rs    # Keyboard event to PTY bytes mapping
│   │   │   └── renderer.rs # Screen snapshot to RSX renderer
│   │   └── 📁 commands/
│   │       ├── mod.rs     # Module gate for web command simulator
│   │       └── web.rs     # Web demo command simulation logic
│   └── 📁 views/          # Web pages and routing
│       ├── mod.rs         # Route definitions
│       ├── home.rs        # Landing page
│       ├── commands.rs    # Command reference
│       ├── demo.rs        # Interactive demo
│       ├── download.rs    # Download page
│       └── not_found.rs   # 404 fallback page
├── 📄 Cargo.toml          # Rust dependencies
├── 📄 Dioxus.toml         # Dioxus configuration  
└── 📄 README.md           # Project documentation
```

## 🛠️ Technology Stack

- **Language**: [Rust](https://www.rust-lang.org/) 2021 Edition
- **UI Framework**: [Dioxus](https://dioxuslabs.com/) 0.7.4
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
safe-mode = []                  # Deprecated in v0.2.0 (no-op)
unsafe-fs = []                  # Deprecated in v0.2.0 (no-op)
```

### Windows Shell Backend

Windows desktop builds use ConPTY through `portable-pty` and launch a real
shell session (PowerShell by default). Older Windows builds without ConPTY
support are not supported.

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
- ❓ [Ask questions](https://github.com/Yassen717/blaze/discussions)

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Dioxus Team](https://github.com/DioxusLabs/dioxus) for the amazing UI framework
- [Rust Community](https://www.rust-lang.org/community) for the incredible ecosystem
- All contributors who help make Blaze Terminal better

---

<div align="center">

**Built with ❤️ and ⚡ by the Blaze Terminal team**

[⭐ Star us on GitHub](https://github.com/Yassen717/blaze) • [🌐 Releases](https://github.com/Yassen717/blaze/releases) • [🐛 Issues](https://github.com/Yassen717/blaze/issues)

</div>


