use std::path::PathBuf;
use std::fs;
use fallout_2d20_core::constants::CONFIG_FILE;
use fallout_2d20_core::structs::AppConfig;

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

fn config_path() -> PathBuf {
    //just lets us have a persistent config when runing under cargo
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let path = PathBuf::from(manifest_dir).join(CONFIG_FILE);
        return path;
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
        let pos = line.find('=');
        if pos.is_none() { continue };
        let prefix = line[..pos.unwrap()].to_string();
        let spos = pos.unwrap() + 1;
        let suffix = line[spos..].trim().to_string();
        match prefix.as_str() {
            "theme_index" => cfg.theme_index = suffix.clone().parse::<usize>().ok().unwrap(),
            "db_path" => cfg.db_path = if PathBuf::from(suffix.clone()).is_file() { PathBuf::from(suffix.clone()) } else { AppConfig::default().db_path },
            "font_path" => cfg.font_path = if PathBuf::from(suffix.clone()).is_file() { Some(PathBuf::from(suffix.clone())) } else { None },
            "font_size" => cfg.font_size = suffix.clone().parse::<f32>().ok().unwrap(),
            "crt_distortion" => cfg.crt_distortion = suffix.clone().parse::<f32>().ok().unwrap(),
            "crt_scanline_strength" => cfg.crt_scanline_strength = suffix.clone().parse::<f32>().ok().unwrap(),
            "crt_vignette_multiplier" => cfg.crt_vignette_multiplier = suffix.clone().parse::<f32>().ok().unwrap(),
            "crt_vignette_exponent" => cfg.crt_vignette_exponent = suffix.clone().parse::<f32>().ok().unwrap(),
            "crt_roll_speed" => cfg.crt_roll_speed = suffix.clone().parse::<f32>().ok().unwrap(),
            "crt_tint_strength" => cfg.crt_tint_strength = suffix.clone().parse::<f32>().ok().unwrap(),
            "crt_chromatic_aberration" => cfg.crt_chromatic_aberration = suffix.clone().parse::<f32>().ok().unwrap(),
            _ => continue,
        }
    }
    cfg
}

pub fn save_config(cfg: &AppConfig) {
    let path = config_path();
    let contents = format!(
"theme_index={}
db_path={:?}
font_path={:?}
font_size={}
crt_distortion={}
crt_scanline_strength={}
crt_vignette_multiplier={}
crt_vignette_exponent={}
crt_roll_speed={}
crt_tint_strength={}
crt_chromatic_aberration={}
",
        cfg.theme_index,
        cfg.db_path,
        cfg.font_path,
        cfg.font_size,
        cfg.crt_distortion,
        cfg.crt_scanline_strength,
        cfg.crt_vignette_multiplier,
        cfg.crt_vignette_exponent,
        cfg.crt_roll_speed,
        cfg.crt_tint_strength,
        cfg.crt_chromatic_aberration
    );
    if let Err(e) = fs::write(&path, contents) {
        eprintln!("Failed to save config: {e}");
    }
}