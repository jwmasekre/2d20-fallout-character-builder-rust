use std::path::PathBuf;
use imgui::Ui;
use sdl2::video::Window;
use uuid::Uuid;
use crate::{AppScreen, character::Character, db::Db};

#[derive(Debug, Clone, PartialEq)]
pub enum ImportStep {
    Idle,
    Confirm(Character),      // file loaded, ask about overwrite
    Done,
    Error(String),
}

pub struct ImportState {
    pub step: ImportStep,
}

impl ImportState {
    pub fn new() -> Self {
        Self { step: ImportStep::Idle }
    }

    pub fn reset(&mut self) {
        self.step = ImportStep::Idle;
    }

    /// Returns true if the character id already exists in the db
    fn id_exists(db: &Db, id: &str) -> bool {
        db.block_on(async {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM characters WHERE id = ?", id
            ).fetch_one(&db.pool).await
        }).unwrap_or(0) > 0
    }

    /// Try to load a json file into a Character struct
    pub fn load_from_file(path: &PathBuf) -> Result<Character, String> {
        let raw = std::fs::read_to_string(path)
            .map_err(|e| format!("Could not read file: {e}"))?;
        serde_json::from_str::<Character>(&raw)
            .map_err(|e| format!("Invalid character JSON: {e}"))
    }
}

pub fn render_import_character(
    ui: &Ui,
    window: &Window,
    state: &mut ImportState,
    screen: &mut AppScreen,
    db: &Db,
) {
    let (win_w, win_h) = window.size();

    // kick off the file dialog immediately on first render
    // (Idle means we haven't started yet)
    if state.step == ImportStep::Idle {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("JSON", &["json"])
            .pick_file()
        else {
            // user cancelled the file dialog
            *screen = AppScreen::MainMenu;
            return;
        };

        match ImportState::load_from_file(&path) {
            Err(e) => {
                state.step = ImportStep::Error(e);
            }
            Ok(character) => {
                let id = character.id.to_string();
                if ImportState::id_exists(db, &id) {
                    state.step = ImportStep::Confirm(character);
                } else {
                    // no conflict — save immediately
                    match db.save_character(&character) {
                        Ok(_) => { state.step = ImportStep::Done; }
                        Err(e) => { state.step = ImportStep::Error(e.to_string()); }
                    }
                }
            }
        }
    }

    match &state.step {
        ImportStep::Idle => {} // handled above

        // ── Overwrite confirmation ────────────────────────────────────
        ImportStep::Confirm(_) => {
            let w = 440.0_f32;
            let h = 160.0_f32;

            // clone name out before mutable borrow below
            let char_name = if let ImportStep::Confirm(c) = &state.step {
                c.name.clone()
            } else { String::new() };

            ui.window("##import_confirm")
                .title_bar(false)
                .resizable(false)
                .movable(false)
                .size([w, h], imgui::Condition::Always)
                .position(
                    [(win_w as f32 - w) * 0.5, (win_h as f32 - h) * 0.5],
                    imgui::Condition::Always,
                )
                .build(|| {
                    ui.text("Character already exists");
                    ui.separator();
                    ui.spacing();
                    ui.text_wrapped(&format!(
                        "\"{}\" already exists in your database. What would you like to do?",
                        char_name
                    ));
                    ui.spacing();
                    ui.separator();
                    ui.spacing();

                    // Overwrite
                    if ui.button("Overwrite##imp_overwrite") {
                        if let ImportStep::Confirm(character) = &state.step {
                            let character = character.clone();
                            match db.save_character(&character) {
                                Ok(_) => { *screen = AppScreen::MainMenu; }
                                Err(e) => { state.step = ImportStep::Error(e.to_string()); }
                            }
                        }
                    }

                    ui.same_line();

                    // Save as new (generate fresh id)
                    if ui.button("Save as New##imp_new") {
                        if let ImportStep::Confirm(character) = &state.step {
                            let mut character = character.clone();
                            character.id = Uuid::now_v7();
                            match db.save_character(&character) {
                                Ok(_) => { *screen = AppScreen::MainMenu; }
                                Err(e) => { state.step = ImportStep::Error(e.to_string()); }
                            }
                        }
                    }

                    ui.same_line();

                    if ui.button("Cancel##imp_cancel") {
                        *screen = AppScreen::MainMenu;
                    }
                });
        }

        // ── Error display ─────────────────────────────────────────────
        ImportStep::Error(msg) => {
            let w = 440.0_f32;
            let h = 140.0_f32;
            let msg = msg.clone();

            ui.window("##import_error")
                .title_bar(false)
                .resizable(false)
                .movable(false)
                .size([w, h], imgui::Condition::Always)
                .position(
                    [(win_w as f32 - w) * 0.5, (win_h as f32 - h) * 0.5],
                    imgui::Condition::Always,
                )
                .build(|| {
                    ui.text("Import Failed");
                    ui.separator();
                    ui.spacing();
                    ui.text_colored([1.0, 0.3, 0.3, 1.0], "Error:");
                    ui.same_line();
                    ui.text_wrapped(&msg);
                    ui.spacing();
                    ui.separator();
                    ui.spacing();
                    if ui.button("OK##imp_err_ok") {
                        *screen = AppScreen::MainMenu;
                    }
                    ui.same_line();
                    if ui.button("Try Again##imp_retry") {
                        state.reset();
                    }
                });
        }

        ImportStep::Done => {
            *screen = AppScreen::MainMenu;
        }
    }
}