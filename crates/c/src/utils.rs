use std::io::Read;

/// Reads stdin and retusn a string with its content.
pub fn read_from_stdin() -> Result<String, std::io::Error> {
    let mut stdin = Vec::new();
    tracing::event!(tracing::Level::INFO, "Reading from stdin...");
    let mut lock = std::io::stdin().lock();
    lock.read_to_end(&mut stdin)?;
    Ok(String::from_utf8_lossy(&stdin).to_string())
}
