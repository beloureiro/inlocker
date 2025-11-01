# InLocker - quick start guide

This guide will help you start development in 15 minutes.

## prerequisites

Before starting, make sure you have installed:

```bash
# 1. Verify Xcode Command Line Tools
xcode-select --version
# If not installed: xcode-select --install

# 2. Verify Rust (1.91+ required)
rustc --version
# If not installed: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 3. Verify Node.js (24 LTS required)
node --version
# If not installed: brew install node@24

# 4. Install pnpm (10.20+)
npm install -g pnpm

# 5. Install Tauri CLI (2.8.5+)
cargo install tauri-cli --version "^2.8"
```

## project setup (week 1, steps 1-2)

### step 1: create tauri project

```bash
# In the InLocker folder, run:
pnpm create tauri-app

# When prompted:
# - Project name: inlocker
# - Package manager: pnpm
# - UI template: React
# - Add TypeScript: Yes
# - Package name (optional): com.inlocker.app
```

### step 2: move files to correct structure

```bash
# Tauri creates in root, let's reorganize
# (Backup the docs/ structure first!)

# Your docs are already in /docs, so:
# - Keep src/ for React
# - Keep src-tauri/ for Rust
```

### step 3: install additional dependencies

```bash
# Frontend (with latest versions)
pnpm add zustand@^5.0 date-fns@^4.1 lucide-react
pnpm add -D tailwindcss@^3.4 postcss autoprefixer
pnpm add -D @types/node

# Configure Tailwind
pnpm dlx tailwindcss init -p
```

### step 4: add rust dependencies

Edit `src-tauri/Cargo.toml` and add:

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

### step 5: run the app for the first time

```bash
# Return to project root
cd /Users/blc/Dev/Apps/InLocker

# Run in dev mode
pnpm tauri dev
```

If everything is correct, a window should open with the base Tauri app!

## next steps

Now follow the roadmap in `docs/04-roadmap.md`:

### week 1 - checklist

```markdown
environment configuration
- [x] Install Rust and tools
- [x] Install Node.js 24 LTS and pnpm 10.20+
- [x] Install Tauri CLI 2.8.5+
- [x] Verify Xcode Command Line Tools

project initialization
- [x] Create Tauri project with React + TypeScript template
- [ ] Configure folder structure (ui, core, services)
- [ ] Configure ESLint + Prettier + Tailwind
- [ ] Configure Git and .gitignore
- [ ] Create repository (optional)
```

## useful commands

```bash
# Development (hot reload)
pnpm tauri dev

# Production build
pnpm tauri build

# Check Rust code
cd src-tauri && cargo check

# Run Rust tests
cd src-tauri && cargo test

# Lint frontend
pnpm lint

# Format Rust
cd src-tauri && cargo fmt
```

## expected structure after setup

```
InLocker/
├── docs/                       ✅ Already exists
├── src/                        ⬅️ Create subfolders
│   ├── main.tsx
│   ├── App.tsx
│   ├── ui/                     ⬅️ Create
│   ├── core/                   ⬅️ Create
│   └── services/               ⬅️ Create
├── src-tauri/                  ✅ Created by Tauri CLI
│   ├── src/
│   │   └── main.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── public/
├── package.json
├── tsconfig.json
└── README.md
```

## troubleshooting

### error: "xcrun: error: unable to find utility"
```bash
sudo xcode-select --switch /Library/Developer/CommandLineTools
```

### error: "rustc not found"
```bash
source $HOME/.cargo/env
```

### error: tauri cli not found
```bash
cargo install tauri-cli --version "^2.8" --locked
```

### error: pnpm command not found
```bash
npm install -g pnpm
```

## resources

- **Tauri Docs:** https://tauri.app/start/
- **Tauri + React Guide:** https://tauri.app/guides/frontend/react
- **Rust Book:** https://doc.rust-lang.org/book/
- **React Docs:** https://react.dev/

## estimated time

- Initial setup: ~15 minutes
- First run: ~5 minutes
- Familiarization: ~30 minutes

**Total:** ~50 minutes to be coding!

---

**ready to start? Run the commands above and go to week 1 of the roadmap!**
