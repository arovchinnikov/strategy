use std::fs;
use std::path::PathBuf;

pub fn init_dir(path: PathBuf) -> Result<(), std::io::Error> {
    fs::create_dir_all(path)?;
    Ok(())
}

pub fn cache_directory() -> PathBuf {
    dirs::cache_dir()
        .expect("Failed to get dir directory")
        .join("fallen-age")
        .join("cache")
}
