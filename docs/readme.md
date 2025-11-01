# InLocker documentation

Welcome to the InLocker documentation! This is an automatic, compressed, and secure backup app for macOS.

## reading guide

We recommend reading the documents in this order:

### 1. [value proposition](01-value-proposition.md)
**Read first to understand:**
- What problem we're solving
- Who it's for
- Why it's different from other solutions
- Success metrics

**Estimated time:** 5 minutes

---

### 2. [architecture](02-architecture.md)
**Read to understand:**
- How the system works internally
- Data flow (input → processing → output)
- Main components
- Visual diagram in ASCII

**Estimated time:** 10 minutes

---

### 3. [tech stack](03-tech-stack.md)
**Read to understand:**
- Technologies used (Tauri, React, Rust)
- Why we chose each tool
- Project file structure
- Dependencies and requirements
- **Latest 2025/2026 versions**

**Estimated time:** 8 minutes

---

### 4. [roadmap](04-roadmap.md)
**Read to understand:**
- Implementation plan week by week
- Task checklist [ ]
- Risks and mitigations
- How to track progress

**Estimated time:** 10 minutes

---

## quick start

If you just want to start developing:

1. Read [tech stack](03-tech-stack.md) → "system requirements" section
2. Read [roadmap](04-roadmap.md) → week 1
3. Execute setup commands
4. Start coding following the checkboxes

---

## overview

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

## useful links

- **Repository:** [GitHub](link-to-be-added)
- **Issues:** [GitHub Issues](link-to-be-added)
- **Tauri Docs:** [tauri.app](https://tauri.app)
- **Rust Book:** [doc.rust-lang.org](https://doc.rust-lang.org/book/)

---

## conventions

Throughout the documentation:

- ✅ = Implemented
- [ ] = Pending
- ⚠️ = Attention needed
- ❌ = Not implemented / discarded
- 🚀 = Priority feature
- 💡 = Idea for future

---

## updates

This documentation is alive and will be updated as the project evolves.

**Last update:** 2025-11-01
**Documentation version:** 2.0.0
**Project status:** planning

---

**happy reading and happy developing!**
