use imgui::Ui;
use sdl2::video::Window;
use crate::{AppScreen, db::Db, character::Character};

pub struct LoadCharacterState {
    pub characters: Vec<(String, String, String)>, // (id, name, player_name)
    pub loaded: bool,
    pub selected: Option<usize>,
    pub error: Option<String>,
}

impl LoadCharacterState {
    pub fn new() -> Self {
        Self {
            characters: vec![],
            loaded: false,
            selected: None,
            error: None,
        }
    }

    pub fn reset(&mut self) {
        self.characters = vec![];
        self.loaded = false;
        self.selected = None;
        self.error = None;
    }

    pub fn load_list(&mut self, db: &Db) {
        if self.loaded { return; }
        let rows = db.block_on(async {
            sqlx::query!(
                r#"SELECT c.id, c.character_name, p.username
                   FROM characters c
                   JOIN players p ON p.id = c.player_id
                   ORDER BY p.username, c.character_name"#
            ).fetch_all(&db.pool).await
        }).unwrap_or_default();

        self.characters = rows.into_iter().map(|r| (
            r.id.unwrap_or_default(),
            r.character_name.unwrap_or_else(|| "(unnamed)".to_string()),
            r.username.unwrap_or_else(|| "(unknown player)".to_string()),
        )).collect();
        self.loaded = true;
    }
}

pub fn render_load_character(
    ui: &Ui,
    window: &Window,
    state: &mut LoadCharacterState,
    screen: &mut AppScreen,
    db: &Db,
    character: &mut Character,
) {
    state.load_list(db);

    let (win_w, win_h) = window.size();
    let w = 480.0_f32;
    let h = 360.0_f32;

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
                let list_h = h - 120.0;
                ui.child_window("##char_list")
                    .size([w - 32.0, list_h])
                    .build(|| {
                        for (i, (_, name, player)) in state.characters.iter().enumerate() {
                            let is_sel = state.selected == Some(i);
                            let label = format!("{} ({})", name, player);
                            if ui.selectable_config(&format!("{}##char_{}", label, i))
                                .selected(is_sel)
                                .build()
                            {
                                state.selected = Some(i);
                            }
                        }
                    });
            }

            // error display
            if let Some(ref err) = state.error {
                ui.spacing();
                ui.text_colored([1.0, 0.3, 0.3, 1.0], err);
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