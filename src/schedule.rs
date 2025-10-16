use chrono::{DateTime, Local};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(target_os = "linux")]
use crate::time::parse_next_run_timestamp;

#[cfg(target_os = "macos")]
const SERVICE_LABEL: &str = "com.michalczmiel.szmer";
#[cfg(target_os = "macos")]
const SERVICE_FILENAME: &str = "com.michalczmiel.szmer.plist";

#[cfg(target_os = "linux")]
const SERVICE_FILENAME: &str = "szmer.service";

/// Install the scheduler to run break reminders at the specified interval
pub fn install(interval_seconds: u64) -> Result<(), Box<dyn std::error::Error>> {
    let service_path = get_service_path()?;

    if service_path.exists() {
        return Err(format!(
            "Service is already installed at {}. Run 'uninstall' first if you want to reinstall.",
            service_path.display()
        )
        .into());
    }

    let binary_path = get_binary_path()?;

    if let Some(parent) = service_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let service_content = generate_service_file(&binary_path, interval_seconds);
    fs::write(&service_path, service_content)?;

    println!("Created service file at: {}", service_path.display());

    load_service(&service_path, interval_seconds)?;

    println!("✓ Break reminder installed successfully!");
    println!(
        "You will receive break reminders every {} minutes.",
        interval_seconds / 60
    );
    println!("\nNote: Do not move or delete the binary at: {binary_path}");
    println!("To uninstall, run: szmer uninstall");

    Ok(())
}

fn get_binary_path() -> Result<String, Box<dyn std::error::Error>> {
    env::current_exe()?
        .canonicalize()?
        .to_str()
        .ok_or_else(|| "Failed to convert binary path to string".into())
        .map(String::from)
}

/// Uninstall the scheduler
pub fn uninstall() -> Result<(), Box<dyn std::error::Error>> {
    let service_path = get_service_path()?;

    if !service_path.exists() {
        println!("Service is not currently installed.");
        return Ok(());
    }

    // Unload/disable the service
    if let Err(e) = unload_service(&service_path) {
        eprintln!("Warning: Failed to unload service: {e}");
        eprintln!("Continuing with service file removal...");
    }

    // Remove the service file
    fs::remove_file(&service_path)?;

    println!("✓ Break reminder uninstalled successfully!");

    Ok(())
}

/// Check if the scheduler is installed
pub fn is_installed() -> bool {
    get_service_path().ok().is_some_and(|p| p.exists())
}

/// Get the scheduler status including next run time if available
pub fn get_scheduler_status() -> Result<SchedulerStatus, Box<dyn std::error::Error>> {
    if !is_installed() {
        return Err("Scheduler is not installed".into());
    }

    get_scheduler_status_impl()
}

#[derive(Debug)]
pub struct SchedulerStatus {
    pub is_running: bool,
    pub next_run: Option<DateTime<Local>>,
}

// Platform-specific implementations

#[cfg(target_os = "macos")]
fn get_service_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = env::var("HOME")?;
    Ok(PathBuf::from(home)
        .join("Library")
        .join("LaunchAgents")
        .join(SERVICE_FILENAME))
}

#[cfg(target_os = "linux")]
fn get_service_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = env::var("HOME")?;
    Ok(PathBuf::from(home)
        .join(".config")
        .join("systemd")
        .join("user")
        .join(SERVICE_FILENAME))
}

#[cfg(target_os = "macos")]
fn generate_service_file(binary_path: &str, interval_seconds: u64) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{SERVICE_LABEL}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{binary_path}</string>
        <string>notify</string>
    </array>
    <key>StartInterval</key>
    <integer>{interval_seconds}</integer>
    <key>RunAtLoad</key>
    <false/>
    <key>StandardOutPath</key>
    <string>/tmp/szmer.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/szmer.err</string>
</dict>
</plist>
"#
    )
}

#[cfg(target_os = "linux")]
fn generate_service_file(binary_path: &str, _interval_seconds: u64) -> String {
    format!(
        r#"[Unit]
Description=Szmer break reminder
After=default.target

[Service]
Type=oneshot
ExecStart={} notify

[Install]
WantedBy=default.target
"#,
        binary_path
    )
}

#[cfg(target_os = "macos")]
fn load_service(
    service_path: &Path,
    _interval_seconds: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    run_command(
        "launchctl",
        &["load", service_path.to_str().unwrap()],
        "Failed to load launchd agent",
    )
}

#[cfg(target_os = "linux")]
fn load_service(
    service_path: &Path,
    interval_seconds: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    run_command(
        "systemctl",
        &["--user", "daemon-reload"],
        "Failed to reload systemd",
    )?;

    let timer_path = service_path.with_extension("timer");
    let timer_content = format!(
        r#"[Unit]
Description=Szmer break reminder timer
Requires=szmer.service

[Timer]
OnBootSec={}
OnUnitActiveSec={}
Persistent=true

[Install]
WantedBy=timers.target
"#,
        interval_seconds, interval_seconds
    );
    fs::write(&timer_path, timer_content)?;

    run_command(
        "systemctl",
        &["--user", "enable", "--now", "szmer.timer"],
        "Failed to enable systemd timer",
    )?;

    Ok(())
}

#[cfg(target_os = "macos")]
fn unload_service(service_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    run_command(
        "launchctl",
        &["unload", service_path.to_str().unwrap()],
        "launchctl unload failed",
    )
}

#[cfg(target_os = "linux")]
fn unload_service(service_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    run_command(
        "systemctl",
        &["--user", "disable", "--now", "szmer.timer"],
        "Failed to disable systemd timer",
    )?;

    let timer_path = service_path.with_extension("timer");
    if timer_path.exists() {
        fs::remove_file(timer_path)?;
    }

    Command::new("systemctl")
        .arg("--user")
        .arg("daemon-reload")
        .output()?;

    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn get_service_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Err("Scheduling is not supported on this platform".into())
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn generate_service_file(_binary_path: &str, _interval_seconds: u64) -> String {
    String::new()
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn load_service(
    _service_path: &Path,
    _interval_seconds: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    Err("Scheduling is not supported on this platform".into())
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn unload_service(_service_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    Err("Scheduling is not supported on this platform".into())
}

#[cfg(target_os = "macos")]
fn get_scheduler_status_impl() -> Result<SchedulerStatus, Box<dyn std::error::Error>> {
    // Check if the launchd job is loaded/running
    let output = Command::new("launchctl")
        .arg("list")
        .arg(SERVICE_LABEL)
        .output()?;

    let is_running = output.status.success();

    // Note: launchd with StartInterval doesn't expose next run time easily
    // We return None for next_run on macOS
    Ok(SchedulerStatus {
        is_running,
        next_run: None,
    })
}

#[cfg(target_os = "linux")]
fn get_scheduler_status_impl() -> Result<SchedulerStatus, Box<dyn std::error::Error>> {
    let status_output = Command::new("systemctl")
        .arg("--user")
        .arg("is-active")
        .arg("szmer.timer")
        .output()?;

    let is_running = status_output.status.success();
    let next_run = if is_running {
        get_next_run_time()?
    } else {
        None
    };

    Ok(SchedulerStatus {
        is_running,
        next_run,
    })
}

#[cfg(target_os = "linux")]
fn get_next_run_time() -> Result<Option<DateTime<Local>>, Box<dyn std::error::Error>> {
    let output = Command::new("systemctl")
        .arg("--user")
        .arg("show")
        .arg("szmer.timer")
        .arg("-p")
        .arg("NextElapseUSecRealtime")
        .output()?;

    if !output.status.success() {
        return Ok(None);
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_next_run_timestamp(&output_str)
}


#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn get_scheduler_status_impl() -> Result<SchedulerStatus, Box<dyn std::error::Error>> {
    Err("Scheduling is not supported on this platform".into())
}

/// Helper function to run a command and return a formatted error if it fails
fn run_command(
    program: &str,
    args: &[&str],
    error_prefix: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(program).args(args).output()?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("{error_prefix}: {error_msg}").into());
    }

    Ok(())
}
