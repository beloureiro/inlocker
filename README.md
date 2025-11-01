# InLocker

**automatic, compressed, and secure backups — simple, reliable, and under control**

A native macOS app that performs automatic backups of your important folders, with intelligent compression and optional encryption, all running locally on your computer.

---

## features

- **lightweight and fast**: Binary <5 MB, startup <500ms
- **total automation**: Set schedules and forget
- **modern compression**: Save up to 50% with zstd
- **security**: Optional AES-256 encryption
- **intuitive dashboard**: Visual metrics and history
- **local control**: Your data stays on your Mac
- **performance**: 1GB backup in <2 minutes

---

## documentation

Complete documentation is organized in `/docs`:

1. **[value proposition](docs/01-value-proposition.md)**
   - The problem we solve
   - Who it's for
   - Competitive advantages

2. **[architecture](docs/02-architecture.md)**
   - System diagram
   - Data flow
   - Main components

3. **[tech stack](docs/03-tech-stack.md)**
   - Technologies used
   - Justifications for choices
   - File structure

4. **[roadmap](docs/04-roadmap.md)**
   - Implementation plan (4 weeks)
   - Progress checklist
   - Success metrics

---

## tech stack

**frontend:**
- React 19.2 + TypeScript 5.9
- TailwindCSS + shadcn/ui
- Zustand (state)

**backend:**
- Tauri 2.8.5 (framework)
- Rust 1.91+ (core)
- zstd 0.13 (compression)
- ring (cryptography)

**build tools:**
- Node.js 24 LTS "Krypton"
- pnpm 10.20+
- Vite 7.0

---

## project status

**current phase:** planning ✅

### progress
- [x] Directory structure
- [x] Complete documentation
- [x] Architecture defined
- [x] Tech stack chosen (latest 2025 versions)
- [ ] Environment setup
- [ ] MVP development (4 weeks)

---

## prerequisites

### for development
- macOS 12.0+ (Monterey)
- Xcode Command Line Tools
- Rust 1.91+
- Node.js 24 LTS+
- pnpm 10.20+

### for end user
- macOS 12.0+
- 100 MB free space

---

## roadmap

### MVP (4 weeks)
- Drag-and-drop folder selection
- Manual and automatic backup
- zstd compression
- Optional encryption
- Dashboard with metrics
- Point-in-time restore

### future
- Incremental backup
- Pattern exclusion (node_modules, .git)
- Multiple destinations
- Linux/Windows support
- Optional cloud sync

---

## quick start

See [QUICKSTART.md](QUICKSTART.md) for detailed setup instructions.

```bash
# Install Tauri CLI
cargo install tauri-cli --version "^2.8"

# Create project
pnpm create tauri-app

# Run development server
pnpm tauri dev
```

---

## license

TBD

---

## contributing

This project is in initial development. Contributions will be welcome after MVP.

---

## contact

Issues: [GitHub Issues](link-to-be-added)

---

**built with Tauri + Rust + React**
