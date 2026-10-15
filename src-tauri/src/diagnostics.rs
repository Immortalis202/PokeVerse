use std::{
    io::Write,
    path::PathBuf,
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};
static LOG: OnceLock<Mutex<PathBuf>> = OnceLock::new();
pub fn initialize(directory: &std::path::Path) {
    let _ = std::fs::create_dir_all(directory.join("logs"));
    let _ = LOG.set(Mutex::new(directory.join("logs/app.log")));
    record("startup engine=0.2.0-core");
}
// Deliberately record codes only: no room token, credentials, character notes, or payloads.
pub fn record(code: &str) {
    if let Some(path) = LOG.get().and_then(|l| l.lock().ok()) {
        if std::fs::metadata(&*path).is_ok_and(|m| m.len() > 1_000_000) {
            let _ = std::fs::write(&*path, b"");
        }
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&*path)
        {
            let time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |d| d.as_secs());
            let _ = writeln!(
                file,
                "{} {}",
                time,
                code.chars()
                    .filter(|c| c.is_ascii_alphanumeric() || " _=-.".contains(*c))
                    .take(120)
                    .collect::<String>()
            );
        }
    }
}
