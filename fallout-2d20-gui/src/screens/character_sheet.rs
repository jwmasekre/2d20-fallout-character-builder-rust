
use imgui::Ui;
use sdl2::video::Window;
use std::path::Path;
use anyhow::Result;
use crate::{AppScreen, character::Character, db::Db, log_on_change, theme::render_window};

pub fn export_character(character: &Character, path: &Path) -> Result<()> {
    let json = serde_json::to_string_pretty(character)?;
    std::fs::write(path, json)?;
    Ok(())
}

pub fn sanitize_filename(name: &str) -> String {
    let reserved_names = [
        "CON", "PRN", "AUX", "NUL",
        "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
        "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    let sanitized: String = name.chars().map(|c| match c {
        '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
        c if c.is_control() => '_',
        ' ' => '_',
        c => c,
    }).collect();
    let sanitized = sanitized.trim_matches('.').to_string();
    let upper = sanitized.to_uppercase();
    let base = upper.split('.').next().unwrap_or("");
    if sanitized.is_empty() || reserved_names.contains(&base) {
        "character".to_string()
    } else {
        sanitized
    }
}

pub fn render_character_sheet(
    ui: &Ui,
    window: &Window,
    _db: &Db,
    character: &Character,
    screen: &mut AppScreen,
) -> f32 {
    let Some((w, h, _token)) = render_window(ui, window, "##character_sheet", "Character Sheet", screen)
        else { return 0.0 };

    log_on_change!(character);

    ui.text(format!("{} --- {} ({})", character.name, character.player.name, character.party.name));
    ui.same_line_with_pos(w - 80.0);
    if ui.button("Export##export") {
        let default_name = format!("{}.json", sanitize_filename(&character.name));
        if let Some(path) = rfd::FileDialog::new()
            .set_file_name(&default_name)
            .add_filter("JSON", &["json"])
            .save_file()
        {
            match export_character(&character, &path) {
                Ok(_) => {},
                Err(e) => eprintln!("Export failed: {e}"),
            };
        }
    }
    ui.separator();
    ui.spacing();



    h
}
