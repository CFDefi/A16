//! I/O Standard Library
//!
//! File reading/writing and path utilities.

use std::path::Path;

/// Read a file to string
pub fn read_file(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read '{}': {}", path, e))
}

/// Write a string to a file
pub fn write_file(path: &str, content: &str) -> Result<(), String> {
    std::fs::write(path, content)
        .map_err(|e| format!("Failed to write '{}': {}", path, e))
}

/// Append a string to a file
pub fn append_file(path: &str, content: &str) -> Result<(), String> {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("Failed to open '{}': {}", path, e))?;
    file.write_all(content.as_bytes())
        .map_err(|e| format!("Failed to write '{}': {}", path, e))
}

/// Read file lines
pub fn read_lines(path: &str) -> Result<Vec<String>, String> {
    let content = read_file(path)?;
    Ok(content.lines().map(|l| l.to_string()).collect())
}

/// Check if a file exists
pub fn file_exists(path: &str) -> bool {
    Path::new(path).exists()
}

/// Check if a path is a file
pub fn is_file(path: &str) -> bool {
    Path::new(path).is_file()
}

/// Check if a path is a directory
pub fn is_dir(path: &str) -> bool {
    Path::new(path).is_dir()
}

/// Get file extension
pub fn file_extension(path: &str) -> Option<String> {
    Path::new(path).extension().map(|e| e.to_string_lossy().to_string())
}

/// Get file stem (name without extension)
pub fn file_stem(path: &str) -> Option<String> {
    Path::new(path).file_stem().map(|s| s.to_string_lossy().to_string())
}

/// Get parent directory
pub fn parent_dir(path: &str) -> Option<String> {
    Path::new(path).parent().map(|p| p.to_string_lossy().to_string())
}

/// Join paths
pub fn join_path(base: &str, child: &str) -> String {
    Path::new(base).join(child).to_string_lossy().to_string()
}

/// List files in a directory
pub fn list_dir(path: &str) -> Result<Vec<String>, String> {
    let entries = std::fs::read_dir(path)
        .map_err(|e| format!("Failed to read dir '{}': {}", path, e))?;

    let mut result = Vec::new();
    for entry in entries {
        if let Ok(entry) = entry {
            result.push(entry.path().to_string_lossy().to_string());
        }
    }
    Ok(result)
}

/// Get file size in bytes
pub fn file_size(path: &str) -> Result<u64, String> {
    std::fs::metadata(path)
        .map(|m| m.len())
        .map_err(|e| format!("Failed to stat '{}': {}", path, e))
}

/// Create a directory (including parents)
pub fn mkdir(path: &str) -> Result<(), String> {
    std::fs::create_dir_all(path)
        .map_err(|e| format!("Failed to mkdir '{}': {}", path, e))
}

/// Remove a file
pub fn remove_file(path: &str) -> Result<(), String> {
    std::fs::remove_file(path)
        .map_err(|e| format!("Failed to remove '{}': {}", path, e))
}

/// Copy a file
pub fn copy_file(from: &str, to: &str) -> Result<u64, String> {
    std::fs::copy(from, to)
        .map_err(|e| format!("Failed to copy '{}' to '{}': {}", from, to, e))
}
