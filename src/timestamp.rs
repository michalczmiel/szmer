use chrono::{DateTime, Local};
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write as IoWrite;
use std::path::PathBuf;

/// Get the path to the cache directory for szmer
fn get_cache_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = env::var("HOME")?;
    Ok(PathBuf::from(home).join(".cache").join("szmer"))
}

/// Get the path to the timestamp file
fn get_timestamp_file() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(get_cache_dir()?.join("last_notification"))
}

/// Record the current timestamp as the last notification time
pub fn record_notification() -> Result<(), Box<dyn std::error::Error>> {
    let cache_dir = get_cache_dir()?;
    fs::create_dir_all(&cache_dir)?;

    let timestamp_file = get_timestamp_file()?;
    let now = Local::now();
    let timestamp = now.timestamp();

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(timestamp_file)?;

    writeln!(file, "{timestamp}")?;

    Ok(())
}

/// Get the last notification timestamp
pub fn get_last_notification() -> Result<Option<DateTime<Local>>, Box<dyn std::error::Error>> {
    let timestamp_file = get_timestamp_file()?;

    if !timestamp_file.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(timestamp_file)?;
    let last_line = content.lines().last();

    match last_line {
        Some(line) => {
            let timestamp = line.trim().parse::<i64>()?;
            let dt = DateTime::from_timestamp(timestamp, 0)
                .ok_or("Invalid timestamp")?
                .with_timezone(&Local);
            Ok(Some(dt))
        }
        None => Ok(None),
    }
}
