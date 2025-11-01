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

1. **[user guide](docs/USER-GUIDE.md)** 👤
   - How to use InLocker
   - Creating backups
   - Managing configurations
   - Troubleshooting

2. **[value proposition](docs/01-value-proposition.md)**
   - The problem we solve
   - Who it's for
   - Competitive advantages

3. **[architecture](docs/02-architecture.md)**
   - System diagram
   - Data flow
   - Main components

4. **[tech stack](docs/03-tech-stack.md)**
   - Technologies used
   - Justifications for choices
   - File structure

5. **[roadmap](docs/04-roadmap.md)**
   - Implementation plan (4 weeks)
   - Progress checklist
   - Success metrics

---

## tech stack

**frontend:**
- React 19.2 + TypeScript 5.8
- TailwindCSS 3.4 + shadcn/ui
- Zustand 5.0 (state)
- lucide-react (icons)

**backend:**
- Tauri 2.9.2 (framework)
- Rust 1.91.0 (core)
- zstd 0.13 (compression)
- ring (cryptography)

**build tools:**
- Node.js 23.11.1
- pnpm 10.19.0
- Vite 7.1.12

---

## project status

**current phase:** Phase 1 ✅ COMPLETE | Phase 2 🔄 NEXT

### progress
- [x] Directory structure
- [x] Complete documentation
- [x] Architecture defined
- [x] Tech stack chosen (latest 2025 versions)
- [x] Environment setup complete
- [x] Tauri + React app running
- [x] **Phase 1: Foundation** ✅ COMPLETE
  - [x] Backend: types, commands, IPC, JSON persistence
  - [x] Frontend: Layout, FolderSelector, BackupList, Zustand store
  - [x] Design: Emerald green theme, lock icon logo, dark titlebar
- [ ] **Phase 2: Backup Core** 🔄 NEXT
- [ ] **Phase 3: Automation and Security** ⏳
- [ ] **Phase 4: Polish and Delivery** ⏳

---

## prerequisites

### for development
- macOS 12.0+ (Monterey)
- Xcode Command Line Tools
- Rust 1.91.0+
- Node.js 23.11.1+
- pnpm 10.19.0+

### for end user
- macOS 12.0+
- 100 MB free space

---

## roadmap

### MVP features
- ✅ Folder selection and configuration management
- ⏳ Manual and automatic backup
- ⏳ zstd compression
- ⏳ Optional encryption
- ⏳ Dashboard with metrics
- ⏳ Point-in-time restore

### future enhancements
- Incremental backup
- Pattern exclusion (node_modules, .git)
- Multiple destinations
- Linux/Windows support
- Optional cloud sync

---

## quick start

**For end users:** See [docs/USER-GUIDE.md](docs/USER-GUIDE.md) for a simple guide on using InLocker.

**For developers:** See [quickstart.md](quickstart.md) for detailed setup instructions.

```bash
# Clone repository
git clone https://github.com/beloureiro/inlocker.git
cd inlocker

# Install dependencies
pnpm install

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

Issues: [GitHub Issues](https://github.com/beloureiro/inlocker/issues)

---

**built with Tauri + Rust + React**
