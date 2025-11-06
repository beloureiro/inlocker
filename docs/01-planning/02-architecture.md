# InLocker - simple architecture

## system overview

```
┌─────────────────────────────────────────────────────────────┐
│                    USER (Interface)                          │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                  TAURI FRONTEND (React)                      │
│  ┌────────────┐  ┌─────────────┐  ┌──────────────┐         │
│  │  Folder    │  │  Schedule   │  │  Dashboard   │         │
│  │  Selector  │  │  Config     │  │  Status      │         │
│  └────────────┘  └─────────────┘  └──────────────┘         │
└──────────────────────┬──────────────────────────────────────┘
                       │ (IPC - Inter-Process Communication)
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                   TAURI BACKEND (Rust)                       │
│                                                               │
│  ┌─────────────┐  ┌──────────────┐  ┌────────────────┐     │
│  │  Scheduler  │──│  Backup      │──│  Compression   │     │
│  │  Engine     │  │  Coordinator │  │  Engine (zstd) │     │
│  └─────────────┘  └──────────────┘  └────────────────┘     │
│                           │                                  │
│  ┌─────────────┐  ┌──────▼───────┐  ┌────────────────┐     │
│  │  Crypto     │  │  File System │  │  Integrity     │     │
│  │  (AES-256)  │  │  Manager     │  │  Checker       │     │
│  └─────────────┘  └──────────────┘  └────────────────┘     │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                    OPERATING SYSTEM                          │
│  ┌────────────┐  ┌──────────────┐  ┌────────────────┐      │
│  │  launchd   │  │  File System │  │  Notifications │      │
│  │ (scheduler)│  │   (macOS)    │  │   (native)     │      │
│  └────────────┘  └──────────────┘  └────────────────┘      │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│              LOCAL STORAGE (Backups)                         │
│                                                               │
│  /Users/you/InLocker-Backups/                                │
│    ├── web-project_2025-11-01_14-30.zst.enc                 │
│    ├── documents_2025-11-01_14-30.zst.enc                   │
│    └── photos_2025-11-01_14-30.zst.enc                      │
└─────────────────────────────────────────────────────────────┘
```

## what the app delivers

### input
```
USER PROVIDES:
├── Folders to backup
│   └── Ex: /Users/you/Projects, /Users/you/Documents
├── Execution schedules
│   └── Ex: Daily at 14:00, Weekly on Fridays at 18:00
└── Backup destination
    └── Ex: /Users/you/InLocker-Backups
```

### processing
```
APP EXECUTES:
1. Monitors scheduled times
2. When time arrives:
   ├── Scans files from source folder
   ├── Streams TAR archive creation (no memory loading)
   ├── Compresses with zstd in streaming mode (fast + efficient)
   ├── Encrypts with AES-256 (optional, chunked processing)
   ├── Generates checksum (integrity)
   └── Writes directly to destination (pipeline architecture)
3. Notifies success or error
4. Records in history

Note: Streaming architecture processes data in ~1MB chunks,
enabling backups larger than available RAM (200GB+ on 8GB systems)
```

### output
```
USER RECEIVES:
├── Automatic compressed backups
│   └── web-project_2025-11-01_14-30.zst.enc (50% smaller)
├── Completion notification
│   └── "Backup of 'Projects' completed successfully"
├── Dashboard with metrics
│   ├── Last backup: 1 hour ago
│   ├── Space saved: 2.3 GB
│   ├── Total backups: 45
│   └── Success rate: 100%
└── Backup history
    └── Facilitates point-in-time restore
```

## main components

### 1. scheduler engine
- Manages schedules via launchd (macOS)
- Monitors configured times
- Triggers backups automatically

### 2. backup coordinator
- Orchestrates backup process
- Manages task queue
- Coordinates compression and encryption

### 3. compression engine
- Uses zstd (Zstandard) with streaming mode
- Better than zip: faster + smaller size
- Incremental mode for recurring backups
- Pipeline architecture: TAR → zstd → disk (no intermediate buffers)
- Handles files larger than available RAM

### 4. crypto module
- AES-256-GCM (optional)
- Password derived with Argon2
- Zero-knowledge (only you have the key)

### 5. integrity checker
- Generates SHA-256 checksums
- Validates integrity after backup
- Alerts about corruption

### 6. ui dashboard
- Modern React interface
- Drag-and-drop for folders
- Visual schedule configuration
- Real-time metrics

## simplified backup flow

```
[Schedule Time Reached]
    │
    ▼
[Scheduler triggers event]
    │
    ▼
[Backup Coordinator starts]
    │
    ├──► [Reads files from source folder]
    │
    ├──► [Compresses with zstd]
    │
    ├──► [Encrypts (if enabled)]
    │
    ├──► [Generates checksum]
    │
    ├──► [Saves to destination]
    │
    ├──► [Verifies integrity]
    │
    └──► [Sends notification + updates UI]
```

## technologies by layer

| Layer | Technology | Reason |
|-------|-----------|--------|
| **UI** | React + TypeScript | Productivity and type-safety |
| **Framework** | Tauri 2.0 | Lightweight, fast, secure |
| **Backend** | Rust | Performance + reliability |
| **Compression** | zstd | Best ratio + speed |
| **Encryption** | AES-256-GCM | Industry standard |
| **Scheduler** | launchd | macOS native |
| **Storage** | File System | Local, no dependencies |
