use chrono::{DateTime, Local};

/// Format a duration until a future time in a human-readable way
///
/// # Examples
///
/// ```
/// use chrono::Local;
/// use std::time::Duration;
///
/// let future = Local::now() + Duration::from_secs(3700);
/// let formatted = format_time_until(future);
/// // Returns something like "in 1 hour 1 minute"
/// ```
pub fn format_time_until(next_run: DateTime<Local>) -> String {
    let duration = next_run.signed_duration_since(Local::now());
    let hours = duration.num_hours().max(0) as u64;
    let minutes = (duration.num_minutes() % 60).max(0) as u64;

    match (hours, minutes) {
        (0, m) if m > 0 => format!("in {m}{}", pluralize_unit(" minute", m)),
        (h, m) if h > 0 => format!(
            "in {h}{} {m}{}",
            pluralize_unit(" hour", h),
            pluralize_unit(" minute", m)
        ),
        _ => "very soon".to_string(),
    }
}

/// Format an interval in seconds to a human-readable string
///
/// # Examples
///
/// ```
/// assert_eq!(format_interval(60), "1 minute");
/// assert_eq!(format_interval(3600), "1 hour");
/// assert_eq!(format_interval(3660), "1 hour 1 minute");
/// assert_eq!(format_interval(7200), "2 hours");
/// ```
pub fn format_interval(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;

    match (hours, minutes) {
        (0, m) => format!("{m}{}", pluralize_unit(" minute", m)),
        (h, 0) => format!("{h}{}", pluralize_unit(" hour", h)),
        (h, m) => format!(
            "{h}{} {m}{}",
            pluralize_unit(" hour", h),
            pluralize_unit(" minute", m)
        ),
    }
}

/// Pluralize time unit words based on count
///
/// # Examples
///
/// ```
/// assert_eq!(pluralize_unit(" minute", 1), " minute");
/// assert_eq!(pluralize_unit(" minute", 2), " minutes");
/// assert_eq!(pluralize_unit(" hour", 1), " hour");
/// assert_eq!(pluralize_unit(" hour", 5), " hours");
/// ```
pub fn pluralize_unit(word: &str, count: u64) -> &'static str {
    if count == 1 {
        match word {
            " minute" => " minute",
            " hour" => " hour",
            _ => "",
        }
    } else {
        match word {
            " minute" => " minutes",
            " hour" => " hours",
            _ => "s",
        }
    }
}

/// Parse systemd timestamp from NextElapseUSecRealtime output
///
/// # Examples
///
/// ```
/// let output = "NextElapseUSecRealtime=1234567890000000\n";
/// let result = parse_next_run_timestamp(output);
/// assert!(result.is_ok());
/// ```
#[cfg(target_os = "linux")]
pub fn parse_next_run_timestamp(
    output: &str,
) -> Result<Option<DateTime<Local>>, Box<dyn std::error::Error>> {
    let timestamp_str = output
        .strip_prefix("NextElapseUSecRealtime=")
        .and_then(|s| s.trim().parse::<i64>().ok());

    let Some(timestamp_usec) = timestamp_str else {
        return Ok(None);
    };

    let timestamp_sec = timestamp_usec / 1_000_000;
    let timestamp_nsec = ((timestamp_usec % 1_000_000) * 1_000) as u32;

    use chrono::TimeZone;
    Ok(Local.timestamp_opt(timestamp_sec, timestamp_nsec).single())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_interval_minutes_only() {
        assert_eq!(format_interval(60), "1 minute");
        assert_eq!(format_interval(120), "2 minutes");
        assert_eq!(format_interval(1800), "30 minutes");
    }

    #[test]
    fn test_format_interval_hours_only() {
        assert_eq!(format_interval(3600), "1 hour");
        assert_eq!(format_interval(7200), "2 hours");
        assert_eq!(format_interval(10800), "3 hours");
    }

    #[test]
    fn test_format_interval_hours_and_minutes() {
        assert_eq!(format_interval(3660), "1 hour 1 minute");
        assert_eq!(format_interval(3720), "1 hour 2 minutes");
        assert_eq!(format_interval(7260), "2 hours 1 minute");
        assert_eq!(format_interval(7320), "2 hours 2 minutes");
    }

    #[test]
    fn test_format_interval_zero_edge_case() {
        assert_eq!(format_interval(0), "0 minutes");
    }

    #[test]
    fn test_pluralize_unit_minutes() {
        assert_eq!(pluralize_unit(" minute", 0), " minutes");
        assert_eq!(pluralize_unit(" minute", 1), " minute");
        assert_eq!(pluralize_unit(" minute", 2), " minutes");
        assert_eq!(pluralize_unit(" minute", 100), " minutes");
    }

    #[test]
    fn test_pluralize_unit_hours() {
        assert_eq!(pluralize_unit(" hour", 0), " hours");
        assert_eq!(pluralize_unit(" hour", 1), " hour");
        assert_eq!(pluralize_unit(" hour", 2), " hours");
        assert_eq!(pluralize_unit(" hour", 24), " hours");
    }

    #[test]
    fn test_pluralize_unit_unknown() {
        assert_eq!(pluralize_unit("unknown", 1), "");
        assert_eq!(pluralize_unit("unknown", 2), "s");
    }

    #[test]
    fn test_format_time_until_minutes() {
        let now = Local::now();
        let future = now + chrono::Duration::minutes(5);
        let result = format_time_until(future);
        // Check for the general format, allowing for minor time variations
        assert!(result.starts_with("in "));
        assert!(result.contains("minute"));
    }

    #[test]
    fn test_format_time_until_hours_and_minutes() {
        let now = Local::now();
        let future = now + chrono::Duration::hours(2) + chrono::Duration::minutes(30);
        let result = format_time_until(future);
        // Check for the general format
        assert!(result.starts_with("in "));
        assert!(result.contains("hour"));
        assert!(result.contains("minute"));
    }

    #[test]
    fn test_format_time_until_very_soon() {
        let now = Local::now();
        let future = now + chrono::Duration::seconds(30);
        let result = format_time_until(future);
        assert_eq!(result, "very soon");
    }

    #[test]
    fn test_format_time_until_past() {
        let now = Local::now();
        let past = now - chrono::Duration::minutes(10);
        let result = format_time_until(past);
        assert_eq!(result, "very soon");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_parse_next_run_timestamp_valid() {
        let output = "NextElapseUSecRealtime=1700000000000000\n";
        let result = parse_next_run_timestamp(output);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_parse_next_run_timestamp_invalid() {
        let output = "NextElapseUSecRealtime=invalid\n";
        let result = parse_next_run_timestamp(output);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_parse_next_run_timestamp_missing_prefix() {
        let output = "SomeOtherValue=1234567890000000\n";
        let result = parse_next_run_timestamp(output);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_parse_next_run_timestamp_empty() {
        let output = "";
        let result = parse_next_run_timestamp(output);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}
