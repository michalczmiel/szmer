# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Szmer is a terminal-based break reminder application for macOS and Linux that sends native system notifications every hour to encourage healthy break habits. The application integrates with platform-specific schedulers (launchd on macOS, systemd on Linux) and displays randomized wellness tips.

## Build and Development Commands

```bash
# Build in debug mode
cargo build

# Build optimized release binary
cargo build --release

# Run the application
cargo run -- <command>

# Run tests (if any exist)
cargo test

# Check code without building
cargo check

# Format code
cargo fmt

# Run clippy for lints
cargo clippy
```

## Application Commands

```bash
# Install break reminder scheduler
./target/release/szmer install

# Uninstall break reminder
./target/release/szmer uninstall

# Test notification manually
./target/release/szmer notify
```

## Architecture

The application follows a modular architecture with clear separation of concerns:

### Module Structure

- **main.rs**: CLI entry point using clap, defines three commands (install, uninstall, notify)
- **config.rs**: Configuration management, stores user preferences in `~/.config/szmer/config.json`
- **notification.rs**: Notification system, displays wellness tips using notify-rust
- **schedule.rs**: Platform-specific scheduler integration (launchd/systemd)
- **sound.rs**: Sound discovery for platform-specific notification sounds

### Key Design Patterns

**Platform Abstraction**: The codebase uses conditional compilation (`#[cfg(target_os)]`) extensively to handle macOS and Linux differences. When adding new platform-specific features:

- macOS uses launchd with plist files at `~/Library/LaunchAgents/com.michalczmiel.szmer.plist`
- Linux uses systemd user services with timer units at `~/.config/systemd/user/szmer.{service,timer}`
- Always provide fallback implementations for unsupported platforms

**Scheduler Integration**:

- The `schedule.rs` module generates platform-specific service files dynamically
- Service files embed the absolute binary path, which means the binary location is fixed after installation
- The scheduler runs `szmer notify` every hour (3600 seconds on macOS via StartInterval, hourly on Linux via OnCalendar)

**Configuration Flow**:

1. During `install`, user selects notification sound via interactive prompt (dialoguer)
2. Configuration is serialized to JSON and saved to `~/.config/szmer/config.json`
3. During `notify`, configuration is loaded and applied to notification

**Notification System**:

- Uses notify-rust crate for cross-platform notifications
- Displays one random wellness tip from WELLNESS_TIPS array (17 tips in notification.rs:5-23)
- Applies user-selected sound from configuration

## Important Implementation Details

**Binary Path Handling**: The scheduler stores the absolute path to the binary. If you modify installation logic, ensure:

- `env::current_exe()?.canonicalize()?` is used to get the absolute path
- Users are warned not to move the binary after installation (see schedule.rs:48-52)
- Installation checks prevent duplicate installations (schedule.rs:18-24)

**Error Handling**: All functions return `Result<(), Box<dyn std::error::Error>>` for flexible error propagation. User-facing errors should be descriptive strings.

**Sound Selection**:

- macOS: Scans `/System/Library/Sounds` for `.aiff` files
- Linux: Scans multiple directories (freedesktop, gnome, ubuntu) for `.oga`, `.ogg`, `.wav` files
- Sound names are stripped of extensions before display

**Logging**:

- macOS launchd logs to `/tmp/szmer.log` and `/tmp/szmer.err` (schedule.rs:124-127)
- Users can check these logs for debugging

## Testing Locally

To test scheduler installation without waiting an hour:

1. Install normally: `cargo run --release -- install`
2. Check launchd/systemd status:
   - macOS: `launchctl list | grep szmer`
   - Linux: `systemctl --user status szmer.timer`
3. Trigger notification manually: `cargo run --release -- notify`
4. View logs:
   - macOS: `cat /tmp/szmer.log /tmp/szmer.err`
   - Linux: `journalctl --user -u szmer.service`

## Dependencies

- **notify-rust**: System notification interface
- **clap**: CLI argument parsing with derive macros
- **serde/serde_json**: Configuration serialization
- **dialoguer**: Interactive terminal prompts
- **rand**: Random tip selection
