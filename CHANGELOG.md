# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.3] - 2025-11-30

### Bug Fixes

- **ui:** improve Test Now button and minor UI fixes

  - Add spinner feedback while opening progress window
  - Show disabled state with tooltip for non-scheduled backups
  - Make Test Now button visible on all cards for consistency
  - Minor UI adjustments in FolderSelector and App

## [0.7.2] - 2025-11-30

### Bug Fixes

- **backup:** improve progress tracking and completed event timing

  - Add original_size and compressed_size to BackupProgress struct
  - Emit size data during compression stages
  - Move completed event emission before slow manifest scan
  - Fix progress window appearing stuck on checksum stage

## [0.7.1] - 2025-11-30

### Bug Fixes

- **scheduling:** replace React progress with pure HTML window

  - Replace ScheduledBackupProgress React component with progress.html
  - Add elapsed timer, original/compressed size display
  - Use app logo instead of generic SVG icon
  - Respect auto-close preference setting
  - Add pulse animation for pending compressed size
  - Fix window capabilities for scheduled-progress

## [0.7.0] - 2025-11-30

### Features

- **settings:** add app preferences with auto-close option

  - Add AppPreferences struct with auto_close_progress_window setting
  - Add load_preferences and save_preferences commands
  - Create usePreferencesStore for frontend state management
  - Add PreferencesModal component with toggle
  - Add settings gear icon in Layout header

## [0.6.5] - 2025-11-23

### Bug Fixes

- **scheduling:** implement window-ready event pattern for blank screen

  - Add window-ready event emission in App.tsx after React render (100ms delay)
  - Add Listener trait and event listeners in lib.rs
  - Backend waits for frontend ready event before showing windows
  - Prevents showing window before React finishes rendering

## [0.6.4] - 2025-11-22

### Bug Fixes

- **ui:** remove light theme causing file picker to appear in light mode

  - Removed theme: Light from scheduled-progress window
  - macOS theme is app-wide, not per-window
  - Kept theme: Dark on main window
  - docs(roadmap): update Phase 3 status to COMPLETE and MVP to NEAR PRODUCTION

## [0.6.3] - 2025-11-22

### Bug Fixes

- **scheduling:** implement Test Now button and backup validation

  - Test Now improvements:
  - Open scheduled-progress window instead of launchctl kickstart
  - Add CLI arg detection in single-instance plugin
  - Add window routing and event listeners
  - Redesign progress UI with dark theme
  - Backup validation:
  - Add verify_backup_exists command
  - Validate file existence before showing stats
  - Prevent stale metadata display

## [0.6.2] - 2025-11-21

### Bug Fixes

- **scheduling:** add separate windows for scheduled backups (untested)

## [0.6.1] - 2025-11-21

### Bug Fixes

- **scheduling:** add separate windows for scheduled backups (untested)

## [0.6.0] - 2025-11-07

### Features

- **ui:** simplify encrypted backup password workflow

  - Remove duplicate password fields from configuration modal:
  - Eliminate password and confirm password inputs
  - Show clear information that password is prompted on execution only
  - Improve security by never storing passwords in configurations
  - Enhance user experience with simpler configuration flow
  - Users now enter password once per backup execution instead of
  - storing it in configuration, improving both security and UX.

## [0.5.4] - 2025-11-07

### Bug Fixes

- **backup:** resolve password prompt timing and timer accuracy issues

  - Password prompt timing:
  - Move password validation before emit_progress to prevent UI progress bar
  - appearing when backup will fail immediately due to missing password
  - Timer accuracy:
  - Sync frontend timer with backend timestamp (started_at field)
  - Frontend now displays actual processing time matching backend execution
  - Eliminates drift between UI timer and real backup duration
  - Config isolation:
  - Update Zustand store in-place instead of full reload
  - Preserves running backup states when editing other configurations
  - Prevents UI disruption during parallel backup operations

## [0.5.3] - 2025-11-07

### Bug Fixes

- **backup:** optimize checksum calculation with 1MB buffer

  - Increase buffer size from 8KB to 1MB in calculate_checksum function.
  - This reduces checksum time for large files (30GB) from 25min to ~2min,
  - achieving 12x performance improvement.

## [0.5.2] - 2025-11-06

### Bug Fixes

- **crypto:** add password prompt workaround for encrypted backups

  - Add React modal for password input (replaces blocked browser dialogs)
  - Modify run_backup_now command to accept password parameter
  - Update roadmap documentation to reflect production blockers
  - Mark Phase 3 as FAILED (2 critical issues)
  - Document encryption limitation: manual backups only (scheduled not supported)
  - Document scheduling limitation: launchd not functional

## [0.5.1] - 2025-11-06

### Bug Fixes

- **build:** resolve DMG bundling failure on macOS

  - Change bundle identifier from com.inlocker.app to com.inlocker.backup
  - (fixes conflict with .app extension warning)
  - Update bundle targets from 'all' string to ['app', 'dmg'] array
  - (resolves intermittent bundle_dmg.sh failures)
  - Add macOS minimumSystemVersion configuration (10.13)
  - Ensures consistent and automatic DMG generation without errors

## [0.5.0] - 2025-11-06

### Features

- **backup:** add run all backups in parallel functionality

  - Implement handleRunAllBackups to execute multiple backups simultaneously
  - Filter and validate enabled configs before execution
  - Add user confirmation dialog with backup list preview
  - Remove deprecated handleRestore function (replaced by RestoreSelector)

## [0.4.0] - 2025-11-06

### Features

- **restore:** implement progress tracking and cancellation support

  - Add restore progress events for all stages (verifying, decrypting, decompressing, extracting)
  - Implement cancellation flag checks throughout restore process
  - Add AppHandle parameter for emitting progress events to frontend
  - Document technical limitations (decryption/decompression cannot be interrupted)
  - Add intelligent cancellation checks before and after blocking operations

## [0.3.0] - 2025-11-06

### Features

- **restore:** add RestoreSelector component with full UX

  - Implement file and folder selection dialogs with spinner feedback
  - Add restore operation with proper parameter serialization
  - Create success result box showing files count, duration, and destination
  - Add collapsible cancellation behavior info section
  - Display real-time progress with stage-specific messages
  - Integrate RestoreSelector into main App

## [0.2.1] - 2025-11-05

### Bug Fixes

- **ui:** prevent config reload from hiding parallel backup progress

  - Replace debouncedLoadConfigs with smartLoadConfigs
  - Only reload configs when NO backups are running (size === 0)
  - Prevents re-renders from affecting running backups UI state
  - Fixes issue where second backup completion hides first backup progress
  - Defers config reload until all parallel backups complete

## [0.2.0] - 2025-11-05

### Features

- **changelog:** implement InLog automatic changelog system

  - Install and configure Husky 9.1.7 for git hooks
  - Create update-changelog.mjs script with semantic versioning
  - Add post-commit hook for automatic CHANGELOG updates
  - Support conventional commits (feat, fix, docs, chore)
  - Auto-increment version in package.json (MAJOR.MINOR.PATCH)
  - Generate CHANGELOG.md entries automatically
  - Prevent infinite loops with .changelog-lock flag file
  - Add comprehensive InLog system guide documentation
  - Populate CHANGELOG.md with complete git history

## [0.1.1] - 2025-11-05

### Bug Fixes

- **restore:** add support for encrypted backups and fix parallel UI

  - Update list_backups() to include .tar.zst.enc files
  - Add password prompt in UI for encrypted restore (BackupList.tsx:215-225)
  - Fix parallel backups UI display with debounced loadConfigs()
  - Move config reload to finally block for proper state management
  - Prevent re-render issues when multiple backups complete simultaneously

## [0.1.0] - 2025-11-02

### Features

- **cancellation:** Implement backup cancellation with atomic flags and cleanup
  - Cancellation mechanism using Arc<AtomicBool> for thread-safe operation
  - New cancel_backup command registered in Tauri backend
  - Cancellation checks during file scanning, copy, TAR creation, compression, and encryption
  - Automatic cleanup of partial backup files on cancellation
  - Frontend integration with cancel button
  - Updated 46 test calls across 6 test files
  - Tested with 709k files (24GB) - immediate cancellation with automatic cleanup

- **testing:** Add comprehensive test suite and real-time progress tracking UI
  - NEW: performance_tests.rs with 4 performance validation tests (backup speed, compression ratio, large file sets, incremental performance)
  - Updated critical_backup_tests.rs, critical_security_tests.rs, adversarial_tests.rs
  - Total: 78 tests (all passing, ~75% coverage)
  - Real-time progress bar during TAR creation (updates every 100 files)
  - Determinate progress (0-100%) for file counting stage
  - Indeterminate progress (animated barberpole) for compression/encryption
  - Full-width progress bar spanning entire card
  - Added current and total fields to BackupProgress struct
  - Progress stages: scanning → creating_tar → compressing → encrypting → writing → checksum

- **encryption:** Add AES-256-GCM encryption and comprehensive test suite
  - Implement optional backup encryption with Argon2id key derivation
  - Encrypted backups use .tar.zst.enc format with embedded metadata
  - Replace weak size+mtime checksums with SHA-256 content hashing
  - Add constant-time comparison to prevent timing attacks
  - 73 tests total (71 passing, 2 ignored for disk-full scenarios)
  - 100% pass rate on active tests
  - Crypto: 31 tests (encryption, key derivation, authentication)
  - Security: 31 tests (vulnerabilities, attacks, integrity)
  - Core: 18 tests (backup/restore workflows)
  - Edge cases: 14 tests (filesystem edge cases)
  - Add crypto.rs module and update backup.rs
  - Auto-format cron expressions in configuration modal

- **progress:** Implement real-time backup progress UI and major improvements
  - Add BackupProgress struct with stage/message/details
  - Emit progress events during backup (scanning, tar, compress, write, checksum)
  - Add event listeners in React to receive live updates
  - Display step-by-step progress with details in UI
  - Shows: "Scanning files (24438.7 MB)", "Compressing with zstd (25006.2 MB)", etc
  - Remove redundant labels, compact spacing (p-6 → p-4, mb-4 → mb-3)
  - Add backup size info to cards (original → compressed with %)
  - Add file count display and live elapsed timer
  - Show total duration in final result
  - Initialize env_logger for automatic Rust logging
  - Add detailed step-by-step logs with emojis
  - Store backup size metadata (original_size, compressed_size, files_count)
  - Change default times from 2 AM → 8 PM (laptop-friendly)

- **core:** Implement Phase 1: Foundation and configuration system
  - Add Tauri commands for folder selection and config management
  - Implement BackupConfig and BackupJob type system
  - Add JSON persistence for backup configurations
  - Integrate tauri-plugin-dialog for native folder picker
  - Set up modular Rust architecture (commands, types, lib)
  - Build React UI with Zustand state management
  - Create Layout component with emerald green branding
  - Add FolderSelector for intuitive folder selection
  - Implement BackupList to display saved configurations
  - Configure dark titlebar for seamless macOS integration
  - Apply emerald green color scheme throughout
  - Add lock icon to reinforce security branding

- **environment:** Setup complete Tauri + React development environment
  - Rust 1.91.0, Tauri CLI 2.9.2, Node.js 23.11.1 with pnpm 10.19.0
  - React 19.2 + TypeScript 5.8.3
  - Vite 7.1.12 build tool
  - TailwindCSS 3.4.18 for styling
  - Zustand 5.0.8 for state management
  - lucide-react for icons, date-fns 4.1.0 for date utilities
  - Tauri 2.9.2 framework with tokio, zstd, ring, notify, chrono
  - Tauri plugins: opener, notification
  - Frontend build successful, hot-reload working, all TypeScript checks passing

- **project:** Initial project structure and documentation
  - Complete project foundation for InLocker backup app
  - Complete documentation in English (value proposition, architecture, tech stack, roadmap)
  - Tech stack defined with latest 2025/2026 versions (Tauri 2.8.5, React 19.2, Rust 1.91, Node 24 LTS)
  - Development roadmap with 4-week MVP plan
  - Quick start guide for developers
  - Git configuration files

### Bug Fixes

- **schedule:** Fix schedule UI: follow cron tree order and improve UX
  - Clock icon badge now properly disappears when schedule removed
  - Check both enabled and cron_expression fields
  - Prevents empty schedule objects from showing badge
  - Cron expression parser follows visual tree order
  - Input '2005' now correctly means minute=20, hour=05
  - Follows standard cron format (minute hour day month weekday)
  - Improved typing experience - no real-time parsing while typing
  - Format applied only when field loses focus (onBlur)
  - Users can freely type, delete, and correct without interference
  - Auto-completes with asterisks only on save

### Documentation

- **docs:** Update documentation to match actual implementation
  - Aligned all documentation with the implemented setup
  - Tauri: 2.8.5 → 2.9.2, TypeScript: 5.9 → 5.8.3
  - Node.js: v24 LTS → v23.11.1, pnpm: 10.20+ → 10.19.0
  - Vite: 7.0 → 7.1.12
  - Updated Cargo.toml to show tauri-plugin architecture
  - Added all actual dependencies (lucide-react, @types/node, autoprefixer, postcss)
  - Marked Week 1 environment configuration and initialization as complete
  - Added actual versions and GitHub repository link
  - Updated README with current phase and progress
  - Add comprehensive DEBUGGING.md with log examples
  - Update roadmap to Phase 4 COMPLETE (95% MVP)
  - Split into quickstart, dev guide, user guide, testing strategy
  - Include testing strategy documentation and coverage reports
  - Enhanced test documentation (README.md, TEST_COVERAGE_REPORT.md)
