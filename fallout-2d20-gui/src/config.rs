use std::path::PathBuf;
use std::fs;

pub fn db_path(config_path: PathBuf) -> PathBuf {
    if config_path.exists() { return config_path }
    let userdata_path = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("fallout-2d20-builder")
        .join("fallout_2d20.db");
    if userdata_path.exists() {
        return userdata_path
    }
    let exe_path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("fallout_2d20.db");
    //if exe_path.exists() {
        return exe_path
    //}
}

const CONFIG_FILE: &str = "usr_config.toml";

pub struct AppConfig {
    pub theme_index: usize,
    pub db_path: PathBuf,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme_index: 0,
            db_path: std::env::current_exe().unwrap().parent().unwrap().join("fallout_2d20.db"),
        }
    }
}

fn config_path() -> PathBuf {
    //just lets us have a persistent config when runing under cargo
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        //println!("running under cargo");
        return PathBuf::from(manifest_dir).join(CONFIG_FILE);
    }
    if let Ok(mut exe) = std::env::current_exe() {
        exe.pop();
        exe.push(CONFIG_FILE);
        exe
    } else {
        PathBuf::from(CONFIG_FILE)
    }
}

pub fn load_config() -> AppConfig {
    let path = config_path();
    let Ok(contents) = fs::read_to_string(&path) else {
        return AppConfig::default();
    };
    let mut cfg = AppConfig::default();
    for line in contents.lines() {
        let line = line.trim();
        if let Some(val) = line.strip_prefix("theme_index=") {
            if let Ok(i) = val.trim().parse::<usize>() {
                cfg.theme_index = i;
            }
        }
        if let Some(val) = line.strip_prefix("db_path=") {
            let db_path = PathBuf::from(val);
            if db_path.is_file() {
                cfg.db_path = db_path;
            }
        }
    }
    cfg
}

pub fn save_config(cfg: &AppConfig) {
    let path = config_path();
    let contents = format!("theme_index={}\ndb_path={:?}\n", cfg.theme_index, cfg.db_path);
    if let Err(e) = fs::write(&path, contents) {
        eprintln!("Failed to save config: {e}");
    }
}