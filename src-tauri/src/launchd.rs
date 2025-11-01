use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// macOS launchd integration for independent backup scheduling
/// Creates and manages .plist files in ~/Library/LaunchAgents

/// Get the path to LaunchAgents directory
fn get_launch_agents_dir() -> Result<PathBuf, String> {
    let home = std::env::var("HOME").map_err(|e| format!("Failed to get HOME: {}", e))?;
    Ok(PathBuf::from(home).join("Library/LaunchAgents"))
}

/// Get the .plist file path for a backup config
fn get_plist_path(config_id: &str) -> Result<PathBuf, String> {
    let dir = get_launch_agents_dir()?;
    Ok(dir.join(format!("com.inlocker.backup.{}.plist", config_id)))
}

/// Get the label for a launch agent
fn get_agent_label(config_id: &str) -> String {
    format!("com.inlocker.backup.{}", config_id)
}

/// Parse cron expression and convert to launchd StartCalendarInterval
/// Cron format: "minute hour day month weekday"
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

    // Parse minute
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

    plist.push_str(&format!(
        r#"
  <key>RunAtLoad</key>
  <false/>

  <key>StandardOutPath</key>
  <string>/tmp/inlocker-{}.log</string>

  <key>StandardErrorPath</key>
  <string>/tmp/inlocker-{}.err</string>
</dict>
</plist>
"#,
        config_id, config_id
    ));

    Ok(plist)
}

/// Install a launch agent (write .plist file and load it)
pub fn install_launch_agent(
    config_id: &str,
    cron_expr: &str,
    app_path: &str,
) -> Result<(), String> {
    // Ensure LaunchAgents directory exists
    let dir = get_launch_agents_dir()?;
    fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create LaunchAgents directory: {}", e))?;

    // Generate plist content
    let plist_content = generate_plist_content(config_id, cron_expr, app_path)?;

    // Write plist file
    let plist_path = get_plist_path(config_id)?;
    fs::write(&plist_path, plist_content)
        .map_err(|e| format!("Failed to write plist file: {}", e))?;

    log::info!("Created plist file: {:?}", plist_path);

    // Load the agent with launchctl
    let label = get_agent_label(config_id);
    let output = Command::new("launchctl")
        .args(&["load", plist_path.to_str().unwrap()])
        .output()
        .map_err(|e| format!("Failed to execute launchctl load: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // It's ok if already loaded
        if !stderr.contains("already loaded") {
            return Err(format!("Failed to load launch agent: {}", stderr));
        }
    }

    log::info!("Loaded launch agent: {}", label);
    Ok(())
}

/// Uninstall a launch agent (unload and delete .plist file)
pub fn uninstall_launch_agent(config_id: &str) -> Result<(), String> {
    let plist_path = get_plist_path(config_id)?;

    // Unload the agent with launchctl
    let label = get_agent_label(config_id);
    let output = Command::new("launchctl")
        .args(&["unload", plist_path.to_str().unwrap()])
        .output()
        .map_err(|e| format!("Failed to execute launchctl unload: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // It's ok if not loaded
        if !stderr.contains("Could not find specified service") {
            log::warn!("Failed to unload launch agent: {}", stderr);
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
