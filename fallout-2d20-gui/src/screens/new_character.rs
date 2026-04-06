use imgui::Ui;
use sdl2::video::Window;
use crate::AppScreen;
use crate::db::Db;
use crate::BAR_HEIGHT;
use crate::screens::special::MutantType;

// ── Data types loaded from DB ─────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct OriginRow {
    pub id: i64,
    pub name: String,
    pub sourcebook: String,
    pub description: String,
    pub can_ghoul: bool,
}

#[derive(Debug, Clone)]
pub struct TraitRow {
    pub id: i64,
    pub origin_id: i64,
    pub name: String,
    pub description: String,
    pub is_ghoul_trait: bool,
}

// ── Screen state ──────────────────────────────────────────────────────────────

pub struct NewCharacterState {
    pub name: String,
    pub level: i32,
    pub origins: Vec<OriginRow>,
    pub traits: Vec<TraitRow>,           // all traits for the selected origin
    pub selected_origin_idx: usize,      // index into origins
    pub is_ghoul: bool,
    pub selected_traits: Vec<bool>,      // one bool per non-ghoul trait for multi-pick
    // Cache: group origins by sourcebook for the combo
    pub origin_labels: Vec<String>,      // display strings for combo (with separators)
    pub origin_label_to_idx: Vec<Option<usize>>, // maps combo item → origins index (None = header)
    pub has_gifted_trait: bool,
    pub mutant_type: MutantType,
}

impl NewCharacterState {
    pub fn has_gifted_trait(&self) -> bool {
        self.traits.iter().enumerate().any(|(i, t)| {
            t.name.eq_ignore_ascii_case("Gifted")
                && self.selected_traits.get(i).copied().unwrap_or(false)
        })
    }
    pub fn mutant_type(&self) -> MutantType {
        if self.origins.is_empty() { return MutantType::None; }
        let origin = &self.origins[self.selected_origin_idx];
        match origin.name.as_str() {
            "Super Mutant" => MutantType::StandardSuperMutant,
            "Nightkin"     => MutantType::Nightkin,
            _              => MutantType::None,
        }
    }
    pub fn load(db: &Db) -> Self {
        let origins = load_origins(db);
        let (labels, label_map) = build_origin_labels(&origins);

        let mut state = Self {
            name: String::new(),
            level: 1,
            origins,
            traits: vec![],
            selected_origin_idx: 0,
            is_ghoul: false,
            selected_traits: vec![],
            origin_labels: labels,
            origin_label_to_idx: label_map,
            has_gifted_trait: false,
            mutant_type: MutantType::None,
        };
        state.refresh_traits(db);
        state
    }
    pub fn selected_origin_id(&self) -> Option<i64> {
        self.origins.get(self.selected_origin_idx).map(|o| o.id as i64)
    }
    fn refresh_traits(&mut self, db: &Db) {
        if self.origins.is_empty() {
            self.traits = vec![];
            self.selected_traits = vec![];
            return;
        }
        let origin_id = self.origins[self.selected_origin_idx].id;
        let new_traits = load_traits(db, origin_id);
        //eprintln!("origin: {} canghoul: {}", self.origins[self.selected_origin_idx].name, self.origins[self.selected_origin_idx].can_ghoul);
        //eprintln!("first trait: {} ghoul: {}", new_traits[0].name, new_traits[0].is_ghoul_trait);
        if new_traits[0].is_ghoul_trait {
            //eprintln!("ghoul origin");
            self.is_ghoul = true;
            //eprintln!("is_ghoul set");
        } else if !self.origins[self.selected_origin_idx].can_ghoul {
            //eprintln!("can't be ghoul");
            self.is_ghoul = false;
            //eprintln!("is_ghoul unset");
        }
        if self.is_ghoul {
            //eprintln!("loading ghoul trait");
            self.traits = load_ghoul_trait(db);
            //eprintln!("ghoultrait: {}", self.traits[0].name);
        } else {
            //eprintln!("loading origin trait(s)");
            self.traits = load_traits(db, origin_id);
        }
        /*
        for t in &self.traits {
            eprintln!("trait: {} ghoul={}", t.name, t.is_ghoul_trait);
        }
        */
        self.selected_traits = vec![false; self.traits.iter().count()];

        if self.traits.len() == 1 && !self.traits[0].is_ghoul_trait {
            self.selected_traits[0] = true;
        }
        //let non_ghoul_count = self.traits.iter().filter(|t| !t.is_ghoul_trait).count();
        //self.selected_traits = vec![false; non_ghoul_count];
    }
}

pub fn sanitize(s: &str) -> String {
    s.replace('\u{2019}', "'")
}

pub fn render_text_wrapped(disabled: bool, colored: bool, ui: &Ui, text: &str, indent_x: f32, wrap_pos: f32) {
    let cleaned = sanitize(text);
    let lines: Vec<&str> = cleaned.split("\\n").collect();

    let desc_color = ui.style_color(imgui::StyleColor::DragDropTarget);
    let dis_color = ui.style_color(imgui::StyleColor::TextDisabled);

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() { ui.spacing(); continue; }

        if i > 0 {
            let y = ui.cursor_pos()[1];
            ui.set_cursor_pos([indent_x, y]);
        }

        let _wrap = ui.push_text_wrap_pos_with_pos(wrap_pos);

        if disabled {
            let _c = ui.push_style_color(imgui::StyleColor::Text, dis_color);
            ui.text_wrapped(trimmed);
        } else if colored {
            let _c = ui.push_style_color(imgui::StyleColor::Text, desc_color);
            ui.text_wrapped(trimmed);
        } else {
            ui.text_wrapped(trimmed);
        }
    }
}

// ── DB queries ────────────────────────────────────────────────────────────────

fn load_origins(db: &Db) -> Vec<OriginRow> {
    let result = db.block_on(async {
        sqlx::query!(
            r#"
            SELECT o.id, o.name, o.description, o.can_ghoul,
                s.name AS sourcebook
            FROM origins o
            JOIN sourcebooks s ON s.id = o.sourcebook_id
            ORDER BY s.id, o.name
            "#
        )
        .fetch_all(&db.pool)
        .await
    });

    match result {
        Ok(rows) => rows.into_iter().map(|r| OriginRow {
            id: r.id,
            name: r.name.unwrap_or_default(),
            sourcebook: r.sourcebook.unwrap_or_default(),
            description: r.description.unwrap_or_default(),
            can_ghoul: r.can_ghoul.unwrap_or(0) != 0,
        }).collect(),
        Err(e) => { eprintln!("Failed to load origins: {e}"); vec![] }
    }
}

fn load_traits(db: &Db, origin_id: i64) -> Vec<TraitRow> {
    let result = db.block_on(async {
        sqlx::query!(
            r#"
            SELECT t.id, ot.origin_id, t.name, t.description,
                ot.is_ghoul_trait
            FROM origin_traits ot
            JOIN traits t ON t.id = ot.trait_id
            WHERE ot.origin_id = ?
            ORDER BY ot.is_ghoul_trait, t.name
            "#,
            origin_id
        )
        .fetch_all(&db.pool)
        .await
    });

    match result {
        Ok(rows) => rows.into_iter().map(|r| TraitRow {
            id: r.id,
            origin_id: r.origin_id.unwrap_or_default(),
            name: r.name.unwrap_or_default(),
            description: r.description.unwrap_or_default(),
            is_ghoul_trait: r.is_ghoul_trait.unwrap_or(0) != 0,
        }).collect(),
        Err(e) => { eprintln!("Failed to load traits: {e}"); vec![] }
    }
}

fn load_ghoul_trait(db: &Db) -> Vec<TraitRow> {
    let result = db.block_on(async {
        sqlx::query!(
            r#"
            SELECT t.id, ot.origin_id, t.name, t.description,
                ot.is_ghoul_trait
            FROM origin_traits ot
            JOIN traits t ON t.id = ot.trait_id
            WHERE ot.is_ghoul_trait = 1
            ORDER BY ot.is_ghoul_trait, t.name
            "#
        )
        .fetch_all(&db.pool)
        .await
    });

    match result {
        Ok(rows) => rows.into_iter().map(|r| TraitRow {
            id: r.id,
            origin_id: r.origin_id.unwrap_or_default(),
            name: r.name.unwrap_or_default(),
            description: r.description.unwrap_or_default(),
            is_ghoul_trait: r.is_ghoul_trait.unwrap_or(0) != 0,
        }).collect(),
        Err(e) => { eprintln!("Failed to load ghoul trait: {e}"); vec![] }
    }
}

// ── Origin label builder (adds sourcebook headers) ────────────────────────────

fn build_origin_labels(origins: &[OriginRow]) -> (Vec<String>, Vec<Option<usize>>) {
    let mut labels: Vec<String> = vec![];
    let mut label_map: Vec<Option<usize>> = vec![];
    let mut current_book = String::new();

    for (i, origin) in origins.iter().enumerate() {
        if origin.sourcebook != current_book {
            current_book = origin.sourcebook.clone();
            labels.push(format!("-- {} --", current_book));
            label_map.push(None); // header — not selectable
        }
        labels.push(format!("  {}", origin.name));
        label_map.push(Some(i));
    }

    (labels, label_map)
}

fn can_advance(state: &NewCharacterState) -> (bool, &'static str) {
    // Must have an origin selected (origins list is non-empty and index is valid)
    if state.origins.is_empty() {
        return (false, "No origins available.");
    }

    //let origin = &state.origins[state.selected_origin_idx];

    // Count normal (non-ghoul) traits
    let normal_traits: Vec<&TraitRow> = state.traits.iter()
        .filter(|t| !t.is_ghoul_trait)
        .collect();

    let selected_count = state.selected_traits.iter()
        .filter(|&&v| v)
        .count();

    if normal_traits.len() > 1 {
        // Multiple choice — must pick exactly 2
        if selected_count < 2 {
            return (false, "Select 2 traits to continue.");
        }
    } else if normal_traits.len() == 1 {
        // Single trait — auto-selected, no action needed
    }

    (true, "")
}

// ── Render ────────────────────────────────────────────────────────────────────

pub fn render_new_character(
    ui: &Ui,
    window: &Window,
    state: &mut NewCharacterState,
    screen: &mut AppScreen,
    db: &Db,
) {
    let (win_w, win_h) = window.size();
    let content_h = win_h as f32 - BAR_HEIGHT;
    let w = (win_w as f32 * 0.65).min(960.0);
    let h = win_h as f32 * 0.85;

    ui.window("##new_character")
        .title_bar(false)
        .resizable(false)
        .movable(false)
        .size([w, h], imgui::Condition::Always)
        .position(
            [(win_w as f32 - w) * 0.5, BAR_HEIGHT + (content_h - h) * 0.5],
            imgui::Condition::Always,
        )
        .build(|| {
            // ── Header ────────────────────────────────────────────────────────
            ui.text("NEW CHARACTER");
            ui.separator();
            ui.spacing();

            let label_w = 140.0_f32;
            let field_w = w - label_w - 32.0;

            // ── Character Name ────────────────────────────────────────────────
            ui.text("Character Name");
            ui.same_line_with_pos(label_w);
            ui.set_next_item_width(field_w);
            ui.input_text("##char_name", &mut state.name).build();

            ui.spacing();

            // ── Character Level ───────────────────────────────────────────────
            ui.text("Character Level");
            ui.same_line_with_pos(label_w);
            if ui.button("-##level_dec") {
                if state.level > 1 { state.level -= 1; }
            }
            ui.same_line();
            ui.text(format!("{}", state.level));
            ui.same_line();
            if ui.button("+##level_inc") {
                state.level += 1;
            }
            if state.level < 1 { state.level = 1; }

            ui.spacing();
            ui.separator();
            ui.spacing();

            // ── Origin dropdown ───────────────────────────────────────────────
            ui.text("Origin");
            ui.same_line_with_pos(label_w);
            ui.set_next_item_width(field_w);

            // Find current combo index (the label entry for selected_origin_idx)
            let current_combo_idx = state.origin_label_to_idx
                .iter()
                .position(|m| *m == Some(state.selected_origin_idx))
                .unwrap_or(0);

            let current_label = state.origin_labels
                .get(current_combo_idx)
                .map(|s| s.trim())
                .unwrap_or("-")
                .to_string();

            let mut origin_changed = false;
            if let Some(_cb) = ui.begin_combo("##origin", &current_label) {
                for (combo_idx, label) in state.origin_labels.iter().enumerate() {
                    match state.origin_label_to_idx[combo_idx] {
                        None => {
                            // Sourcebook header — dimmed, not selectable
                            ui.text_disabled(label);
                        }
                        Some(origin_idx) => {
                            let selected = origin_idx == state.selected_origin_idx;
                            if ui.selectable_config(label.trim()).selected(selected).build() {
                                if origin_idx != state.selected_origin_idx {
                                    state.selected_origin_idx = origin_idx;
                                    origin_changed = true;
                                }
                            }
                            if selected {
                                ui.set_item_default_focus();
                            }
                        }
                    }
                }
            }

            if origin_changed {
                //eprintln!("origin changed, refreshing");
                state.refresh_traits(db);
            }

            ui.spacing();

            /*
            let can_be_ghoul = state.origins
                .get(state.selected_origin_idx)
                .map(|o| o.can_ghoul)
                .unwrap_or(false);


            let origin_description = state.origins
                .get(state.selected_origin_idx)
                .map(|o| o.description.clone())
                .unwrap_or_default();


            let origin_name = state.origins
                .get(state.selected_origin_idx)
                .map(|o| o.description.clone())
                .unwrap_or_default();
             */

            // ── Origin Description ────────────────────────────────────────────
            if let Some(origin) = state.origins.get(state.selected_origin_idx) {
                ui.text("Description");
                ui.same_line_with_pos(label_w);
                {
                    //ui.push_text_wrap_pos_with_pos(label_w + field_w);
                    //ui.text_wrapped(&origin.description);
                    render_text_wrapped(false, true, ui, &origin.description, label_w, label_w + field_w);
                }

                ui.spacing();

                // ── Ghoul checkbox ────────────────────────────────────────────
                let mut ghoul_changed = false;
                
                if origin.can_ghoul {
                    ui.text("Ghoul?");
                    ui.same_line_with_pos(label_w);
                    let mut ghoul = state.is_ghoul;
                    if ui.checkbox("##is_ghoul", &mut ghoul) {
                        if ghoul != state.is_ghoul {
                            ghoul_changed = true;
                        }
                        state.is_ghoul = ghoul;
                    }
                    ui.spacing();
                }

                // ── Traits ────────────────────────────────────────────────────
                ui.separator();
                ui.spacing();
                ui.text("Trait");



                /*
                if ghoul_changed {
                    //eprintln!("is_ghoul changed check");
                    state.refresh_traits(db);
                    if state.is_ghoul  {
                        // Show ghoul traits
                        /*
                        let ghoul_traits: Vec<&TraitRow> = state.traits.iter()
                            .filter(|t| t.is_ghoul_trait)
                            .collect();
                        eprintln!("ghoul_traits: {}", ghoul_traits.len());
                        if ghoul_traits.is_empty() {
                            ui.same_line_with_pos(label_w);
                            ui.text_disabled("(no ghoul traits defined)");
                        } else {

                        */
                        //for t in &ghoul_traits {
                        eprintln!("state is_ghoul");

                        let t = &state.traits[0];
                        ui.same_line_with_pos(label_w);
                        /*ui.push_text_wrap_pos_with_pos(label_w + field_w);
                        ui.text_colored([0.55, 0.85, 0.55, 1.0], &t.name);
                        {
                            ui.push_text_wrap_pos_with_pos(label_w + field_w);
                            ui.text_wrapped(&t.description);
                        }

                        ui.spacing();
                        */
                        ui.text(&t.name);
                        ui.new_line();
                        let y = ui.cursor_pos()[1];
                        ui.set_cursor_pos([label_w, y]);
                        {
                            //ui.begin_disabled(true);
                            //let _wrap = ui.push_text_wrap_pos_with_pos(label_w + field_w);
                            //ui.text_wrapped(&t.description);   
                            render_text_wrapped(false, true, ui, &t.description, label_w, label_w + field_w);
                        }
                        ui.spacing();

                            //}
                        //}
                    }
                } else {
                    /*
                    let normal_traits: Vec<(usize, &TraitRow)> = state.traits.iter()
                        .filter(|t| !t.is_ghoul_trait)
                        .enumerate()
                        .collect();
                    */

                    let normal_traits: Vec<(usize, &TraitRow)> = state.traits.iter()
                        .enumerate()
                        .collect();

                    if normal_traits.is_empty() {
                        ui.same_line_with_pos(label_w);
                        ui.text_disabled("(no traits defined)");
                    } else if normal_traits.len() == 1 {
                        // Single trait — just show it
                        let (_, t) = normal_traits[0];
                        ui.same_line_with_pos(label_w);
                        //ui.push_text_wrap_pos_with_pos(label_w + field_w);
                        ui.text(&t.name);
                        ui.new_line();
                        let y = ui.cursor_pos()[1];
                        ui.set_cursor_pos([label_w, y]);
                        {
                            //ui.begin_disabled(true);
                            //ui.push_text_wrap_pos_with_pos(label_w + field_w);
                            //ui.text_wrapped(&t.description);
                            render_text_wrapped(false, true, ui, &t.description, label_w, label_w + field_w);
                        }
                    } else {
                        // Multiple traits — show checkboxes
                        let selected_count = state.selected_traits.iter().filter(|&&v| v).count();
                        //ui.new_line();
                        let y = ui.cursor_pos()[1];
                        ui.set_cursor_pos([label_w, y]);
                        ui.text_disabled("Choose up to 2:");
                        ui.spacing();
                        for (ti, t) in &normal_traits {
                            let currently_checked = state.selected_traits.get(*ti).copied().unwrap_or(false);
                            let at_limit = !currently_checked && selected_count >= 2;
                            let mut checked = currently_checked;

                            let y = ui.cursor_pos()[1];
                            ui.set_cursor_pos([label_w, y]);

                            let cb_label = format!("##trait_{}", ti);
                            if at_limit {
                                //ui.begin_disabled(true);
                                //ui.checkbox(&format!("{}##trait_{}", t.name, ti), &mut checked);
                                ui.checkbox(&cb_label, &mut checked);
                                //ui.end_disabled();
                            } else {
                                if ui.checkbox(&cb_label, &mut checked) {
                                    if let Some(v) = state.selected_traits.get_mut(*ti) {
                                        *v = checked;
                                    }
                                }
                            }
                            ui.same_line_with_pos(label_w + 24.0);
                            if at_limit {
                                render_text_wrapped(true, false, ui, &t.name, label_w + 24.0, label_w + field_w + 24.0);
                            } else {
                                render_text_wrapped(false, true, ui, &t.name, label_w + 24.0, label_w + field_w + 24.0);
                            }
                            
                            //ui.push_text_wrap_pos_with_pos(label_w + field_w);
                            let y = ui.cursor_pos()[1];
                            {
                                //ui.begin_disabled(true);
                                ui.set_cursor_pos([label_w + 24.0, y]);
                                //ui.text_wrapped(&t.description);
                                if at_limit {
                                    render_text_wrapped(true, false, ui, &t.description, label_w, label_w + field_w);
                                } else {
                                    render_text_wrapped(false, true, ui, &t.description, label_w, label_w + field_w);
                                }
                            }
                            
                            ui.spacing();
                        }
                    }
                }
            }

            */

                // Refresh traits if ghoul state just changed
                if ghoul_changed {
                    state.refresh_traits(db);
                }

                let traits_snapshot: Vec<TraitRow> = state.traits.clone();

                if traits_snapshot.is_empty() {
                    ui.same_line_with_pos(label_w);
                    ui.text_disabled("(no traits defined)");
                } else if state.is_ghoul {
                    // Ghoul trait — always a single fixed trait, just display it
                    let t = &traits_snapshot[0];
                    ui.same_line_with_pos(label_w);
                    ui.text(&t.name);
                    ui.new_line();
                    let y = ui.cursor_pos()[1];
                    ui.set_cursor_pos([label_w, y]);
                    render_text_wrapped(false, true, ui, &t.description, label_w, label_w + field_w);
                    ui.spacing();
                } else if traits_snapshot.len() == 1 {
                    // Single normal trait — just display it
                    let t = &traits_snapshot[0];
                    ui.same_line_with_pos(label_w);
                    ui.text(&t.name);
                    ui.new_line();
                    let y = ui.cursor_pos()[1];
                    ui.set_cursor_pos([label_w, y]);
                    render_text_wrapped(false, true, ui, &t.description, label_w, label_w + field_w);
                    ui.spacing();
                } else {
                    // Multiple traits — checkboxes, max 2
                    let selected_count = state.selected_traits.iter().filter(|&&v| v).count();
                    let y = ui.cursor_pos()[1];
                    ui.set_cursor_pos([label_w, y]);
                    ui.text_disabled("Choose up to 2:");
                    ui.spacing();

                    for (ti, t) in traits_snapshot.iter().enumerate() {
                        let currently_checked = state.selected_traits.get(ti).copied().unwrap_or(false);
                        let at_limit = !currently_checked && selected_count >= 2;
                        let mut checked = currently_checked;

                        let y = ui.cursor_pos()[1];
                        ui.set_cursor_pos([label_w, y]);

                        if at_limit {
                            let _lim_guard = at_limit.then(|| ui.begin_disabled(true));
                            ui.checkbox(&format!("##trait_{}", ti), &mut checked);
                        } else {
                            if ui.checkbox(&format!("##trait_{}", ti), &mut checked) {
                                if let Some(v) = state.selected_traits.get_mut(ti) {
                                    *v = checked;
                                }
                            }
                        }

                        ui.same_line_with_pos(label_w + 24.0);
                        if at_limit {
                            ui.text_disabled(&t.name);
                        } else {
                            ui.text(&t.name);
                        }

                        let y = ui.cursor_pos()[1];
                        ui.set_cursor_pos([label_w + 24.0, y]);
                        render_text_wrapped(at_limit, !at_limit, ui, &t.description, label_w + 24.0, label_w + field_w);

                        ui.spacing();
                    }
                }

                // ── Footer buttons ────────────────────────────────────────────────
                let footer_y = h - 48.0;
                ui.set_cursor_pos([16.0, footer_y]);
                if ui.button("< Back") {
                    *screen = AppScreen::MainMenu;
                }
                ui.same_line();

                let (can_next, reason) = can_advance(state);
                let _next_guard = (!can_next).then(|| ui.begin_disabled(true));
                if ui.button("Next >") {
                    *screen = AppScreen::Special;
                }
                if !reason.is_empty() {
                    render_text_wrapped(true, false, ui, reason, 16.0, 256.0);
                }
            }
        });
}