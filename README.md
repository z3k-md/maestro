# Maestro 🎵

[![CI](https://github.com/z3k-md/maestro/actions/workflows/ci.yaml/badge.svg?branch=main)](https://github.com/z3k-md/maestro/actions/workflows/ci.yaml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows-lightgrey.svg)](https://github.com/z3k-md/maestro)

---
### Ditch Window's uninspired audio controls. 

Maestro is a beautiful, modern audio mixer for Windows. Control individual application volumes with mappable key bindings.

## ✨ Features

- 🎨 **Beautiful UI** - Modern interface that replaces Windows' basic volume controls
- 🔧 **Configurable** - Remap keybinds, change themes, and customize behavior
- 🚀 **Lightweight** - Built with Tauri for optimal performance

## 📦 Installation

#### Download the [latest release](https://github.com/z3k-md/maestro/releases)

#### Requirements
- Windows 10/11
- WebView2 Runtime (pre-installed with modern releases)

## 🚀 Quick Start

1. Download and install Maestro
2. Runs at startup by default

## ⚙️ Configuration

Settings are available from the tray icon, or edit the config file directly:

- Config: `%APPDATA%\com.maestro.app\config.yaml`
- Logs: `%LOCALAPPDATA%\com.maestro.app\logs` (also reachable from the tray menu)

## 🛠️ Development

### Architecture
Built with [Tauri](https://tauri.app), Windows API integration, and audio session management handled in [Rust](https://rust-lang.org/). UI uses [WebView2](https://learn.microsoft.com/en-us/microsoft-edge/webview2/), built with Vue.

### Development Setup

#### Requirements
- **Rust** (latest stable)
- **Bun** (or equivalent runtime)

#### Getting Started
```bash
# Clone the repository
git clone https://github.com/z3k-md/maestro.git

# Install dependencies
bun install

# Start development server
bun run start

# Build the executable
bun run tauri build

# Checks (also run in CI)
bun run typecheck
cd src-tauri && cargo fmt --check && cargo clippy --all-targets && cargo test
```

### Releasing
`bun run version [patch|minor|major]` bumps the version from the latest tag, then offers to commit, tag, and push. Pushing a `v*` tag builds the installer and publishes a GitHub release.

## 🤝 Contributing

Contributions are welcome!
- 🐛 **Report Bugs**: Open an issue with detailed reproduction steps
- 💡 **Feature Requests**: Suggest new features or improvements
- 🔧 **Code Contributions**: Submit pull requests for bug fixes or features


#### Future Plans
- Hardware integrations
- Automatic updates  
- Multiple output devices
- Input device support


## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

---

**Made with ❤️ for Windows enthusiasts**
