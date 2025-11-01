# InLocker - implementation roadmap

## overview

**Total Duration:** 4 weeks
**Model:** Iterative incremental
**Goal:** Functional and tested MVP

---

## week 1: setup and foundation

### environment configuration
- [x] Install Rust and tools (rustup, cargo) - Rust 1.91.0 ✅
- [x] Install Node.js 23.11.1 and pnpm 10.19.0 ✅
- [x] Install Tauri CLI 2.9.2 ✅
- [x] Verify Xcode Command Line Tools ✅

### project initialization
- [x] Create Tauri project with React + TypeScript template ✅
- [x] Configure folder structure (ui, core, services) ✅
- [x] Configure ESLint + Prettier + Tailwind ✅
- [x] Configure Git and .gitignore ✅
- [x] Create repository at github.com/beloureiro/inlocker ✅

### basic backend (Rust)
- [ ] Define basic types (BackupConfig, BackupJob)
- [ ] Implement Tauri command: `select_folder`
- [ ] Implement Tauri command: `save_config`
- [ ] Implement Tauri command: `load_config`
- [ ] Test IPC communication (frontend ↔ backend)

### basic frontend (React)
- [ ] Create main layout (Header + Sidebar + Content)
- [ ] Create FolderSelector component (drag-drop)
- [ ] Create basic Zustand store
- [ ] Implement folder selection via Tauri
- [ ] Display selected folders in UI

**Week 1 Deliverable:** App opens, user can select folders

---

## week 2: backup core

### compression engine
- [ ] Add `zstd` dependency in Cargo.toml
- [ ] Implement function `compress_folder(path, output)`
- [ ] Create filename with timestamp
- [ ] Test compression of small folder
- [ ] Add progress logs

### manual backup execution
- [ ] Create Tauri command: `run_backup_now`
- [ ] Implement complete backup logic
- [ ] Generate compressed file in destination
- [ ] Calculate size before/after
- [ ] Return result to frontend

### backup UI
- [ ] Create "Backup Now" button
- [ ] Add loading state during backup
- [ ] Display success/error notification
- [ ] Show statistics (size, time)
- [ ] Create list of completed backups

### data persistence
- [ ] Save configurations in local JSON
- [ ] Load configurations on app start
- [ ] Implement backup history
- [ ] Store metadata (date, size, status)

**Week 2 Deliverable:** User performs manual backup and sees result

---

## week 3: automation and security

### scheduler (scheduling)
- [ ] Implement cron expression parser
- [ ] Create ScheduleConfig component (UI)
- [ ] Add schedule presets (daily, weekly, etc)
- [ ] Save schedule with configuration
- [ ] Create background task in Rust with tokio

### launchd integration (macOS)
- [ ] Generate .plist file for launchd
- [ ] Install daemon when configuring schedule
- [ ] Tauri command: `register_schedule`
- [ ] Tauri command: `unregister_schedule`
- [ ] Test automatic trigger

### encryption (optional)
- [ ] Add `ring` dependency in Cargo.toml
- [ ] Implement function `encrypt_file(input, password)`
- [ ] Implement function `decrypt_file(input, password)`
- [ ] Add toggle in UI (enable/disable)
- [ ] Password input with confirmation

### native notifications
- [ ] Use Tauri notification API
- [ ] Notify scheduled backup success
- [ ] Notify backup error
- [ ] Add sounds (optional)

**Week 3 Deliverable:** Automatic backups working + optional encryption

---

## week 4: polish and delivery

### dashboard and metrics
- [ ] Create Dashboard component
- [ ] Display "Last backup: X hours ago"
- [ ] Calculate total space saved
- [ ] Display success rate (%)
- [ ] Show next scheduled backup

### restore (restoration)
- [ ] Create BackupHistory component
- [ ] List available backups
- [ ] Implement command `restore_backup`
- [ ] Decompress + decrypt
- [ ] Allow choosing restore destination
- [ ] Test complete restore

### integrity verification
- [ ] Generate SHA-256 checksum when creating backup
- [ ] Save checksum in metadata
- [ ] Verify integrity when restoring
- [ ] Alert if file is corrupted

### final tests
- [ ] Test backup of large folder (>1GB)
- [ ] Test scheduled backup for 24h
- [ ] Test complete restore
- [ ] Test encryption + restore
- [ ] Validate 0 errors in 10 consecutive backups

### build and distribution
- [ ] Configure app icon
- [ ] Generate production build: `pnpm tauri build`
- [ ] Test .dmg installer on macOS
- [ ] Create README with usage instructions
- [ ] Document how to do manual restore

**Week 4 Deliverable:** Complete MVP, tested and ready to use

---

## final delivery checklist

### core features
- [ ] Multiple folder selection
- [ ] One-click manual backup
- [ ] Scheduled automatic backup
- [ ] Functional zstd compression
- [ ] AES-256 encryption (optional)
- [ ] Point-in-time restore
- [ ] Native notifications
- [ ] Dashboard with metrics

### quality
- [ ] 0 crashes in 24h tests
- [ ] Backups validated with checksum
- [ ] Detailed logs for debug
- [ ] Error handling (no panics)
- [ ] Responsive UI (<100ms for actions)

### documentation
- [ ] README with screenshots
- [ ] Installation guide
- [ ] Basic usage guide
- [ ] FAQ (frequently asked questions)
- [ ] How to report bugs

### performance
- [ ] App starts in <500ms
- [ ] Compression 2x faster than native zip
- [ ] Final binary <5 MB
- [ ] RAM usage <100 MB when idle
- [ ] 1GB backup completes in <2 min

---

## next steps (post-MVP)

### phase 2 (optional)
- [ ] Support for exclusion patterns (*.log, node_modules)
- [ ] Incremental backup (only changes)
- [ ] Periodic automatic integrity verification
- [ ] Configuration export
- [ ] Dark/light themes
- [ ] Support for multiple destinations
- [ ] Optional cloud synchronization
- [ ] Backup versioning (keep last N)

### phase 3 (future)
- [ ] Linux app
- [ ] Windows app
- [ ] CLI mode (no GUI)
- [ ] Differential compression
- [ ] File deduplication
- [ ] WebUI for remote management

---

## risks and mitigations

| Risk | Probability | Impact | Mitigation |
|------|------------|---------|-----------|
| launchd complex | Medium | High | Use existing Rust library |
| Very slow backup | Low | Medium | zstd with level 3 (balanced) |
| Encryption bugs | Low | High | Use audited library (ring) |
| Confusing UI | Medium | Medium | Test with real users |
| Data corruption | Low | Critical | Checksums + extensive tests |

---

## success metrics

**MVP is successful if:**
- ✅ User configures backup in <3 minutes
- ✅ Automatic backup runs without failures for 7 days
- ✅ Restore works 100% of the time
- ✅ App is lighter than Time Machine/Electron apps
- ✅ 0 critical bugs reported

**Start:** Week 1, Day 1
**Finish:** Week 4, Day 7
**Review:** Daily (checklist)
