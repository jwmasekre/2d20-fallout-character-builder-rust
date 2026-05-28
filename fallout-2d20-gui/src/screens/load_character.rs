use imgui::Ui;
use sdl2::video::Window;
use fallout_2d20_core::{
    character::Character,
    db::Db,
    states::{
        LoadCharacterState,
        SheetState
    },
    structs::{AppConfig, AppScreen}
};

pub fn render_load_character(
    ui: &Ui,
    window: &Window,
    state: &mut LoadCharacterState,
    screen: &mut AppScreen,
    db: &Db,
    character: &mut Character,
    sheet_state: &mut SheetState,
    cfg: &AppConfig,
) {
    state.load_list(db);

    let (win_w, win_h) = window.size();
    let w = 480.0 * cfg.ui_scale;
    let h = 360.0 * cfg.ui_scale;

    ui.window("##load_character")
        .title_bar(false)
        .resizable(false)
        .movable(false)
        .size([w, h], imgui::Condition::Always)
        .position(
            [(win_w as f32 - w) * 0.5, (win_h as f32 - h) * 0.5],
            imgui::Condition::Always,
        )
        .build(|| {
            ui.text("Load Character");
            ui.separator();
            ui.spacing();

            if state.characters.is_empty() {
                ui.text_disabled("No saved characters found.");
            } else {
                let list_h = h - 120.0 * cfg.ui_scale;
                ui.child_window("##char_list")
                    .size([w - 32.0 * cfg.ui_scale, list_h])
                    .build(|| {
                        for (i, (_, name, player)) in state.characters.iter().enumerate() {
                            let is_sel = state.selected == Some(i);
                            let label = format!("{} ({})", name, player);
                            if ui.selectable_config(&format!("{}##char_{}", label, i))
                                .selected(is_sel)
                                .size([w - 180.0 * cfg.ui_scale, 0.0])
                                .build()
                            {
                                state.selected = Some(i);
                            }
                            ui.same_line_with_pos(w - 172.0);
                            let del_label = format!("Delete##del_{}", i);
                            let c = ui.push_style_color(
                                imgui::StyleColor::Button,
                                [0.55, 0.1, 0.1, 1.0],
                            );
                            let c2 = ui.push_style_color(
                                imgui::StyleColor::ButtonHovered,
                                [0.75, 0.15, 0.15, 1.0],
                            );
                            if ui.button(&del_label) {
                                state.confirm_delete = Some(i);
                            }
                            drop(c);
                            drop(c2);
                        }
                    });
            }

            if let Some(index) = state.confirm_delete {
                let (id, name, _) = &state.characters[index];
                let id = id.clone();
                let name = name.clone();

                let pw = 340.0 * cfg.ui_scale;
                let ph = 160.0 * cfg.ui_scale;
                ui.window("##confirm_delete")
                    .title_bar(false)
                    .resizable(false)
                    .movable(false)
                    .size([pw, ph], imgui::Condition::Always)
                    .position(
                        [(win_w as f32 - pw) * 0.5, (win_h as f32 - ph) * 0.5],
                        imgui::Condition::Always,
                    )
                    .build(|| {
                        ui.text("Confirm Delete");
                        ui.separator();
                        ui.spacing();
                        ui.text_wrapped(&format!("Permanently delete \"{}\"? This cannot be undone.", name));
                        ui.spacing();
                        ui.separator();
                        ui.spacing();

                        let c = ui.push_style_color(imgui::StyleColor::Button, [0.55, 0.1, 0.1, 1.0]);
                        let c2 = ui.push_style_color(imgui::StyleColor::ButtonHovered, [0.75, 0.15, 0.15, 1.0]);
                        if ui.button("Delete##confirm_del") {
                            match db.delete_character(&id) {
                                Ok(_) => {
                                    state.error = None;
                                    state.confirm_delete = None;
                                    state.loaded = false; // force list reload
                                    // if the deleted character was selected, clear selection
                                    if state.selected == Some(index) {
                                        state.selected = None;
                                    }
                                }
                                Err(e) => {
                                    state.error = Some(format!("Delete failed: {e}"));
                                    state.confirm_delete = None;
                                }
                            }
                        }
                        drop(c); drop(c2);

                        ui.same_line();
                        if ui.button("Cancel##cancel_del") {
                            state.confirm_delete = None;
                        }
                    });
            }

            // error display
            if let Some(ref err) = state.error {
                ui.spacing();
                let ec = ui.push_style_color(imgui::StyleColor::Text, [1.0, 0.3, 0.3, 1.0]);
                ui.text_wrapped(err);
                drop(ec);
            }

            ui.spacing();
            ui.separator();
            ui.spacing();

            if ui.button("Cancel##lc_cancel") {
                *screen = AppScreen::MainMenu;
            }

            ui.same_line();

            let can_load = state.selected.is_some();
            let _d = if !can_load { Some(ui.begin_disabled(true)) } else { None };
            if ui.button("Load##lc_load") {
                if let Some(idx) = state.selected {
                    let id = state.characters[idx].0.clone();
                    match db.load_character(&id) {
                        Ok(loaded) => {
                            *character = loaded;
                            state.error = None;
                            sheet_state.new_character(character);
                            *screen = AppScreen::CharacterSheet;
                        }
                        Err(e) => {
                            state.error = Some(format!("Failed to load: {e}"));
                        }
                    }
                }
            }
            drop(_d);
        });
}