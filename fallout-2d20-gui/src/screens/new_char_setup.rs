use imgui::Ui;
use sdl2::video::Window;
use uuid::Uuid;
use fallout_2d20_core::{
    character::{
        Character,
        Party,
        Player,
    },
    constants::NULL_PARTY,
    db::Db,
    states::{
        NewCharacterSetupState,
        PartyChoice,
        PlayerChoice,
    },
    structs::{AppConfig, AppScreen},
};

pub fn render_new_character_setup(
    ui: &Ui,
    window: &Window,
    state: &mut NewCharacterSetupState,
    screen: &mut AppScreen,
    db: &Db,
    // out-params — caller uses these when setup completes
    out_player_name: &mut Option<String>,
    out_party_name: &mut Option<String>,
    character: &mut Character,
    cfg: &AppConfig,
) {
    let (win_w, win_h) = window.size();
    let w = 480.0 * cfg.ui_scale;
    let h = 360.0 * cfg.ui_scale;

    ui.window("##nc_setup")
        .title_bar(false)
        .resizable(false)
        .movable(false)
        .size([w, h], imgui::Condition::Always)
        .position(
            [(win_w as f32 - w) * 0.5, (win_h as f32 - h) * 0.5],
            imgui::Condition::Always,
        )
        .build(|| {
            // ── Step indicator ────────────────────────────────────
            let steps = ["1. Player", "2. Party"];
            for (i, label) in steps.iter().enumerate() {
                if i == state.step {
                    ui.text(label);
                } else {
                    ui.text_disabled(label);
                }
                if i < steps.len() - 1 { ui.same_line(); ui.text(" › "); ui.same_line(); }
            }
            ui.separator();
            ui.spacing();

            match state.step {
                // ── STEP 0: Player ────────────────────────────────
                0 => {
                    state.load_players(db);
                    ui.text("Who is playing this character?");
                    ui.spacing();

                    // New player option
                    let new_selected = state.player_choice == PlayerChoice::New;
                    if ui.radio_button_bool("New player##np", new_selected) {
                        state.player_choice = PlayerChoice::New;
                    }
                    if state.player_choice == PlayerChoice::New {
                        ui.same_line();
                        ui.set_next_item_width(w - 160.0);
                        ui.input_text("##np_name", &mut state.new_player_name).build();
                    }
                    ui.spacing();

                    // Existing players
                    if state.players.is_empty() {
                        ui.text_disabled("  (no existing players in database)");
                    } else {
                        ui.text("Existing players:");
                        ui.spacing();
                        let list_h = (h - 180.0 * cfg.ui_scale).max(80.0 * cfg.ui_scale);
                        ui.child_window("##player_list")
                            .size([w - 32.0 * cfg.ui_scale, list_h])
                            .build(|| {
                                for (id, name) in &state.players {
                                    let is_sel = state.player_choice == PlayerChoice::Existing(id.clone());
                                    if ui.radio_button_bool(
                                        &format!("{}##pid_{}", name, id),
                                        is_sel,
                                    ) {
                                        state.player_choice = PlayerChoice::Existing(id.clone());
                                    }
                                }
                            });
                    }

                    ui.spacing();
                    ui.separator();
                    ui.spacing();

                    // Footer nav
                    let can_next = match &state.player_choice {
                        PlayerChoice::New => !state.new_player_name.trim().is_empty(),
                        PlayerChoice::Existing(_) => true,
                        PlayerChoice::Unset => false,
                    };

                    if ui.button("Cancel##nc_cancel") {
                        *screen = AppScreen::MainMenu;
                    }
                    ui.same_line();
                    let _d = if !can_next { Some(ui.begin_disabled(true)) } else { None };
                    if ui.button("Next >##nc_next") {
                        state.step = 1;
                    }
                    drop(_d);
                }

                // ── STEP 1: Party ─────────────────────────────────
                1 => {
                    state.load_parties(db);
                    ui.text("Party assignment:");
                    ui.spacing();

                    // No party
                    if ui.radio_button_bool(
                        "No party (solo)##party_none",
                        state.party_choice == PartyChoice::None,
                    ) {
                        state.party_choice = PartyChoice::None;
                    }
                    ui.spacing();

                    // New party
                    let new_sel = state.party_choice == PartyChoice::New;
                    if ui.radio_button_bool("New party##party_new", new_sel) {
                        state.party_choice = PartyChoice::New;
                    }
                    if state.party_choice == PartyChoice::New {
                        ui.same_line();
                        ui.set_next_item_width(w - 160.0 * cfg.ui_scale);
                        ui.input_text("##party_name", &mut state.new_party_name).build();
                    }
                    ui.spacing();

                    // Existing parties
                    if !state.parties.is_empty() {
                        ui.text("Existing parties:");
                        ui.spacing();
                        let list_h = (h - 220.0 * cfg.ui_scale).max(60.0 * cfg.ui_scale);
                        ui.child_window("##party_list")
                            .size([w - 32.0 * cfg.ui_scale, list_h])
                            .build(|| {
                                for (id, name) in &state.parties {
                                    let is_sel = state.party_choice == PartyChoice::Existing(id.clone());
                                    if ui.radio_button_bool(
                                        &format!("{}##pid_{}", name, id),
                                        is_sel,
                                    ) {
                                        state.party_choice = PartyChoice::Existing(id.clone());
                                    }
                                }
                            });
                    }

                    ui.spacing();
                    ui.separator();
                    ui.spacing();

                    let can_confirm = match &state.party_choice {
                        PartyChoice::New => !state.new_party_name.trim().is_empty(),
                        PartyChoice::None | PartyChoice::Existing(_) => true,
                        PartyChoice::Unset => false,
                    };

                    if ui.button("< Back##nc_back") {
                        state.step = 0;
                    }
                    ui.same_line();
                    if ui.button("Cancel##nc_cancel2") {
                        *screen = AppScreen::MainMenu;
                    }
                    ui.same_line();
                    let _d = if !can_confirm { Some(ui.begin_disabled(true)) } else { None };
                    if ui.button("Begin >##nc_begin") {
                        // Write out-params so the caller can finish setup
                        let player_id = match &state.player_choice {
                            PlayerChoice::New => {
                                let name = state.new_player_name.trim().to_string();
                                let id = db.create_player(&name);
                                *out_player_name = Some(name);
                                id
                            }
                            PlayerChoice::Existing(id) => {
                                *out_player_name = state.players.iter()
                                    .find(|(pid, _)| pid == id)
                                    .map(|(_, n)| n.clone());
                                Uuid::parse_str(&id.clone()).ok().unwrap()
                            }
                            _ => unreachable!(),
                        };
                        let party_id = match &state.party_choice {
                            PartyChoice::None => {
                                *out_party_name = Some("Solo".to_string());
                                Uuid::parse_str(NULL_PARTY).ok().unwrap()
                            }
                            PartyChoice::New => {
                                let name = state.new_party_name.trim().to_string();
                                let id = db.create_party(&name);
                                *out_party_name = Some(name);
                                id
                            }
                            PartyChoice::Existing(id) => {
                                *out_party_name = state.parties.iter()
                                    .find(|(pid, _)| pid == id)
                                    .map(|(_, n)| n.clone());
                                Uuid::parse_str(&id.clone()).ok().unwrap()
                            }
                            _ => unreachable!()
                        };
                        character.player = Player {
                            id: player_id,
                            name: out_player_name.clone().unwrap_or("".to_string()),
                        };
                        character.party = Party {
                            id: party_id,
                            name: out_party_name.clone().unwrap_or("".to_string()),
                            ap_players: 6,
                            ap_gm: 0,
                            max_ap: 6,
                        };
                        *screen = AppScreen::OriginSelect;
                    }
                    drop(_d);
                }
                _ => {}
            }
        });
}