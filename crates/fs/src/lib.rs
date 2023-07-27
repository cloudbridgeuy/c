use std::env;
use std::fs;
use std::path::Path;

/// Checks if a file exists.
pub fn file_exists(filename: &str) -> bool {
    fs::metadata(filename).is_ok()
}

/// Chacks if a directory exists.
pub fn directory_exists(dir_name: &str) -> bool {
    let path = Path::new(dir_name);
    path.exists() && path.is_dir()
}

/// Get HOME directory.
pub fn get_home_directory() -> String {
    match env::var("HOME") {
        Ok(val) => val + "/.c/sessions",
        Err(_) => String::from("/tmp/.c/sessions"),
    }
}
