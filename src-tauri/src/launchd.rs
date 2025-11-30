use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// macOS launchd integration for independent backup scheduling
/// Creates and manages .plist files in ~/Library/LaunchAgents

/// Get the correct executable path (handles both dev and production mode)
pub fn get_executable_path() -> Result<String, String> {
    let current = std::env::current_exe()
        .map_err(|e| format!("Failed to get current exe: {}", e))?;

    let path_str = current.to_str().ok_or("Invalid UTF-8 in exe path")?;

    log::info!("Current executable path: {}", path_str);

    // Check if running from bundle (production)
    if path_str.contains(".app/Contents/MacOS") {
        // Production: use bundle path
        let bundle_path = "/Applications/InLocker.app/Contents/MacOS/inlocker";
        log::info!("Detected production mode, using bundle path: {}", bundle_path);
        return Ok(bundle_path.to_string());
    }

    // Dev mode: use current executable
    log::info!("Detected dev mode, using current path: {}", path_str);
    Ok(path_str.to_string())
}

/// Get the path to LaunchAgents directory
fn get_launch_agents_dir() -> Result<PathBuf, String> {
    let home = std::env::var("HOME").map_err(|e| format!("Failed to get HOME: {}", e))?;
    Ok(PathBuf::from(home).join("Library/LaunchAgents"))
}

/// Get the .plist file path for a backup config
pub fn get_plist_path(config_id: &str) -> Result<PathBuf, String> {
    let dir = get_launch_agents_dir()?;
    Ok(dir.join(format!("com.inlocker.backup.{}.plist", config_id)))
}

/// Get the label for a launch agent
fn get_agent_label(config_id: &str) -> String {
    format!("com.inlocker.backup.{}", config_id)
}

/// Get current user's UID
fn get_user_uid() -> Result<String, String> {
    let output = Command::new("id")
        .args(&["-u"])
        .output()
        .map_err(|e| format!("Failed to execute id command: {}", e))?;

    if !output.status.success() {
        return Err("Failed to get user UID".to_string());
    }

    let uid = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(uid)
}

/// Get logs directory path
pub fn get_logs_dir() -> Result<PathBuf, String> {
    let home = std::env::var("HOME").map_err(|e| format!("Failed to get HOME: {}", e))?;
    let logs_dir = PathBuf::from(home).join("Library/Logs/InLocker");

    // Create directory if it doesn't exist
    if !logs_dir.exists() {
        fs::create_dir_all(&logs_dir)
            .map_err(|e| format!("Failed to create logs directory: {}", e))?;
        log::info!("Created logs directory: {:?}", logs_dir);
    }

    Ok(logs_dir)
}

/// Get log file path for a config
pub fn get_log_path(config_id: &str) -> Result<PathBuf, String> {
    let logs_dir = get_logs_dir()?;
    Ok(logs_dir.join(format!("scheduled-{}.log", config_id)))
}

/// Get error log file path for a config
pub fn get_error_log_path(config_id: &str) -> Result<PathBuf, String> {
    let logs_dir = get_logs_dir()?;
    Ok(logs_dir.join(format!("scheduled-{}.err", config_id)))
}

/// Check if a launch agent is loaded in launchd
pub fn is_agent_loaded(config_id: &str) -> Result<bool, String> {
    let label = get_agent_label(config_id);

    let output = Command::new("launchctl")
        .args(&["list"])
        .output()
        .map_err(|e| format!("Failed to execute launchctl list: {}", e))?;

    if !output.status.success() {
        return Err("Failed to list launch agents".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.contains(&label))
}

/// Parse cron expression and convert to launchd StartCalendarInterval
/// Cron format: "minute hour day month weekday"
/// NOTE: launchd uses LOCAL timezone, no conversion needed
/// Returns Vec of calendar intervals (can have multiple for complex schedules)
fn parse_cron_to_calendar_interval(cron_expr: &str) -> Result<Vec<CalendarInterval>, String> {
    let parts: Vec<&str> = cron_expr.split_whitespace().collect();

    if parts.len() != 5 {
        return Err(format!(
            "Invalid cron expression: expected 5 fields, got {}",
            parts.len()
        ));
    }

    let minute = parts[0];
    let hour = parts[1];
    let day = parts[2];
    let month = parts[3];
    let weekday = parts[4];

    // Simple case: specific time (e.g., "0 2 * * *" = daily at 2:00 AM)
    let mut intervals = Vec::new();

    // Parse minute and hour (input is in LOCAL timezone)
    let minutes = parse_cron_field(minute, 0, 59)?;
    let hours = parse_cron_field(hour, 0, 23)?;
    let days = if day == "*" {
        vec![]
    } else {
        parse_cron_field(day, 1, 31)?
    };
    let months = if month == "*" {
        vec![]
    } else {
        parse_cron_field(month, 1, 12)?
    };
    let weekdays = if weekday == "*" {
        vec![]
    } else {
        parse_cron_field(weekday, 0, 6)?
    };

    // Generate all combinations
    if minutes.is_empty() || hours.is_empty() {
        return Err("Minute and Hour must be specified".to_string());
    }

    for &m in &minutes {
        for &h in &hours {
            // launchd uses LOCAL timezone, NOT UTC
            // No conversion needed - just use the hour/minute as specified
            log::info!(
                "Scheduling backup at LOCAL time: {}:{:02}",
                h, m
            );

            let interval = CalendarInterval {
                minute: Some(m),
                hour: Some(h),
                day: if days.is_empty() { None } else { Some(days[0]) },
                month: if months.is_empty() {
                    None
                } else {
                    Some(months[0])
                },
                weekday: if weekdays.is_empty() {
                    None
                } else {
                    Some(weekdays[0])
                },
            };
            intervals.push(interval);
        }
    }

    if intervals.is_empty() {
        return Err("No valid intervals generated from cron expression".to_string());
    }

    Ok(intervals)
}

/// Parse a single cron field (supports *, specific values, and ranges)
fn parse_cron_field(field: &str, min: i32, max: i32) -> Result<Vec<i32>, String> {
    if field == "*" {
        // For *, we don't expand - handled separately
        return Ok(vec![]);
    }

    if field.contains(',') {
        // List: "1,5,10"
        return field
            .split(',')
            .map(|s| {
                s.parse::<i32>()
                    .map_err(|e| format!("Invalid number: {}", e))
                    .and_then(|n| {
                        if n >= min && n <= max {
                            Ok(n)
                        } else {
                            Err(format!("Value {} out of range {}-{}", n, min, max))
                        }
                    })
            })
            .collect();
    }

    if field.contains('-') {
        // Range: "1-5"
        let parts: Vec<&str> = field.split('-').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid range: {}", field));
        }
        let start: i32 = parts[0]
            .parse()
            .map_err(|e| format!("Invalid range start: {}", e))?;
        let end: i32 = parts[1]
            .parse()
            .map_err(|e| format!("Invalid range end: {}", e))?;

        if start < min || end > max || start > end {
            return Err(format!("Invalid range {}-{}", start, end));
        }

        return Ok((start..=end).collect());
    }

    // Single value
    let val: i32 = field
        .parse()
        .map_err(|e| format!("Invalid number: {}", e))?;

    if val < min || val > max {
        return Err(format!("Value {} out of range {}-{}", val, min, max));
    }

    Ok(vec![val])
}

#[derive(Debug)]
struct CalendarInterval {
    minute: Option<i32>,
    hour: Option<i32>,
    day: Option<i32>,
    month: Option<i32>,
    weekday: Option<i32>,
}

impl CalendarInterval {
    fn to_xml(&self) -> String {
        let mut xml = String::from("    <dict>\n");

        if let Some(minute) = self.minute {
            xml.push_str(&format!("      <key>Minute</key>\n"));
            xml.push_str(&format!("      <integer>{}</integer>\n", minute));
        }

        if let Some(hour) = self.hour {
            xml.push_str(&format!("      <key>Hour</key>\n"));
            xml.push_str(&format!("      <integer>{}</integer>\n", hour));
        }

        if let Some(day) = self.day {
            xml.push_str(&format!("      <key>Day</key>\n"));
            xml.push_str(&format!("      <integer>{}</integer>\n", day));
        }

        if let Some(month) = self.month {
            xml.push_str(&format!("      <key>Month</key>\n"));
            xml.push_str(&format!("      <integer>{}</integer>\n", month));
        }

        if let Some(weekday) = self.weekday {
            xml.push_str(&format!("      <key>Weekday</key>\n"));
            xml.push_str(&format!("      <integer>{}</integer>\n", weekday));
        }

        xml.push_str("    </dict>\n");
        xml
    }
}

/// Generate .plist XML content for a backup schedule
pub fn generate_plist_content(
    config_id: &str,
    cron_expr: &str,
    app_path: &str,
) -> Result<String, String> {
    let label = get_agent_label(config_id);
    let intervals = parse_cron_to_calendar_interval(cron_expr)?;

    let mut plist = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>"#,
    );
    plist.push_str(&label);
    plist.push_str(
        r#"</string>

  <key>ProgramArguments</key>
  <array>
    <string>"#,
    );
    plist.push_str(app_path);
    plist.push_str(
        r#"</string>
    <string>--backup</string>
    <string>"#,
    );
    plist.push_str(config_id);
    plist.push_str(
        r#"</string>
  </array>

  <key>StartCalendarInterval</key>
"#,
    );

    // Add calendar intervals
    if intervals.len() == 1 {
        plist.push_str(&intervals[0].to_xml());
    } else {
        plist.push_str("  <array>\n");
        for interval in intervals {
            plist.push_str(&interval.to_xml());
        }
        plist.push_str("  </array>\n");
    }

    // Get persistent log paths
    let log_path = get_log_path(config_id)?;
    let err_log_path = get_error_log_path(config_id)?;

    plist.push_str(&format!(
        r#"
  <key>RunAtLoad</key>
  <false/>

  <key>StandardOutPath</key>
  <string>{}</string>

  <key>StandardErrorPath</key>
  <string>{}</string>
</dict>
</plist>
"#,
        log_path.to_string_lossy(),
        err_log_path.to_string_lossy()
    ));

    Ok(plist)
}

/// Install a launch agent (write .plist file and load it)
/// with robust verification
pub fn install_launch_agent(
    config_id: &str,
    cron_expr: &str,
    app_path: &str,
) -> Result<(), String> {
    log::info!("=== Installing launch agent for config: {} ===", config_id);
    log::info!("Cron expression: {}", cron_expr);
    log::info!("Executable path: {}", app_path);

    // STEP 1: Ensure LaunchAgents directory exists
    let dir = get_launch_agents_dir()?;
    log::info!("LaunchAgents directory: {:?}", dir);

    if !dir.exists() {
        log::info!("Creating LaunchAgents directory...");
        fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create LaunchAgents directory: {}", e))?;
    }

    // STEP 2: Ensure logs directory exists
    let logs_dir = get_logs_dir()?;
    log::info!("Logs directory: {:?}", logs_dir);

    // STEP 3: Generate plist content
    log::info!("Generating .plist content...");
    let plist_content = generate_plist_content(config_id, cron_expr, app_path)?;

    // STEP 3.5: If agent already exists, bootout it first to force reload
    let plist_path = get_plist_path(config_id)?;
    let label = get_agent_label(config_id);
    let user_uid = get_user_uid()?;

    if plist_path.exists() {
        log::info!("Agent already exists, using bootout before update...");
        let bootout_output = Command::new("launchctl")
            .args(&["bootout", &format!("gui/{}/{}", user_uid, label)])
            .output()
            .map_err(|e| format!("Failed to execute launchctl bootout: {}", e))?;

        if !bootout_output.status.success() {
            let stderr = String::from_utf8_lossy(&bootout_output.stderr);
            // It's ok if not loaded (exit code 3 = No such process)
            if !stderr.contains("No such process") {
                log::warn!("Failed to bootout agent (continuing anyway): {}", stderr);
            } else {
                log::info!("Agent was not loaded (this is OK)");
            }
        } else {
            log::info!("✓ Agent unloaded successfully with bootout");
        }
    } else {
        log::info!("New agent installation (no existing plist)");
    }

    // STEP 4: Write plist file
    log::info!("Writing .plist file to: {:?}", plist_path);

    fs::write(&plist_path, &plist_content)
        .map_err(|e| format!("Failed to write plist file: {}", e))?;

    // STEP 5: Verify plist file was created
    if !plist_path.exists() {
        return Err(format!("Plist file was not created: {:?}", plist_path));
    }
    log::info!("✓ Plist file created successfully");

    // STEP 6: Verify plist content
    let written_content = fs::read_to_string(&plist_path)
        .map_err(|e| format!("Failed to read plist file: {}", e))?;
    if written_content != plist_content {
        return Err("Plist file content mismatch after write".to_string());
    }
    log::info!("✓ Plist content verified");

    // STEP 7: Load the agent with launchctl bootstrap (modern macOS command)
    log::info!("Loading agent with launchctl bootstrap: {}", label);

    let output = Command::new("launchctl")
        .args(&[
            "bootstrap",
            &format!("gui/{}", user_uid),
            plist_path.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| format!("Failed to execute launchctl bootstrap: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        log::warn!("launchctl bootstrap stderr: {}", stderr);
        log::warn!("launchctl bootstrap stdout: {}", stdout);

        // Exit code 5 means already loaded or I/O error
        if !stderr.contains("service is already loaded") && !stderr.contains("Bootstrap failed: 5") {
            return Err(format!("Failed to load launch agent: {}", stderr));
        }
        log::info!("Agent was already loaded (this is OK)");
    } else {
        log::info!("✓ Agent loaded successfully with bootstrap");
    }

    // STEP 8: Verify agent is loaded
    log::info!("Verifying agent is loaded...");
    match is_agent_loaded(config_id) {
        Ok(true) => {
            log::info!("✓ Agent verified in launchctl list");
        }
        Ok(false) => {
            return Err(format!(
                "Agent loaded but not visible in launchctl list: {}",
                label
            ));
        }
        Err(e) => {
            log::warn!("Could not verify agent status: {}", e);
        }
    }

    // STEP 9: Skip automatic kickstart to avoid launching duplicate app instance
    // User can manually test using "Test Now" button in UI
    log::info!("✓ Schedule registered successfully (skipping automatic test)");
    log::info!("   User can test manually using 'Test Now' button");
    log::info!("=== Launch agent installation completed successfully ===");
    Ok(())
}

/// Uninstall a launch agent (bootout and delete .plist file)
pub fn uninstall_launch_agent(config_id: &str) -> Result<(), String> {
    let plist_path = get_plist_path(config_id)?;
    let label = get_agent_label(config_id);
    let user_uid = get_user_uid()?;

    // Bootout the agent with launchctl (modern macOS command)
    let output = Command::new("launchctl")
        .args(&["bootout", &format!("gui/{}/{}", user_uid, label)])
        .output()
        .map_err(|e| format!("Failed to execute launchctl bootout: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // It's ok if not loaded (exit code 3 = No such process)
        if !stderr.contains("No such process") {
            log::warn!("Failed to bootout launch agent: {}", stderr);
        }
    }

    // Delete plist file
    if plist_path.exists() {
        fs::remove_file(&plist_path)
            .map_err(|e| format!("Failed to delete plist file: {}", e))?;
        log::info!("Deleted plist file: {:?}", plist_path);
    }

    log::info!("Uninstalled launch agent: {}", label);
    Ok(())
}

/// Check if a launch agent is installed
#[allow(dead_code)]
pub fn is_agent_installed(config_id: &str) -> bool {
    get_plist_path(config_id)
        .map(|p| p.exists())
        .unwrap_or(false)
}
