use crate::config::TimewarriorConfig;
use dialoguer::Confirm;
use std::path::PathBuf;
use std::process::Command;

/// Status information about timewarrior integration
#[derive(Debug)]
pub struct Status {
    /// Whether timewarrior is installed on the system
    pub is_installed: bool,
    /// Path to the timewarrior binary, if found
    pub binary_path: Option<PathBuf>,
    /// Whether there's currently an active tracking session
    /// None if status cannot be determined (e.g., timewarrior not installed)
    pub is_tracking: Option<bool>,
}

/// Check if timewarrior is installed and available in PATH
///
/// # Examples
///
/// ```
/// if timewarrior::is_installed() {
///     println!("Timewarrior is available");
/// }
/// ```
pub fn is_installed() -> bool {
    get_binary_path().is_some()
}

/// Get the absolute path to the timewarrior binary
///
/// # Returns
///
/// Returns `Some(PathBuf)` if the `timew` binary is found in PATH,
/// otherwise returns `None`.
///
/// # Examples
///
/// ```
/// if let Some(path) = timewarrior::get_binary_path() {
///     println!("Timewarrior found at: {}", path.display());
/// }
/// ```
pub fn get_binary_path() -> Option<PathBuf> {
    Command::new("which")
        .arg("timew")
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout)
                    .ok()
                    .map(|s| PathBuf::from(s.trim()))
            } else {
                None
            }
        })
}

/// Check if there's currently an active time tracking session
///
/// # Errors
///
/// Returns an error if:
/// - Timewarrior is not installed
/// - The `timew get dom.active` command fails
/// - Command output cannot be parsed
///
/// # Examples
///
/// ```
/// match timewarrior::is_tracking_active() {
///     Ok(true) => println!("Currently tracking time"),
///     Ok(false) => println!("Not tracking"),
///     Err(e) => eprintln!("Error checking status: {}", e),
/// }
/// ```
pub fn is_tracking_active() -> Result<bool, Box<dyn std::error::Error>> {
    let output = Command::new("timew")
        .arg("get")
        .arg("dom.active")
        .output()?;

    if !output.status.success() {
        return Err("Failed to query timewarrior status".into());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim() == "1")
}

/// Determine if a notification should be sent based on timewarrior status
///
/// This encapsulates the decision logic:
/// - If integration is disabled, always send notifications
/// - If timewarrior is not installed, always send notifications
/// - If timewarrior is installed and enabled:
///   - Send notification only if actively tracking
///   - Don't send if not tracking (prevents interruptions during personal time)
///
/// # Arguments
///
/// * `config` - The timewarrior configuration settings
///
/// # Returns
///
/// Returns `true` if notification should be sent, `false` otherwise
///
/// # Examples
///
/// ```
/// let config = TimewarriorConfig { enabled: true };
/// if timewarrior::should_send_notification(&config) {
///     notification::send(...);
/// }
/// ```
pub fn should_send_notification(config: &TimewarriorConfig) -> bool {
    // Integration disabled - always notify
    if !config.enabled {
        return true;
    }

    // Timewarrior no longer available - fall back to always notifying
    if !is_installed() {
        return true;
    }

    // Check active status - on error, assume active (fail-safe: send notification)
    is_tracking_active().unwrap_or(true)
}

/// Interactively prompt the user to configure timewarrior integration
///
/// This function:
/// 1. Checks if timewarrior is installed
/// 2. If not installed, returns default config (disabled)
/// 3. If installed, displays information and prompts user for preference
///
/// # Errors
///
/// Returns an error if the interactive prompt fails
///
/// # Examples
///
/// ```
/// let config = timewarrior::prompt_for_configuration()?;
/// println!("Timewarrior enabled: {}", config.enabled);
/// ```
pub fn prompt_for_configuration() -> Result<TimewarriorConfig, Box<dyn std::error::Error>> {
    // Check if timewarrior is installed
    if !is_installed() {
        // Not found, return default (disabled)
        return Ok(TimewarriorConfig::default());
    }

    // Display detection info
    if let Some(path) = get_binary_path() {
        println!("\n✓ Timewarrior detected at {}", path.display());
    }

    // Show explanation
    println!("\nTimewarrior Integration");
    println!("━━━━━━━━━━━━━━━━━━━━━━━");
    println!("\nSzmer can integrate with Timewarrior to skip reminders when you're not tracking work:");
    println!("  • Notifications only when actively tracking time");
    println!("  • No interruptions during personal time");
    println!("  • Helps maintain focus during work sessions");

    // Prompt user
    let enable = Confirm::new()
        .with_prompt("\nEnable Timewarrior integration? (skip notifications when not tracking)")
        .default(true)
        .interact()?;

    // Provide feedback
    if enable {
        println!("✓ Timewarrior integration enabled - will skip notifications when not tracking");
    } else {
        println!("✓ Using standard interval-based reminders");
    }

    Ok(TimewarriorConfig { enabled: enable })
}

/// Get comprehensive status information about timewarrior
///
/// This is useful for displaying detailed status to users, including
/// installation status, binary path, and current tracking state.
///
/// # Examples
///
/// ```
/// let status = timewarrior::get_status();
/// if status.is_installed {
///     println!("Installed at: {}", status.binary_path.unwrap().display());
///     match status.is_tracking {
///         Some(true) => println!("Currently tracking"),
///         Some(false) => println!("Not tracking"),
///         None => println!("Status unknown"),
///     }
/// }
/// ```
pub fn get_status() -> Status {
    let is_installed = is_installed();
    let binary_path = get_binary_path();
    let is_tracking = if is_installed {
        is_tracking_active().ok()
    } else {
        None
    };

    Status {
        is_installed,
        binary_path,
        is_tracking,
    }
}
