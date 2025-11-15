mod config;
mod notification;
mod schedule;
mod sound;
mod time;
mod timestamp;
mod timewarrior;

use clap::{Parser, Subcommand};
use config::Config;
use dialoguer::{Input, Select};
use time::{format_interval, format_time_until};

#[derive(Parser)]
#[command(name = "szmer")]
#[command(about = "A simple break reminder for macOS and Linux", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install the break reminder as a launchd agent
    Install,
    /// Uninstall the break reminder
    Uninstall,
    /// Send a break notification (used internally by launchd)
    Notify,
    /// Stop break reminders temporarily
    Stop,
    /// Resume break reminders
    Resume,
    /// Show current status and next notification time
    Status,
    /// Manage configuration settings
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Show current configuration
    Show,
    /// Set configuration values
    Set {
        /// Configuration key (e.g., "timewarrior.enabled")
        key: String,
        /// Configuration value (e.g., "true", "false")
        value: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Install => install(),
        Commands::Uninstall => uninstall(),
        Commands::Notify => notify(),
        Commands::Stop => stop(),
        Commands::Resume => resume(),
        Commands::Status => status(),
        Commands::Config { action } => config(action),
    }
}

fn notify() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;

    if config.paused {
        return Ok(());
    }

    // Check timewarrior integration - skip notification if not tracking
    if !timewarrior::should_send_notification(&config.timewarrior) {
        return Err("Skipping notification: no active timewarrior session".into());
    }

    notification::send_break_reminder(config.notification_sound, None)
}

fn install() -> Result<(), Box<dyn std::error::Error>> {
    if schedule::is_installed() {
        return Err(
            "Break reminder is already installed. Run 'uninstall' first if you want to reinstall."
                .into(),
        );
    }

    let interval_seconds = select_interval()?;
    println!(
        "\n✓ Break interval set to {} minutes",
        interval_seconds / 60
    );

    let selected_sound = select_notification_sound()?;

    let timewarrior_config = configure_timewarrior()?;

    let config = Config {
        notification_sound: selected_sound.clone(),
        paused: false,
        interval_seconds,
        timewarrior: timewarrior_config,
    };
    config.save()?;

    print_sound_confirmation(&selected_sound);

    schedule::install(interval_seconds)?;

    println!("\nTip: You can test the notification by running: szmer notify");

    Ok(())
}

fn select_interval() -> Result<u64, Box<dyn std::error::Error>> {
    println!("\nSelect a break interval:");

    let interval_presets = [
        ("Eye Saver (20 minutes) - 20-20-20 rule", 20),
        ("Pomodoro Focus (25 minutes)", 25),
        ("Answer to Everything (42 minutes)", 42),
        ("Standard Hour (60 minutes)", 60),
        ("Deep Work (90 minutes)", 90),
        ("The Numbers (108 minutes)", 108),
        ("Custom interval", 0),
    ];

    let interval_options: Vec<_> = interval_presets.iter().map(|(desc, _)| *desc).collect();

    let interval_selection = Select::new()
        .items(&interval_options)
        .default(3)
        .interact()?;

    let minutes = if interval_presets[interval_selection].1 == 0 {
        get_custom_interval()?
    } else {
        interval_presets[interval_selection].1
    };

    Ok(minutes * 60)
}

fn get_custom_interval() -> Result<u64, Box<dyn std::error::Error>> {
    let input: String = Input::new()
        .with_prompt("Enter custom interval in minutes")
        .validate_with(|input: &String| -> Result<(), &str> {
            match input.parse::<u64>() {
                Ok(n) if n > 0 && n <= 1440 => Ok(()),
                Ok(_) => Err("Please enter a value between 1 and 1440 minutes (24 hours)"),
                Err(_) => Err("Please enter a valid number"),
            }
        })
        .interact()?;

    Ok(input.parse().expect("validated input"))
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn select_notification_sound() -> Result<Option<String>, Box<dyn std::error::Error>> {
    println!("\nSelect a notification sound:");
    let sounds = sound::get_available_sounds()?;

    if sounds.is_empty() {
        return Err("No sounds found in system sounds directory".into());
    }

    let selection = Select::new().items(&sounds).default(0).interact()?;
    Ok(Some(sounds[selection].clone()))
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn select_notification_sound() -> Result<Option<String>, Box<dyn std::error::Error>> {
    Ok(None)
}

fn configure_timewarrior() -> Result<config::TimewarriorConfig, Box<dyn std::error::Error>> {
    timewarrior::prompt_for_configuration()
}

fn print_sound_confirmation(sound: &Option<String>) {
    match sound {
        Some(s) => println!("\n✓ Configuration saved with sound: {s}"),
        None => println!("\n✓ Configuration saved"),
    }
}

fn uninstall() -> Result<(), Box<dyn std::error::Error>> {
    schedule::uninstall()
}

fn stop() -> Result<(), Box<dyn std::error::Error>> {
    if !schedule::is_installed() {
        return Err("Break reminder is not installed. Run 'install' first.".into());
    }

    let mut config = Config::load()?;

    if config.paused {
        println!("Break reminders are already stopped.");
        return Ok(());
    }

    config.paused = true;
    config.save()?;

    println!("✓ Break reminders stopped.");
    println!("Run 'szmer resume' to start receiving reminders again.");

    Ok(())
}

fn resume() -> Result<(), Box<dyn std::error::Error>> {
    if !schedule::is_installed() {
        return Err("Break reminder is not installed. Run 'install' first.".into());
    }

    let mut config = Config::load()?;

    if !config.paused {
        println!("Break reminders are already running.");
        return Ok(());
    }

    config.paused = false;
    config.save()?;

    println!("✓ Break reminders resumed.");

    Ok(())
}

fn status() -> Result<(), Box<dyn std::error::Error>> {
    println!("\nSzmer Status");
    println!("━━━━━━━━━━━━");

    if !schedule::is_installed() {
        println!("\nScheduler:    ✗ Not installed");
        println!("\nRun 'szmer install' to set up break reminders.");
        return Ok(());
    }

    let config = Config::load()?;
    let scheduler_status = schedule::get_scheduler_status();

    print_scheduler_status(&scheduler_status);
    print_interval(&config);
    print_sound_setting(&config);
    print_pause_status(&config);
    print_next_break(&scheduler_status, &config);

    println!();
    Ok(())
}

fn print_scheduler_status(status: &Result<schedule::SchedulerStatus, Box<dyn std::error::Error>>) {
    let message = match status {
        Ok(s) if s.is_running => "✓ Running",
        Ok(_) => "⚠ Installed but not running",
        Err(_) => "✗ Error checking status",
    };
    println!("\nScheduler:    {message}");
}

fn print_interval(config: &Config) {
    println!("Interval:     {}", format_interval(config.interval_seconds));
}

fn print_sound_setting(config: &Config) {
    let sound = config
        .notification_sound
        .as_deref()
        .unwrap_or("(system default)");
    println!("Sound:        {sound}");
}

fn print_pause_status(config: &Config) {
    if config.paused {
        println!("Status:       ⏸ Paused");
        println!("\nRun 'szmer resume' to start receiving reminders again.");
    } else {
        println!("Status:       ▶ Active");
    }
}

fn print_next_break(
    scheduler_status: &Result<schedule::SchedulerStatus, Box<dyn std::error::Error>>,
    config: &Config,
) {
    if config.paused {
        return;
    }

    let Ok(status) = scheduler_status else {
        return;
    };

    match status.next_run {
        Some(next_run) => {
            let time_until = format_time_until(next_run);
            println!(
                "Next break:   {time_until} ({})",
                next_run.format("%I:%M %p")
            );
        }
        None => {
            println!(
                "Next break:   Every {} (no notification sent yet)",
                format_interval(config.interval_seconds)
            );
        }
    }
}

fn config(action: ConfigAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        ConfigAction::Show => show_config(),
        ConfigAction::Set { key, value } => set_config(&key, &value),
    }
}

fn show_config() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;

    println!("\nCurrent Configuration");
    println!("━━━━━━━━━━━━━━━━━━━━━");
    println!("\nSound:                 {}",
        config.notification_sound.as_deref().unwrap_or("(system default)"));
    println!("Paused:                {}", config.paused);
    println!("Interval:              {}", format_interval(config.interval_seconds));

    println!("\nTimewarrior Integration:");
    println!("  Enabled:             {}", config.timewarrior.enabled);

    if config.timewarrior.enabled {
        let status = timewarrior::get_status();

        if status.is_installed {
            if let Some(path) = status.binary_path {
                println!("  Binary path:         {}", path.display());
            }

            // Check current status
            match status.is_tracking {
                Some(true) => println!("  Current status:      ✓ Active session (will notify)"),
                Some(false) => println!("  Current status:      ○ No active session (will skip)"),
                None => println!("  Current status:      ⚠ Error checking status"),
            }
        } else {
            println!("  Status:              ⚠ Timewarrior not found in PATH");
        }
    }

    println!();
    Ok(())
}

fn set_config(key: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Config::load()?;

    match key {
        "timewarrior.enabled" => {
            let enabled = parse_bool(value)?;

            if enabled && !timewarrior::is_installed() {
                return Err("Cannot enable timewarrior integration: timewarrior not found in PATH".into());
            }

            config.timewarrior.enabled = enabled;
            println!("✓ Timewarrior integration {}", if enabled { "enabled (will skip notifications when not tracking)" } else { "disabled" });
        }
        _ => {
            return Err(format!(
                "Unknown configuration key: '{key}'. Available keys:\n  - timewarrior.enabled"
            ).into());
        }
    }

    config.save()?;
    Ok(())
}

fn parse_bool(value: &str) -> Result<bool, Box<dyn std::error::Error>> {
    match value.to_lowercase().as_str() {
        "true" | "1" | "yes" | "y" => Ok(true),
        "false" | "0" | "no" | "n" => Ok(false),
        _ => Err(format!("Invalid boolean value: '{value}'. Use 'true' or 'false'").into()),
    }
}

