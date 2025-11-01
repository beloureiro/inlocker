# InLocker - User Guide

**Simple, automatic, and secure backups for macOS**

---

## Getting Started

### For Developers (Running from source)

If you're running InLocker from source code, follow these steps:

#### Prerequisites
- macOS 12.0 or later
- Terminal access

#### Installation

1. **Clone the repository** (if you haven't already):
   ```bash
   git clone https://github.com/beloureiro/inlocker.git
   cd inlocker
   ```

2. **Install dependencies**:
   ```bash
   pnpm install
   ```

3. **Run the app**:
   ```bash
   pnpm tauri dev
   ```

The InLocker app will open in a new window.

---

## Using InLocker

### Creating Your First Backup

1. **Launch InLocker**
   - The app opens with a clean, dark interface
   - You'll see "Add New Backup" section at the top

2. **Select Source Folder**
   - Click the **Browse** button next to "Source Folder"
   - Choose the folder you want to backup (e.g., Documents, Projects, Photos)
   - The path will appear in the text field

3. **Select Destination Folder**
   - Click the **Browse** button next to "Destination Folder"
   - Choose where you want to save your backups (e.g., external drive, another folder)
   - The path will appear in the text field

4. **Save Configuration**
   - Click the green **Save Backup Configuration** button
   - Your backup is now saved and will appear in the "Saved Backups" list below

### Managing Your Backups

#### View Saved Backups
- All your backup configurations appear in the list below the form
- Each backup shows:
  - Backup name (automatically generated from folder name)
  - Source path (where files are backed up from)
  - Destination path (where backups are saved to)
  - Status badges (Enabled/Disabled, Encrypted)

#### Delete a Backup
- Click the **Delete** button on any backup configuration
- Confirm when prompted
- The backup configuration will be removed

### Understanding the Interface

**Header**
- 🔒 Lock icon in green - represents security and data protection
- "InLocker" - app name
- Tagline: "Automatic, compressed, and secure backups"

**Color Scheme**
- Emerald green - represents security and safety
- Dark background - reduces eye strain
- Clear typography - easy to read

**Badges**
- **Enabled** (green) - backup is active
- **Disabled** (gray) - backup is inactive
- **🔒 Encrypted** (green with lock) - backup uses encryption

---

## Where Are My Configurations Stored?

Your backup configurations are saved in:
```
~/Library/Application Support/com.inlocker.app/configs.json
```

This ensures your settings persist between app launches.

---

## Coming Soon

The following features are currently in development:

- ✅ **Manual backup execution** - Run backups on demand
- 🔄 **Automatic scheduled backups** - Set it and forget it
- 📦 **Compression** - Save disk space with zstd
- 🔐 **Encryption** - Protect your data with AES-256
- 📊 **Dashboard** - View backup history and statistics
- ♻️ **Restore** - Recover files from backups

---

## Tips for Best Results

1. **Choose External Drives for Destinations**
   - Protect against disk failure
   - Keep backups separate from source

2. **Descriptive Folder Names**
   - Backup names are auto-generated from folder names
   - Use clear folder names for easy identification

3. **Regular Testing**
   - Verify your backups are working
   - Check destination folder for backup files

---

## Troubleshooting

### App Won't Open
- Make sure you're running macOS 12.0 or later
- Check Terminal for error messages
- Try running `pnpm tauri dev` again

### Can't Select Folders
- Ensure you have read permissions for the source folder
- Ensure you have write permissions for the destination folder

### Configuration Not Saving
- Check that the app has permission to write to Application Support folder
- Try restarting the app

---

## Keyboard Shortcuts

Currently, InLocker uses standard macOS shortcuts:
- **⌘Q** - Quit application
- **⌘W** - Close window

More shortcuts coming in future updates!

---

## Need Help?

- **Report Issues**: [GitHub Issues](https://github.com/beloureiro/inlocker/issues)
- **Documentation**: Check the `/docs` folder in the repository
- **Roadmap**: See `docs/04-roadmap.md` for upcoming features

---

## Privacy & Security

- ✅ All data stays on your Mac
- ✅ No cloud services or external connections
- ✅ No telemetry or tracking
- ✅ Open source - you can verify the code

---

**Built with ❤️ using Tauri, Rust, and React**
