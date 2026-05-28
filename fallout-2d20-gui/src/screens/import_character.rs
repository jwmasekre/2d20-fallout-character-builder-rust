use imgui::Ui;
use sdl2::video::Window;
use uuid::Uuid;

use fallout_2d20_core::{
    db::Db, states::{ImportState, ImportStep}, structs::{AppConfig, AppScreen}
};

pub fn render_import_character(
    ui: &Ui,
    window: &Window,
    state: &mut ImportState,
    screen: &mut AppScreen,
    db: &Db,
    cfg: &AppConfig,
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
            let w = 440.0 * cfg.ui_scale;
            let h = 160.0 * cfg.ui_scale;

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
            let w = 440.0 * cfg.ui_scale;
            let h = 140.0 * cfg.ui_scale;
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