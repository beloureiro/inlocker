# InLocker Documentation

Welcome to the InLocker documentation! This is an automatic, compressed, and secure backup app for macOS.

## 📁 Documentation Structure

```
docs/
├── README.md                              # This file - navigation guide
├── 01-planning/                           # Project planning and architecture
│   ├── 01-value-proposition.md            # Problem, solution, competitive advantage
│   ├── 02-architecture.md                 # System design and data flow
│   └── 03-tech-stack.md                   # Technologies and justifications
├── 02-development/                        # Development resources
│   ├── 01-roadmap.md                      # Implementation phases (with progress)
│   ├── 02-quickstart.md                   # Setup instructions for developers
│   ├── 03-dev-guide.md                    # Daily commands and debugging
│   └── 04-testing-strategy.md             # Testing approach and test suites
└── 03-systems/                            # User and system guides
    ├── 01-user-guide.md                   # End-user instructions
    └── 02-inlog-system-guide.md           # InLog automatic changelog system
```

---

## 🚀 Quick Navigation

### For Users
- **[User Guide](03-systems/01-user-guide.md)** - How to use InLocker (5 min read)

### For Developers
- **[Quick Start](02-development/02-quickstart.md)** - Get up and running fast (10 min)
- **[Developer Guide](02-development/03-dev-guide.md)** - Daily commands and debugging (reference)

### For Understanding the Project

Read the core documentation in this recommended order:

---

## 📚 01 - Planning

### [01 - Value Proposition](01-planning/01-value-proposition.md)
**Read first to understand:**
- What problem we're solving
- Who it's for
- Why it's different from other solutions
- Success metrics

**Estimated time:** 5 minutes

---

### [02 - Architecture](01-planning/02-architecture.md)
**Read to understand:**
- How the system works internally
- Data flow (input → processing → output)
- Main components
- Visual diagram in ASCII

**Estimated time:** 10 minutes

---

### [03 - Tech Stack](01-planning/03-tech-stack.md)
**Read to understand:**
- Technologies used (Tauri, React, Rust)
- Why we chose each tool
- Project file structure
- Dependencies and requirements
- **Latest 2025/2026 versions**

**Estimated time:** 8 minutes

---

## 💻 02 - Development

### [01 - Roadmap](02-development/01-roadmap.md)
**Read to understand:**
- Implementation phases and progress (99% complete!)
- Task checklists with completion status
- Risks and mitigations
- MVP status and next steps

**Estimated time:** 10 minutes

---

### [02 - Quick Start](02-development/02-quickstart.md)
**Read to get started:**
- Environment setup
- Installation steps
- First run instructions

**Estimated time:** 10 minutes

---

### [03 - Developer Guide](02-development/03-dev-guide.md)
**Daily reference for:**
- Common commands
- Debugging tips
- Development workflow
- Troubleshooting

**Estimated time:** Reference (5 min to skim)

---

### [04 - Testing Strategy](02-development/04-testing-strategy.md)
**Read to understand:**
- Test architecture
- Test coverage goals
- How to run tests
- Test implementation examples

**Estimated time:** 8 minutes

---

## 📖 03 - Systems

### [01 - User Guide](03-systems/01-user-guide.md)
**End-user instructions:**
- Installing InLocker
- Creating backups
- Scheduling automatic backups
- Restoring files

**Estimated time:** 10 minutes

---

### [02 - InLog System Guide](03-systems/02-inlog-system-guide.md)
**Complete guide for:**
- InLog automatic changelog system
- Git hooks setup
- Conventional commits
- Customization options

**Estimated time:** 15-20 minutes

---

## 🎯 Getting Started

### New to InLocker Development?

1. **Start here:** [Quick Start Guide](02-development/02-quickstart.md) - Set up your environment
2. **Then read:** [Tech Stack](01-planning/03-tech-stack.md) - Understand the technologies
3. **Check progress:** [Roadmap](02-development/01-roadmap.md) - See what's done and what's next
4. **Daily work:** [Developer Guide](02-development/03-dev-guide.md) - Commands and troubleshooting

### Just Want to Use InLocker?

Go straight to the **[User Guide](03-systems/01-user-guide.md)** for step-by-step instructions

---

## 📊 Project Overview

```
InLocker is an app that:

INPUT               PROCESS                  OUTPUT
┌──────────┐        ┌──────────┐             ┌──────────┐
│ Folders  │   ───► │ Compress │      ───►   │ Backups  │
│ Schedules│        │ Encrypt  │             │ Secure   │
│ Destin.  │        │ Verify   │             │ Auto     │
└──────────┘        └──────────┘             └──────────┘
```

---

## ✨ Key Features

### Compression
- **Algorithm:** zstd (Zstandard)
- **Performance:** 2x faster than gzip with better compression ratio
- **Format:** TAR + ZSTD (.tar.zst)
- **Typical savings:** 40-70% space reduction

### Backup Types
- **Full Backup:** Complete copy of all files
- **Incremental Backup:** Only changed/new files since last backup
- **Smart Detection:** Compares file size and modification dates

### Scheduling
- **macOS Native:** Uses launchd for reliability
- **Presets Available:** Hourly, Daily, Weekly, Monthly
- **Custom Schedules:** Cron expression support
- **Works Offline:** Backups run even when app is closed

### User Experience
- **Real-time Progress:** Live feedback during backup operations
- **Native Notifications:** macOS notification center integration
- **Dark UI:** Easy on the eyes with emerald green accent
- **Minimal:** Clean interface, no clutter

---

## 🛠️ Technologies

Built with modern, production-ready tools:

- **Framework:** Tauri 2.9.2 (lightweight, secure, fast)
- **Backend:** Rust 1.91 (memory-safe, high-performance)
- **Frontend:** React 19.2 + TypeScript 5.8
- **State:** Zustand 5.0 (simple, minimal)
- **Styling:** TailwindCSS 3.4
- **Build:** Vite 7.1.12 (ultra-fast dev experience)
- **Package Manager:** pnpm 10.19.0
- **Git Hooks:** Husky 9.1.7 + InLog system

See [Tech Stack](01-planning/03-tech-stack.md) for detailed justifications.

---

## 🔗 Useful Links

- **Repository:** [github.com/beloureiro/inlocker](https://github.com/beloureiro/inlocker)
- **Issues:** [GitHub Issues](https://github.com/beloureiro/inlocker/issues)
- **Tauri Docs:** [tauri.app](https://tauri.app)
- **Rust Book:** [doc.rust-lang.org](https://doc.rust-lang.org/book/)

---

## 📝 Conventions

Throughout the documentation:

- ✅ = Implemented
- [ ] = Pending
- ⚠️ = Attention needed
- ❌ = Not implemented / discarded
- 🚀 = Priority feature
- 💡 = Idea for future

---

## 📈 Project Status

**Current Phase:** Phase 4 - Polish and Delivery 🚀 (99% Complete!)

**What's Working:**
- ✅ Full and incremental backups with zstd compression
- ✅ Real-time backup progress UI with live updates
- ✅ Automatic scheduling via macOS launchd (works with app closed!)
- ✅ Native macOS notifications (start/success/error)
- ✅ Backup restore functionality with integrity verification
- ✅ AES-256-GCM encryption (optional)
- ✅ Size tracking and compression ratio display
- ✅ Backup cancellation support
- ✅ InLog automatic changelog system
- ✅ 78 automated tests (all passing)

**Next Steps:**
- ⏳ Performance tests
- ⏳ Manual validation
- ⏳ Dashboard with metrics (nice-to-have)

---

## 🔄 Updates

This documentation is alive and will be updated as the project evolves.

**Last update:** 2025-11-05
**Documentation version:** 4.0.0
**Project status:** MVP near completion (99%)

---

**Happy reading and happy developing! 🚀**
