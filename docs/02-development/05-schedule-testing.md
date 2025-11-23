# Visual Schedule Testing Guide

This guide shows you how to test the scheduling system **visually** without running terminal commands.

## Step 1: Start Dev Mode

Execute this command (clears cache and starts app):
```bash
rm -rf dist node_modules/.vite && pnpm tauri dev
```

**What to see:**
- App window opens
- Dev console appears (keep it visible!)

---

## Step 2: Configure Schedule (2 minutes from now)

1. Click **Settings** button on any existing backup
2. In the modal:
   - Select **Schedule: Daily**
   - Set **Hour** to current hour (e.g., if now is 15:30, set 15)
   - Set **Minute** to 2 minutes from now (e.g., if now is 15:30, set 32)
3. Click **Save Changes**

**What to see in CONSOLE:**
```
============================================================
🔧 REGISTERING SCHEDULE
============================================================
Config ID: dev-backup-abc123
Preset: daily
Cron Expression: 32 15 * * *
Calling backend register_schedule...
✅ Schedule registered successfully!

📋 AUTOMATIC VERIFICATION:
   1. .plist created in ~/Library/LaunchAgents/
   2. Agent loaded in launchd
   3. Logs will be saved to ~/Library/Logs/InLocker/

🔍 Manual verification commands:
   ls -la ~/Library/LaunchAgents/com.inlocker*
   launchctl list | grep inlocker
============================================================
```

**What to see in UI:**
- Alert popup: "Schedule configured successfully!"
- Backup card shows clock icon (⏰) badge
- Schedule info shows "Daily" in card

---

## Step 3: Wait and Watch (Auto-Execute)

**Set a timer for 2 minutes and watch the UI.**

When the scheduled time arrives, you'll see **automatically**:

### In the UI (Backup Card):
1. Card expands automatically
2. Blue progress bar appears
3. Spinner icon shows
4. Message: "Starting backup..." → "Creating archive..." → etc.
5. Timer shows elapsed time (0:01, 0:02, ...)

### In the Console:
```
[BackupList] Progress event: {config_id: "dev-backup-abc123", stage: "starting", message: "Starting backup..."}
[BackupList] Progress event: {stage: "archiving", message: "Creating archive...", current: 10, total: 100}
...
[BackupList] Backup result: {success: true, message: "Backup completed successfully"}
```

### After Completion:
- Green success box appears
- Shows: "Backup Successful" with duration
- Card shows last backup timestamp

---

## Step 4: Verify Everything Worked

### Visual Verification (No Terminal Needed):

1. **Click "Logs" button** on the backup card
   - Finder opens showing `~/Library/Logs/InLocker/`
   - You'll see file: `scheduled-{config-id}.log`
   - Open it to see execution logs

2. **Check backup file created**:
   - Navigate to destination folder (shown in card: "To: /path/to/dest")
   - You'll see new `.zst` file with timestamp

3. **Test manual trigger**:
   - Click **"Test Now"** button
   - Should see alert: "Schedule test executed successfully"
   - Another backup starts immediately

---

## Troubleshooting

### If Nothing Happens After 2 Minutes:

1. **Check Console for Errors**:
   - Look for red error messages
   - Check if registration actually succeeded

2. **Use Test Now Button**:
   - Click "Test Now" to force immediate execution
   - If this works, schedule is registered correctly
   - If this fails, check error message

3. **Check Logs Folder**:
   - Click "Logs" button
   - Check if `.err` file exists with errors

---

## Expected Timeline

```
00:00 - Save schedule (see console logs + alert)
00:01 - Schedule registered (clock icon appears)
02:00 - Backup starts automatically (UI shows progress)
02:30 - Backup completes (green success box)
```

---

## What You Should NOT Need To Do

- ❌ Open Terminal
- ❌ Run `ls` or `launchctl` commands
- ❌ Navigate to LaunchAgents folder manually
- ❌ Check .plist files manually

**Everything is visible in the UI and console!**

---

## Quick Test (30 seconds)

If you don't want to wait 2 minutes:

1. Configure schedule
2. Immediately click **"Test Now"** button
3. Watch backup execute in real-time
4. Confirms system works without waiting

---

## Known Issues / Bugs

### 🔴 Bug: Tela Branca Aparece

**Status:** NÃO RESOLVIDO (2025-11-23)

**Descrição:**
- [ ] Janela abre mas mostra apenas tela branca
- [ ] UI não carrega (ScheduledBackupProgress.tsx)

**Solução:**
- [ ] https://github.com/tauri-apps/tauri/issues/9393
- [ ] usar on_page_load(PageLoadEvent::Finished) antes de show()

**Arquivo de Bug Completo:** `/Users/blc/Dev/Apps/InLocker/docs/04-bugs/002-scheduling-system-not-working.md`

---

**Last Updated:** 2025-11-23
**Status:** [ ] Bug ativo - tela branca
