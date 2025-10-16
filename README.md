# szmer

A simple terminal-based break reminder for macOS and Linux that sends native notifications every X minutes to remind you to take breaks. Each notification includes a wellness tip to encourage healthy work habits.

## Features

- Scheduled break reminders via native system notifications
- Randomized wellness tips with each notification
- Configurable notification sounds
- Automatic scheduling using your system's scheduler (launchd on macOS, systemd on Linux)
- Simple CLI for installation and management
- Lightweight and simple to use

## Requirements

- macOS or Linux
- Rust toolchain (for building)

## Installation

1. Build the project in release mode:

   ```bash
   cargo build --release
   ```

2. Install the break reminder:

   ```bash
   ./target/release/szmer install
   ```

   This will:
   - Prompt you to select a notification sound from your system's available sounds
   - Save your preferences to `~/.config/szmer/config.json`
   - Set up automatic scheduling (launchd on macOS, systemd on Linux)
   - Display the binary path (do not move the binary after installation!)

## Usage

```
A simple break reminder for macOS and Linux

Usage: szmer <COMMAND>

Commands:
  install    Install the break reminder as a launchd agent
  uninstall  Uninstall the break reminder
  notify     Send a break notification (used internally by launchd)
  stop       Stop break reminders temporarily
  resume     Resume break reminders
  status     Show current status and next notification time
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## How It Works

1. The `install` command lets you choose a notification sound and saves your preferences
2. It creates a scheduler configuration (launchd on macOS, systemd on Linux) that runs the `notify` command every X minutes
3. Your system scheduler automatically runs the notification command in the background
4. You'll receive a native notification X minutes with a random wellness tip reminding you to take a break
5. The first notification appears X minutes after installation

## Important Notes

- **Do not move or delete the binary after installation**. The scheduler configuration contains the absolute path to the binary.
- If you need to move the binary, uninstall first, move it, then reinstall.
- Your notification sound preference is saved in `~/.config/szmer/config.json`.

