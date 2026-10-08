# Contributing to Blaze Terminal

Thanks for your interest in contributing! Here's how to get started.

## How to Contribute

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Development Setup

- Install the latest stable [Rust](https://rustup.rs/)
- Install the [Dioxus CLI](https://dioxuslabs.com/learn/0.7/getting_started/installation):
  `curl -sSL https://dioxus.dev/install.sh | sh`
- Run the desktop app with `dx serve --platform desktop`
- Run the web showcase with `dx serve --platform web`

## Guidelines

- Keep changes focused; open one pull request per feature or fix
- Follow existing code style and run `cargo fmt` before committing
- Make sure the project builds (`cargo build`) and passes `cargo clippy`
- Describe what you changed and how you tested it in the pull request

## Reporting Issues

- [Report bugs](https://github.com/Yassen717/blaze/issues)
- [Request features](https://github.com/Yassen717/blaze/issues)
- [Ask questions](https://github.com/Yassen717/blaze/discussions)
