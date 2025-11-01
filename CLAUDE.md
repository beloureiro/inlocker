# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**InLocker** is a native macOS backup application built with Tauri 2.9.2, Rust 1.91, and React 19.2. It performs automatic, compressed (zstd), and optionally encrypted (AES-256) backups of user folders, all running locally on macOS.

**Current Status:** Week 1 development - basic Tauri + React app with IPC communication established.

## Development Commands

### Running the App
```bash
# Development mode with hot reload
pnpm tauri dev

# Production build
pnpm tauri build
```

### Frontend
```bash
# Install dependencies
pnpm install

# Build frontend only
pnpm build

# TypeScript type checking
tsc --noEmit
```

### Backend (Rust)
```bash
# Check Rust code for errors
cd src-tauri && cargo check

# Run Rust tests
cd src-tauri && cargo test

# Format Rust code
cd src-tauri && cargo fmt

# Lint Rust code
cd src-tauri && cargo clippy
```

## Architecture Overview

InLocker follows a **layered Tauri architecture** with clear separation between frontend (React/TypeScript) and backend (Rust):

### Frontend Layer (React + TypeScript)
- **Entry**: `src/main.tsx` bootstraps React app
- **Main Component**: `src/App.tsx` - currently testing IPC communication
- **State Management**: Zustand (planned structure in `src/ui/store/`)
- **Components**: Planned in `src/ui/components/` (FolderSelector, Dashboard, etc.)
- **Services**: `src/services/tauri.ts` wraps Tauri API calls

### Backend Layer (Rust)
- **Entry**: `src-tauri/src/lib.rs` - initializes Tauri app and registers commands
- **Commands**: `src-tauri/src/commands.rs` - Tauri IPC commands exposed to frontend
  - `select_folder()` - folder picker (placeholder implementation)
  - `save_config()` - persist backup configuration
  - `load_configs()` - retrieve all backup configs
  - `delete_config()` - remove backup config
  - `run_backup_now()` - execute backup immediately (Week 2 implementation)
- **Types**: `src-tauri/src/types.rs` - shared data structures
  - `BackupConfig` - folder backup configuration
  - `BackupJob` - single backup execution record
  - `BackupStatus` - job status enum (Pending, Running, Completed, Failed)
  - `BackupResult` - backup operation result
- **State**: `AppState` in `commands.rs` manages in-memory config storage via `Mutex<Vec<BackupConfig>>`

### IPC Communication Pattern
Frontend calls Rust commands via Tauri's `invoke()`:
```typescript
import { invoke } from "@tauri-apps/api/core";
const result = await invoke<ReturnType>("command_name", { param: value });
```

Backend implements commands with `#[tauri::command]` macro and registers them in `lib.rs`.

## Key Technologies

- **Tauri 2.9.2**: Native app framework (WebKit on macOS)
- **Rust 1.91.0**: Backend logic, file operations, compression
- **React 19.2 + TypeScript 5.8**: UI layer
- **Vite 7.1.12**: Build tool (dev server on port 1420)
- **pnpm 10.19.0**: Package manager
- **Zustand 5.0**: State management (planned)
- **TailwindCSS 3.4**: Styling
- **zstd 0.13**: Compression library (Week 2 integration)
- **ring 0.17**: Cryptography for AES-256 (Week 3 integration)
- **tokio 1.42**: Async runtime for Rust
- **chrono 0.4**: Date/time handling
- **notify 7.0**: File system watcher (future use)

## Implementation Roadmap

The project follows a 4-week incremental plan (see `docs/04-roadmap.md` for full details):

**Week 1 (Current):** Basic backend commands + folder selection UI
**Week 2:** Backup core with zstd compression + manual backup execution
**Week 3:** Scheduler with launchd integration + optional encryption
**Week 4:** Dashboard, restore functionality, integrity checks, polish

## Important Implementation Notes

### Rust Backend Development
- All Tauri commands must be **async** and return `Result<T, String>`
- Register new commands in `lib.rs` via `invoke_handler![]` macro
- Use `AppState` for in-memory state; persist to JSON files for durability
- Commands are defined in `src-tauri/src/commands.rs` and imported in `lib.rs`

### macOS-Specific Integration
- **Scheduler**: Will use macOS `launchd` (Week 3) - generate `.plist` files for automatic backups
- **Notifications**: Use `tauri-plugin-notification` for native macOS notifications
- **File Dialogs**: Use `tauri-plugin-dialog` for folder/file picking (not yet implemented)

### Security Considerations
- Encryption is **optional** (user choice) with AES-256-GCM
- Password derivation will use Argon2
- Zero-knowledge design: encryption keys stay local
- Backups stored locally by default (no cloud sync in MVP)

### Data Flow for Backup Operation
1. User selects folder via UI → calls `select_folder()` command
2. User configures schedule/destination → calls `save_config()`
3. User clicks "Backup Now" or scheduler triggers → calls `run_backup_now()`
4. Backend reads files → compresses with zstd → optionally encrypts → saves to destination
5. Backend generates SHA-256 checksum for integrity verification
6. Result returned to frontend → UI updates with status/metrics

### File Naming Convention
Backups use timestamp format: `{folder-name}_{YYYY-MM-DD_HH-MM}.zst.enc`
Example: `web-project_2025-11-01_14-30.zst.enc`

## Configuration Files

- `src-tauri/tauri.conf.json`: Tauri app configuration (window size, bundle settings, identifier: `com.inlocker.app`)
- `package.json`: Frontend dependencies and scripts
- `src-tauri/Cargo.toml`: Rust dependencies (lib name: `inlocker_lib`)
- `vite.config.ts`: Vite dev server (port 1420, ignores `src-tauri/` watch)
- `tsconfig.json`: TypeScript strict mode enabled

## Testing Strategy

- **Rust**: `cargo test` for unit tests, `cargo tarpaulin` for coverage
- **Frontend**: Vitest + Testing Library (planned)
- **Integration**: Test complete backup → restore cycle with various folder sizes
- **Performance targets**: App start <500ms, 1GB backup <2min, binary <5MB

## Documentation Structure

Comprehensive docs in `/docs`:
1. `01-value-proposition.md` - Problem, target users, competitive advantages
2. `02-architecture.md` - System diagram, data flow, component breakdown
3. `03-tech-stack.md` - Technology justifications and version details
4. `04-roadmap.md` - 4-week implementation plan with checklists

Also see `quickstart.md` for development environment setup.

## Common Patterns

### Adding a New Tauri Command
1. Define command function in `src-tauri/src/commands.rs`:
   ```rust
   #[tauri::command]
   pub async fn my_command(param: String) -> Result<MyType, String> {
       // implementation
   }
   ```
2. Register in `src-tauri/src/lib.rs`:
   ```rust
   .invoke_handler(tauri::generate_handler![
       existing_commands,
       my_command,  // add here
   ])
   ```
3. Call from frontend:
   ```typescript
   const result = await invoke<MyType>("my_command", { param: "value" });
   ```

### State Management Pattern
- Use `State<'_, AppState>` parameter in Tauri commands to access shared state
- Lock mutex, modify data, unlock: `state.configs.lock().map_err(|e| e.to_string())?`
- TODO items indicate where persistence to JSON files is needed

## Git Workflow

- Main branch: `main`
- Recent commits show setup progress: environment setup → Tauri init → IPC testing
- Modified files: `src-tauri/src/lib.rs`, `src/App.tsx`
- Untracked: `src-tauri/src/commands.rs`, `src-tauri/src/types.rs` (need to be added)
