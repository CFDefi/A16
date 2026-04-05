//! OS Standard Library
//!
//! Operating system utilities.

/// Get an environment variable
pub fn env_get(key: &str) -> Option<String> {
    std::env::var(key).ok()
}

/// Get an environment variable with default
pub fn env_get_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Get current working directory
pub fn cwd() -> Result<String, String> {
    std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| format!("Failed to get cwd: {}", e))
}

/// Get the OS name
pub fn os_name() -> &'static str {
    std::env::consts::OS
}

/// Get the CPU architecture
pub fn arch() -> &'static str {
    std::env::consts::ARCH
}

/// Get platform-specific path separator
pub fn path_separator() -> &'static str {
    if cfg!(windows) { "\\" } else { "/" }
}

/// Get process ID
pub fn pid() -> u32 {
    std::process::id()
}

/// Get current timestamp (seconds since epoch)
pub fn timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Get current timestamp in milliseconds
pub fn timestamp_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

/// Get command line arguments
pub fn args() -> Vec<String> {
    std::env::args().collect()
}

/// Exit program with code
pub fn exit(code: i32) {
    std::process::exit(code)
}

/// Sleep for milliseconds
pub fn sleep_ms(ms: u64) {
    std::thread::sleep(std::time::Duration::from_millis(ms));
}
