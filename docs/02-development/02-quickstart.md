# InLocker - Quick Start Guide

Get InLocker running on your machine in minutes.

---

## Prerequisites

Verify you have the required tools installed:

```bash
# Xcode Command Line Tools
xcode-select --version
# If not installed: xcode-select --install

# Rust 1.91+
rustc --version
# If not installed: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node.js 24 LTS
node --version
# If not installed: brew install node@24

# pnpm 10.20+
pnpm --version
# If not installed: npm install -g pnpm

# Tauri CLI 2.8.5+
cargo tauri --version
# If not installed: cargo install tauri-cli --version "^2.8"
```

---

## Project Setup

### Step 1: Create Tauri Project

```bash
# In the InLocker folder
pnpm create tauri-app

# Prompts:
# - Project name: inlocker
# - Package manager: pnpm
# - UI template: React
# - TypeScript: Yes
# - Package name: com.inlocker.app
```

### Step 2: Install Frontend Dependencies

```bash
# UI libraries
pnpm add zustand@^5.0 date-fns@^4.1 lucide-react

# Tailwind CSS
pnpm add -D tailwindcss@^3.4 postcss autoprefixer @types/node

# Initialize Tailwind
pnpm dlx tailwindcss init -p
```

### Step 3: Add Rust Dependencies

Edit `src-tauri/Cargo.toml`:

```toml
[dependencies]
tauri = { version = "2.8", features = ["notification"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.42", features = ["full"] }
zstd = "0.13"
ring = "0.17"
notify = "7.0"
chrono = "0.4"
thiserror = "2.0"
log = "0.4"
env_logger = "0.11"

[build-dependencies]
tauri-build = "2.0"
```

### Step 4: Run the App

```bash
# Development mode (hot reload)
pnpm tauri dev
```

A window should open with your Tauri app running!

---

## Next Steps

Follow the implementation roadmap in `docs/04-roadmap.md` for Week 1 tasks.

For daily development commands, see `docs/dev-guide.md`.

---

## Troubleshooting

### Error: "xcrun: error: unable to find utility"
```bash
sudo xcode-select --switch /Library/Developer/CommandLineTools
```

### Error: "rustc not found"
```bash
source $HOME/.cargo/env
```

### Error: "tauri cli not found"
```bash
cargo install tauri-cli --version "^2.8" --locked
```

### Error: "pnpm command not found"
```bash
npm install -g pnpm
```

---

## Resources

- **Tauri Docs:** https://tauri.app/start/
- **Tauri + React:** https://tauri.app/guides/frontend/react
- **Rust Book:** https://doc.rust-lang.org/book/
- **React Docs:** https://react.dev/
