use std::fs;
use std::path::Path;

#[cfg(target_os = "macos")]
const SYSTEM_SOUNDS_DIR: &str = "/System/Library/Sounds";

#[cfg(target_os = "linux")]
const LINUX_SOUNDS_DIRS: &[&str] = &[
    "/usr/share/sounds/freedesktop/stereo",
    "/usr/share/sounds/gnome/default/alerts",
    "/usr/share/sounds/ubuntu/stereo",
];

pub fn get_available_sounds() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    #[cfg(target_os = "macos")]
    return get_macos_sounds();

    #[cfg(target_os = "linux")]
    return get_linux_sounds();

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    return Err("Sound selection not supported on this platform".into());
}

#[cfg(target_os = "macos")]
fn get_macos_sounds() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut sounds: Vec<String> = fs::read_dir(SYSTEM_SOUNDS_DIR)?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_file())
        .filter_map(|entry| extract_sound_name(&entry.path(), &[".aiff"]))
        .collect();

    sounds.sort();
    Ok(sounds)
}

#[cfg(target_os = "linux")]
fn get_linux_sounds() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let extensions = [".oga", ".ogg", ".wav"];

    let mut sounds: Vec<String> = LINUX_SOUNDS_DIRS
        .iter()
        .filter_map(|dir| fs::read_dir(dir).ok())
        .flat_map(|entries| entries.filter_map(Result::ok))
        .filter(|entry| entry.path().is_file())
        .filter_map(|entry| extract_sound_name(&entry.path(), &extensions))
        .collect();

    sounds.sort();
    sounds.dedup();

    if sounds.is_empty() {
        return Err("No sounds found in Linux sound directories".into());
    }

    Ok(sounds)
}

fn extract_sound_name(path: &Path, extensions: &[&str]) -> Option<String> {
    let file_name = path.file_name()?.to_str()?;

    extensions
        .iter()
        .find_map(|ext| file_name.strip_suffix(ext))
        .map(String::from)
}
