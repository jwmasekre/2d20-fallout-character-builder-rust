use std::path::PathBuf;

pub fn db_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("fallout-2d20-builder")
        .join("fallout_2d20.db")
}