# InLocker - Developer Guide

Quick reference for daily development and debugging.

---

## Essential Commands

### Development
```bash
# Run app with hot reload
pnpm tauri dev

# Build production app
pnpm tauri build

# Type check (no emit)
tsc --noEmit
```

### Rust Backend
```bash
# Check + test + lint (run from project root)
cd src-tauri && cargo check && cargo test && cargo clippy

# Format code
cd src-tauri && cargo fmt
```

### Dependencies
```bash
# Frontend
pnpm install

# Rust
cd src-tauri && cargo update
```

---

## Debugging

### Frontend Logs (Browser Console)

**Open:** `Cmd + Option + I` → Console tab

**What you'll see:**
```
[BackupList] Starting backup for config: backup-1234567890
[BackupList] Backup result: { success: true, ... }
```

Errors appear in red.

### Backend Logs (Terminal)

**Where:** Terminal where you ran `pnpm tauri dev`

**Logs enabled by default** - just run the app normally.

**Example output:**
```
[2025-11-01T14:15:20Z INFO  inlocker_lib] 🔵 Starting INCREMENTAL backup
[2025-11-01T14:15:20Z INFO  inlocker_lib] 📂 Source: /Users/blc/Dev
[2025-11-01T14:15:23Z INFO  inlocker_lib] ✅ Compressed to 1.20 MB (73.3% compression)
[2025-11-01T14:15:23Z INFO  inlocker_lib] 🎉 Backup completed successfully in 3s
```

Errors show as:
```
[2025-11-01T14:16:05Z ERROR inlocker_lib] Backup failed: No such file or directory
```

### UI Feedback

**During backup:**
- Blue spinner + "Backup in progress..."
- "Run Backup" button disabled

**On success:**
- Green box with checkmark
- Details: "123 files, 4.5 MB → 1.2 MB (73.3% compression)"

**On error:**
- Red box with X icon
- Alert popup with error details

### launchd Scheduled Backup Logs

```bash
# View logs
tail -f /tmp/inlocker-backup-<config_id>.log

# View errors
tail -f /tmp/inlocker-backup-<config_id>.err
```

---

## Troubleshooting

### Backup Won't Start

**Check browser console:**
```
[BackupList] Backup error: Config not found
```

**Verify paths exist and permissions:**
```bash
ls -la /path/to/source/folder
touch /path/to/destination/test.txt && rm /path/to/destination/test.txt
```

### Permission Denied

**Grant access:**
1. System Settings → Privacy & Security → Files and Folders
2. Find InLocker
3. Enable required folders

### Schedule Not Working

```bash
# Verify .plist created
ls -la ~/Library/LaunchAgents/com.inlocker.backup.*

# Check if loaded
launchctl list | grep inlocker

# Reload manually
launchctl unload ~/Library/LaunchAgents/com.inlocker.backup.*.plist
launchctl load ~/Library/LaunchAgents/com.inlocker.backup.*.plist
```

### Restore Can't Find Backups

```bash
# Check files exist
ls -lh /path/to/destination/*.tar.zst

# Files must match format: backup_incr_20251101_140530.tar.zst
```

---

## Testing Backup Flow

1. **Open DevTools** (`Cmd+Option+I`)
2. **Create test config** with small source folder
3. **Click "Run Backup"**
4. **Observe:**
   - Browser console: `[BackupList] Starting backup...`
   - Terminal: `INFO Running backup for: ...`
   - UI: Blue spinner → Green success box
5. **Verify file created:**
   ```bash
   ls -lh /path/to/destination/
   # Should show: backup_incr_YYYYMMDD_HHMMSS.tar.zst
   ```
6. **Test restore** and verify files

---

## Quick Health Check

- [ ] App opens without errors
- [ ] No red errors in browser console
- [ ] Can create backup config
- [ ] "Run Backup" creates .tar.zst file
- [ ] Can restore backup successfully
- [ ] Schedule appears in `launchctl list | grep inlocker`

---

## Build Output Locations

```bash
# App bundle
src-tauri/target/release/bundle/macos/InLocker.app

# DMG installer
src-tauri/target/release/bundle/dmg/InLocker_0.1.0_aarch64.dmg

# Open app (for testing)
open src-tauri/target/release/bundle/macos/InLocker.app
```

---

## Icon Customization

```bash
# Replace icon (1024x1024 PNG)
# src-tauri/icons/icon.png

# Generate all sizes
pnpm tauri icon src-tauri/icons/icon.png

# Rebuild to apply
pnpm tauri build
```
