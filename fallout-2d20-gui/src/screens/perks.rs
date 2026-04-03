use imgui::Ui;
use sdl2::video::Window;
use serde::{Deserialize, Serialize};
use serde_json;
use crate::{AppScreen, BAR_HEIGHT};
use crate::db::Db;
use crate::screens::new_character::render_text_wrapped;
use crate::screens::special::SpecialState;
use crate::screens::skills::SKILLS;

// ── Types ─────────────────────────────────────────────────────────────────────

const SPECIAL_LABELS: [&str; 7] = [
    "Strength", "Perception", "Endurance",
    "Charisma", "Intelligence", "Agility", "Luck",
];

#[derive(Debug, Clone)]
pub struct PerkRow {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub level_req: i64,
    pub ranks: i64,
    pub rank_range: i64,   // additional levels required per rank
    pub reqs: Vec<String>, // e.g. "strength: 6", "book"
    pub limits: Vec<String>, // e.g. "no ghoul", "no robot"
    pub sourcebook: String,
}

#[derive(Debug, Clone)]
pub struct CharPerk {
    pub perk_id: i64,
    pub ranks: i64,
}

// ── State ─────────────────────────────────────────────────────────────────────

pub struct PerksState {
    pub all_perks: Vec<PerkRow>,
    pub char_perks: Vec<CharPerk>,  // taken perks

    // Character context
    pub level: i64,
    pub special: [i64; 7],         // display values (post-modifier)
    pub is_ghoul: bool,
    pub is_robot: bool,
    pub is_super_mutant: bool,
    pub has_companion: bool,
    pub perk_trait: bool, // trait 10 = +1 perk slot

    // Filter state
    pub show_eligible_only: bool,
    pub special_filters: [bool; 8], // X, S, P, E, C, I, A, L

    pub pending_resolution: Option<i64>, // set to perk_id when Take is clicked
}

impl PerksState {
    pub fn new(
        all_perks: Vec<PerkRow>,
        level: i64,
        special: [i64; 7],
        is_ghoul: bool,
        is_robot: bool,
        is_super_mutant: bool,
        has_companion: bool,
        perk_trait: bool,
    ) -> Self {
        Self {
            all_perks,
            char_perks: vec![],
            level,
            special: special,
            is_ghoul,
            is_robot,
            is_super_mutant,
            has_companion,
            perk_trait,
            show_eligible_only: false,
            special_filters: [true; 8],
            pending_resolution: None,
        }
    }

    pub fn max_perks(&self) -> i64 {
        self.level + if self.perk_trait { 1 } else { 0 }
    }

    pub fn perks_taken(&self) -> i64 {
        //self.char_perks.len() as i64
        self.char_perks.iter().map(|p| p.ranks).sum()
    }

    pub fn perks_remaining(&self) -> i64 {
        self.max_perks() - self.perks_taken()
    }

    pub fn get_ranks(&self, perk_id: i64) -> i64 {
        self.char_perks.iter()
            .find(|p| p.perk_id == perk_id)
            .map(|p| p.ranks)
            .unwrap_or(0)
            .into()
    }

    pub fn has_perk(&self, perk_id: i64) -> bool {
        self.get_ranks(perk_id) > 0
    }

    pub fn is_eligible(&self, perk: &PerkRow) -> bool {
        let ranks_taken = self.get_ranks(perk.id);

        // Already at max ranks
        if ranks_taken >= perk.ranks { return false; }

        // Level req for next rank
        let next_rank_level = perk.level_req + ranks_taken * perk.rank_range;
        if self.level < next_rank_level { return false; }

        // SPECIAL requirements
        for req in &perk.reqs {
            if req.contains(':') {
                let parts: Vec<&str> = req.splitn(2, ':').collect();
                if parts.len() != 2 { continue; }
                let stat = parts[0].trim().to_lowercase();
                let val: i64 = parts[1].trim().parse().unwrap_or(0);
                let char_val = self.special_value_for(&stat);
                if char_val < val { return false; }
            }
            if req.trim().to_lowercase() == "book" {
                // Book perks not acquirable during creation
                return false;
            }
        }

        // Limits
        for limit in &perk.limits {
            let lower = limit.to_lowercase();
            if lower.contains("daring nature") && self.has_perk(25) { return false; }
            if lower.contains("cautious nature") && self.has_perk(18) { return false; }
            if lower.contains("robot") && self.is_robot { return false; }
            if lower.contains("ghoul") && self.is_ghoul { return false; }
            if lower.contains("rads") && (self.is_robot || self.is_ghoul || self.is_super_mutant) {
                return false;
            }
            if lower.contains("companion") && self.has_companion { return false; }
        }

        true
    }

    fn special_value_for(&self, stat: &str) -> i64 {
        match stat {
            "strength"     => self.special[0],
            "perception"   => self.special[1],
            "endurance"    => self.special[2],
            "charisma"     => self.special[3],
            "intelligence" => self.special[4],
            "agility"      => self.special[5],
            "luck"         => self.special[6],
            _ => 0,
        }
    }

    /// Which SPECIAL filter bucket does this perk belong to?
    /// Returns index into special_filters: 0=X, 1=S, 2=P, 3=E, 4=C, 5=I, 6=A, 7=L
    pub fn perk_filter_indices(perk: &PerkRow) -> Vec<usize> {
        let stat_reqs: Vec<&str> = perk.reqs.iter()
            .filter(|r| r.contains(':'))
            .map(|r| r.splitn(2, ':').next().unwrap_or("").trim())
            .collect();

        if stat_reqs.is_empty() {
            return vec![0]; // X = no stat req
        }

        stat_reqs.iter().map(|s| match s.to_lowercase().as_str() {
            "strength"     => 1,
            "perception"   => 2,
            "endurance"    => 3,
            "charisma"     => 4,
            "intelligence" => 5,
            "agility"      => 6,
            "luck"         => 7,
            _              => 0,
        }).collect()
    }

    pub fn perk_passes_filter(&self, perk: &PerkRow) -> bool {
        Self::perk_filter_indices(perk)
            .iter()
            .any(|&i| self.special_filters[i])
    }

    pub fn add_perk(&mut self, perk_id: i64) {
        if let Some(cp) = self.char_perks.iter_mut().find(|p| p.perk_id == perk_id) {
            cp.ranks += 1;
        } else {
            self.char_perks.push(CharPerk { perk_id, ranks: 1 });
        }
    }

    pub fn remove_perk(&mut self, perk_id: i64) {
        if let Some(pos) = self.char_perks.iter().position(|p| p.perk_id == perk_id) {
            if self.char_perks[pos].ranks > 1 {
                self.char_perks[pos].ranks -= 1;
            } else {
                self.char_perks.remove(pos);
            }
        }
    }
    pub fn begin_resolve(&self, perk_id: i64, perk_name: &str) -> Option<PerkResolutionPopup> {
        let resolution = match perk_id {
            PERK_INTENSE_TRAINING => Some(PerkResolution::IntenseTraining { selected_stat: None }),
            PERK_SKILLED => Some(PerkResolution::Skilled {
                mode: SkilledMode::TwoToOne,
                skill_a: None,
                skill_b: None,
            }),
            PERK_TAG => Some(PerkResolution::Tag { selected_skill: None }),
            _ => None,
        };
        resolution.map(|r| PerkResolutionPopup {
            perk_id,
            perk_name: perk_name.to_string(),
            resolution: r,
            open: true,
        })
    }

    pub fn is_resolution_complete(popup: &PerkResolutionPopup) -> bool {
        match &popup.resolution {
            PerkResolution::IntenseTraining { selected_stat } => selected_stat.is_some(),
            PerkResolution::Skilled { mode, skill_a, skill_b } => {
                match mode {
                    SkilledMode::TwoToOne => skill_a.is_some(),
                    SkilledMode::OneToTwo => skill_a.is_some() && skill_b.is_some()
                        && skill_a != skill_b,
                }
            }
            PerkResolution::Tag { selected_skill } => selected_skill.is_some(),
        }
    }
}

pub fn load_perks(db: &Db) -> Vec<PerkRow> {
    let result = db.block_on(async {
        sqlx::query!(
            r#"
            SELECT p.id, p.name, p.description, p.ranks, p.rank_range,
                p.level_req, p.reqs, p.limits, s.name AS sourcebook
            FROM perks p
            JOIN sourcebooks s ON s.id = p.sourcebook_id
            ORDER BY s.id, p.name
            "#
        )
        .fetch_all(&db.pool)
        .await
    });

    match result {
        Ok(rows) => rows.into_iter().map(|r| {
            let reqs: Vec<String> = r.reqs
                .as_deref()
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_default();
            let limits: Vec<String> = r.limits
                .as_deref()
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_default();
            PerkRow {
                id: r.id,
                name: r.name.unwrap_or_default(),
                sourcebook: r.sourcebook.unwrap_or_default(),
                description: r.description.unwrap_or_default(),
                level_req: r.level_req.unwrap_or_default(),
                ranks: r.ranks.unwrap_or_default(),
                rank_range: r.rank_range.unwrap_or_default(),
                limits,
                reqs,
            }
        }).collect(),
        Err(e) => { eprintln!("Failed to load perks: {e}"); vec![] }
    }
}

// ── Perk Resolution ───────────────────────────────────────────────────────────

const PERK_INTENSE_TRAINING: i64 = 45;
const PERK_SKILLED: i64 = 83;
const PERK_TAG: i64 = 92;

#[derive(Debug, Clone, PartialEq)]
pub enum SkilledMode {
    TwoToOne,
    OneToTwo,
}

#[derive(Debug, Clone)]
pub enum PerkResolution {
    IntenseTraining {
        selected_stat: Option<usize>, // index into SPECIAL (0-6)
    },
    Skilled {
        mode: SkilledMode,
        skill_a: Option<usize>,       // first skill index
        skill_b: Option<usize>,       // second skill (OneToTwo only)
    },
    Tag {
        selected_skill: Option<usize>,
    },
}

pub struct PerkResolutionPopup {
    pub perk_id: i64,
    pub perk_name: String,
    pub resolution: PerkResolution,
    pub open: bool,
}

// ── Render ────────────────────────────────────────────────────────────────────

const FILTER_LABELS: [&str; 8] = ["X", "S", "P", "E", "C", "I", "A", "L"];

pub fn render_perks(
    ui: &Ui,
    window: &Window,
    state: &mut PerksState,
    screen: &mut AppScreen,
) {
    let (win_w, win_h) = window.size();
    let content_h = win_h as f32 - BAR_HEIGHT;
    let w = (win_w as f32 * 0.65).min(860.0);
    let h = win_h as f32 * 0.85;

    let Some(_tok) = ui.window("##perks")
        .title_bar(false)
        .resizable(false)
        .movable(false)
        .size([w, h], imgui::Condition::Always)
        .position(
            [(win_w as f32 - w) * 0.5, BAR_HEIGHT + (content_h - h) * 0.5],
            imgui::Condition::Always,
        )
        .begin()
    else { return; };

    // ── Header ────────────────────────────────────────────────────
    ui.text("PERKS");
    ui.separator();
    ui.spacing();

    let remaining = state.perks_remaining();
    let taken = state.perks_taken();
    let max = state.max_perks();

    if remaining == 0 {
        render_text_wrapped(false, true, ui,
            &format!("Perks: {}/{}", taken, max), 0.0, w);
    } else {
        ui.text(format!("Perks: {}/{} ({} remaining)", taken, max, remaining));
    }

    ui.spacing();

    // ── Filters ───────────────────────────────────────────────────
    ui.checkbox("Show eligible only##eo", &mut state.show_eligible_only);
    ui.same_line();
    ui.text_disabled("|");
    ui.same_line();
    ui.text_disabled("Filter by SPECIAL:");
    ui.same_line();

    for i in 0..8 {
        if ui.checkbox(
            &format!("{}##sf_{}", FILTER_LABELS[i], i),
            &mut state.special_filters[i]
        ) {}
        if i < 7 { ui.same_line(); }
    }

    ui.spacing();
    ui.separator();
    ui.spacing();

    // ── Perk list (scrollable child region) ───────────────────────
    let list_h = h - 140.0; // leave room for header + footer
    let Some(_child) = ui.child_window("##perk_scroll")
        .size([w - 16.0, list_h])
        .begin()
    else { return; };

    let col_name  = 0.0_f32;
    let col_reqs  = 260.0_f32;
    let col_ranks = 460.0_f32;
    let col_btns  = 540.0_f32;

    // Collect filtered perks (avoid borrow issues)
    let filtered: Vec<usize> = (0..state.all_perks.len())
        .filter(|&i| {
            let perk = &state.all_perks[i];
            state.perk_passes_filter(perk)
                && (!state.show_eligible_only || state.is_eligible(perk))
        })
        .collect();

    //let (labels, label_map) = build_perk_labels(&state.all_perks);
    let mut current_label = String::new();

    for &pi in &filtered {
        // Re-borrow inside loop to avoid holding state borrow
        let perk_id   = state.all_perks[pi].id;
        let perk_name = state.all_perks[pi].name.clone();
        let perk_desc = state.all_perks[pi].description.clone();
        let perk_lvl  = state.all_perks[pi].level_req;
        let perk_max  = state.all_perks[pi].ranks;
        let perk_reqs = state.all_perks[pi].reqs.clone();
        let perk_lims = state.all_perks[pi].limits.clone();
        let perk_rank_rng = state.all_perks[pi].rank_range;

        let ranks_taken = state.get_ranks(perk_id);
        let eligible    = state.is_eligible(&state.all_perks[pi].clone());
        let at_cap      = ranks_taken >= perk_max;
        let no_slots    = state.perks_remaining() <= 0;

        let perk_src = state.all_perks[pi].sourcebook.clone();
        if current_label != perk_src {
            ui.text_disabled(format!(" ----- {} -----", perk_src));
            ui.separator();
            current_label = perk_src;
        }
        /*
        let perk_label_idx = label_map
            .iter()
            .position(|m| *m == Some(perk_id.into()))
            .unwrap_or(0);
        let perk_label = labels
            .get(perk_label_idx)
            .map(|s| s.trim())
            .unwrap_or("-");
            //.to_string();
        //let perk_label_copy = perk_label.clone();
        if perk_label != current_label {
            ui.text_disabled(perk_label);
            ui.separator();
            current_label = perk_label.to_string();
            //current_label = perk_label_copy;
        }
        */

        // Row background tint based on status
        let cursor = ui.cursor_pos();
        let draw_list = ui.get_window_draw_list();
        let win_pos = ui.window_pos();
        let abs_x = win_pos[0] + cursor[0];
        let abs_y = win_pos[1] + cursor[1] - ui.scroll_y();
        let row_h = 80.0_f32;

        //need to think how i want to handle this wrt themes
        let tint = if at_cap {
            [0.15, 0.35, 0.15, 0.3_f32]  // fully taken — green tint
        } else if eligible && ranks_taken > 0 {
            [0.20, 0.40, 0.50, 0.3_f32]  // rank available — blue tint
        } else if eligible {
            [0.10, 0.25, 0.10, 0.2_f32]  // available — subtle green
        } else {
            [0.0, 0.0, 0.0, 0.0_f32]     // unavailable — no tint
        };

        let button_height = ui.clone_style().frame_padding[1] * 2.0 + ui.text_line_height();

        let rect_fill = imgui::ImColor32::from_rgba_f32s(tint[0], tint[1], tint[2], tint[3]);

        if tint[3] > 0.0 {
            draw_list.add_rect_filled_multicolor(
                [abs_x - 4.0, abs_y - 4.0],
                [abs_x + w - 24.0, abs_y + button_height + 4.0],
                rect_fill, rect_fill, rect_fill, rect_fill
            );
        }

        // Perk name + rank pips
        //let pips = "★".repeat(ranks_taken as usize) + &"☆".repeat((perk_max - ranks_taken) as usize);
        let pips = "*".repeat(ranks_taken as usize) + &"¤".repeat((perk_max - ranks_taken) as usize);
        if at_cap {
            render_text_wrapped(false, true, ui,
                &format!("{} {}", perk_name, pips), col_name, col_reqs);
        } else if eligible {
            ui.text(format!("{} {}", perk_name, pips));
        } else {
            render_text_wrapped(true, false, ui,
                &format!("{} {}", perk_name, pips), col_name, col_reqs);
        }

        let mut lvl_string = String::new();
        for i in 0..perk_max {
            if i == 0 { continue } else {
                let next_lvl = format!("/{}", perk_lvl + (i * perk_rank_rng));
                lvl_string.push_str(&next_lvl);
            }
        }

        // Level req
        ui.same_line_with_pos(col_reqs);
        ui.text_disabled(format!("lv {}{}", perk_lvl, lvl_string));

        // Ranks display
        ui.same_line_with_pos(col_ranks);
        ui.text_disabled(format!("{}/{}", ranks_taken, perk_max));

        // Buttons
        ui.same_line_with_pos(col_btns);

        if ranks_taken == 0 {
            // Take / (disabled Drop)
            let _g = (!eligible || no_slots).then(|| ui.begin_disabled(true));
            if ui.button(format!("Take##take_{}", perk_id)) {
                state.add_perk(perk_id);
                state.pending_resolution = Some(perk_id);
            }
            drop(_g);
            ui.same_line();
            let _g2 = true.then(|| ui.begin_disabled(true));
            ui.button(format!("Drop##drop_{}", perk_id));
            drop(_g2);
        } else if at_cap {
            // (disabled Add Rank) / Drop Rank
            let _g = true.then(|| ui.begin_disabled(true));
            ui.button(format!("Rank+##rankp_{}", perk_id));
            drop(_g);
            ui.same_line();
            if ui.button(format!("Drop##drop_{}", perk_id)) {
                state.remove_perk(perk_id);
            }
        } else {
            // Add Rank / Drop Rank
            let _g = (!eligible || no_slots).then(|| ui.begin_disabled(true));
            if ui.button(format!("Rank+##rankp_{}", perk_id)) {
                state.add_perk(perk_id);
                state.pending_resolution = Some(perk_id);
            }
            drop(_g);
            ui.same_line();
            if ui.button(format!("Drop##drop_{}", perk_id)) {
                state.remove_perk(perk_id);
            }
        }

        // Description (dimmed)
        let y = ui.cursor_pos()[1];
        ui.set_cursor_pos([col_name + 8.0, y]);
        render_text_wrapped(true, false, ui, &perk_desc, col_name + 8.0, w - 24.0);

        // Reqs / limits line
        if !perk_reqs.is_empty() || !perk_lims.is_empty() {
            let y = ui.cursor_pos()[1];
            ui.set_cursor_pos([col_name + 8.0, y]);
            let reqs_str = if perk_reqs.is_empty() {
                "None".to_string()
            } else {
                perk_reqs.join(", ")
            };
            let lims_str = if perk_lims.is_empty() {
                "None".to_string()
            } else {
                perk_lims.join(", ")
            };
            render_text_wrapped(true, false, ui,
                &format!("Req: {}  |  Limits: {}", reqs_str, lims_str),
                col_name + 8.0, w - 24.0);
        }

        ui.separator();
        ui.spacing();
    }

    drop(_child);

    // ── Footer ────────────────────────────────────────────────────
    render_footer(ui, h, screen, remaining == 0);
}

/// Returns true if the popup was confirmed (apply the perk effect),
/// false if it was cancelled (remove the perk).
pub fn render_perk_resolution(
    ui: &Ui,
    window: &Window,
    popup: &mut PerkResolutionPopup,
    special: &mut [i64; 7],
    special_max: &[i32; 7],       // per-stat caps
    skills_state: &mut crate::screens::skills::SkillsState,
) -> Option<bool> { // Some(true)=confirmed, Some(false)=cancelled, None=still open
    if !popup.open { return Some(false); }

    let (win_w, win_h) = (800_u32, 600_u32); // use actual window size if available
    let pw = 380.0_f32;
    let ph = 220.0_f32;

    let Some(_tok) = ui.window(format!("##resolve_{}", popup.perk_id))
        .title_bar(false)
        .resizable(false)
        .movable(true)
        .size([pw, ph], imgui::Condition::Always)
        .position(
            [(win_w as f32 - pw) * 0.5, (win_h as f32 - ph) * 0.5],
            imgui::Condition::Appearing,
        )
        .begin()
    else { return None; };

    // Title + X
    ui.text(format!("Resolve: {}", popup.perk_name));
    ui.same_line_with_pos(pw - 32.0);
    if ui.button(format!("X##res_close_{}", popup.perk_id)) {
        popup.open = false;
        return Some(false); // cancelled — caller should remove_perk
    }
    ui.separator();
    ui.spacing();

    match &mut popup.resolution {

        // ── Intense Training: pick one SPECIAL stat ───────────────
        PerkResolution::IntenseTraining { selected_stat } => {
            ui.text("Increase one SPECIAL stat by 1:");
            ui.spacing();
            ui.set_next_item_width(220.0);

            let preview = selected_stat
                .map(|i| SPECIAL_LABELS[i])
                .unwrap_or("-- Select stat --");

            if let Some(_cb) = ui.begin_combo("##it_stat", preview) {
                for i in 0..7 {
                    if special[i] as i32 >= special_max[i] { 
                        let _g = ui.begin_disabled(true);
                        ui.selectable_config(
                            &format!("{} (at cap)", SPECIAL_LABELS[i])
                        ).build();
                        drop(_g);
                        continue; 
                    }
                    let sel = *selected_stat == Some(i);
                    if ui.selectable_config(SPECIAL_LABELS[i]).selected(sel).build() {
                        *selected_stat = Some(i);
                    }
                }
            }
            if *selected_stat == Some(4) {
                ui.spacing();
                ui.text_wrapped("Remember to update your skills on the previous page")
            }
        }

        // ── Skilled: +2 to one skill OR +1 to two skills ──────────
        PerkResolution::Skilled { mode, skill_a, skill_b } => {
            ui.text("Choose distribution:");
            ui.spacing();

            let mut mode_val = matches!(mode, SkilledMode::OneToTwo);
            if ui.radio_button_bool("+2 to one skill##sk2", !mode_val) {
                *mode = SkilledMode::TwoToOne;
                *skill_b = None;
            }
            ui.same_line();
            if ui.radio_button_bool("+1 to two skills##sk1", mode_val) {
                *mode = SkilledMode::OneToTwo;
            }

            ui.spacing();

            let bonus_a = if matches!(mode, SkilledMode::TwoToOne) { 2 } else { 1 };

            // Skill A
            ui.text(if matches!(mode, SkilledMode::TwoToOne) { "+2 Skill:" } else { "+1 Skill 1:" });
            ui.same_line();
            ui.set_next_item_width(200.0);
            let preview_a = skill_a.map(|i| SKILLS[i]).unwrap_or("-- Select --");
            if let Some(_cb) = ui.begin_combo("##sk_a", preview_a) {
                for (si, &name) in SKILLS.iter().enumerate() {
                    /* 
                    let already_tagged = skills_state.skills[si].tagged;
                    let would_exceed_cap = skills_state.skills[si].ranks + 2 > skills_state.max_rank_for(si);
                    let disabled = already_tagged || would_exceed_cap;
                    let _g = disabled.then(|| ui.begin_disabled(true));
                    let sel = *selected_skill == Some(si);
                    let label = if already_tagged {
                        format!("{} (already tagged)", name)
                    } else if would_exceed_cap {
                        format!("{} (would exceed cap)", name)
                    } else {
                        name.to_string()
                    };
                    */
                    let tag_bonus = if skills_state.skills[si].tagged { 2 } else { 0 };
                    let at_sk_cap = skills_state.skills[si].ranks + tag_bonus >= skills_state.max_rank_for(si);
                    let would_exceed_cap = skills_state.skills[si].ranks + tag_bonus + 2
                    >= skills_state.max_rank_for(si);
                    let is_b = *skill_b == Some(si);
                    let disabled = at_sk_cap || is_b || would_exceed_cap;
                    let _g = disabled.then(|| ui.begin_disabled(true));
                    let sel = *skill_a == Some(si);
                    let label = if at_sk_cap { format!("{} (cap)", name) } else if would_exceed_cap { format!("{} (would exceed cap)", name) } else { name.to_string() };
                    if ui.selectable_config(&label).selected(sel).build() {
                        *skill_a = Some(si);
                    }
                    drop(_g);
                }
            }

            // Skill B (OneToTwo only)
            if matches!(mode, SkilledMode::OneToTwo) {
                ui.spacing();
                ui.text("+1 Skill 2:");
                ui.same_line();
                ui.set_next_item_width(200.0);
                let preview_b = skill_b.map(|i| SKILLS[i]).unwrap_or("-- Select --");
                if let Some(_cb) = ui.begin_combo("##sk_b", preview_b) {
                    for (si, &name) in SKILLS.iter().enumerate() {
                        let tag_bonus = if skills_state.skills[si].tagged { 2 } else { 0 };
                        let at_sk_cap = skills_state.skills[si].ranks + tag_bonus >= skills_state.max_rank_for(si);
                        let is_a = *skill_a == Some(si);
                        let disabled = at_sk_cap || is_a;
                        let _g = disabled.then(|| ui.begin_disabled(true));
                        let sel = *skill_b == Some(si);
                        let label = if at_sk_cap { format!("{} (cap)", name) } else { name.to_string() };
                        if ui.selectable_config(&label).selected(sel).build() {
                            *skill_b = Some(si);
                        }
                        drop(_g);
                    }
                }
            }
        }

        // ── Tag!: pick one more tag skill ─────────────────────────
        PerkResolution::Tag { selected_skill } => {
            ui.text("Tag one additional skill:");
            ui.spacing();
            ui.set_next_item_width(220.0);

            let preview = selected_skill
                .map(|i| SKILLS[i])
                .unwrap_or("-- Select skill --");

            if let Some(_cb) = ui.begin_combo("##tag_skill", preview) {
                for (si, &name) in SKILLS.iter().enumerate() {
                    let already_tagged = skills_state.skills[si].tagged;
                    let would_exceed_cap = skills_state.skills[si].ranks + 2 > skills_state.max_rank_for(si);
                    let disabled = already_tagged || would_exceed_cap;
                    let _g = disabled.then(|| ui.begin_disabled(true));
                    let sel = *selected_skill == Some(si);
                    let label = if already_tagged {
                        format!("{} (already tagged)", name)
                    } else if would_exceed_cap {
                        format!("{} (would exceed cap)", name)
                    } else {
                        name.to_string()
                    };
                    if ui.selectable_config(&label).selected(sel).build() {
                        *selected_skill = Some(si);
                    }
                    drop(_g);
                }
            }
        }
    }

    ui.spacing();
    ui.separator();
    ui.spacing();

    // Confirm button — only enabled when selection is complete
    let complete = PerksState::is_resolution_complete(popup);
    let _g = (!complete).then(|| ui.begin_disabled(true));
    if ui.button(format!("Confirm##res_confirm_{}", popup.perk_id)) {
        // Apply the effect
        apply_resolution(popup, special, skills_state);
        popup.open = false;
        return Some(true);
    }
    drop(_g);

    if !complete {
        ui.same_line();
        ui.text_disabled("Make a selection to confirm.");
    }

    None
}

fn apply_resolution(
    popup: &PerkResolutionPopup,
    special: &mut [i64; 7],
    skills_state: &mut crate::screens::skills::SkillsState,
) {
    match &popup.resolution {
        PerkResolution::IntenseTraining { selected_stat: Some(i) } => {
            special[*i] += 1;
            // Also update skills_state so INT change reflects on skill points
            if *i == 4 {
                let int_stat = special[*i];
                skills_state.intelligence = int_stat as i32};
        }
        PerkResolution::Skilled { mode, skill_a, skill_b } => {
            if let Some(a) = skill_a {
                let bonus = if matches!(mode, SkilledMode::TwoToOne) { 2 } else { 1 };
                skills_state.skills[*a].ranks =
                    (skills_state.skills[*a].ranks + bonus)
                    .min(skills_state.max_rank_for(*a));
            }
            if let Some(b) = skill_b {
                skills_state.skills[*b].ranks =
                    (skills_state.skills[*b].ranks + 1)
                    .min(skills_state.max_rank_for(*b));
            }
        }
        PerkResolution::Tag { selected_skill: Some(si) } => {
            skills_state.skills[*si].tagged = true;
            // Increase tag slot count so the skills screen counter stays correct
            skills_state.perk_tag_slots += 1;
            skills_state.extra_tags.push(*si);
        }
        _ => {}
    }
}

fn render_footer(ui: &Ui, win_h: f32, screen: &mut AppScreen, perks_complete: bool) {
    let footer_y = win_h - 44.0;
    ui.set_cursor_pos([16.0, footer_y]);
    if ui.button("< Back") {
        *screen = AppScreen::Skills;
    }
    ui.same_line();
    let _g = (!perks_complete).then(|| ui.begin_disabled(true));
    if ui.button("Next >") {
        // TODO: next screen
    }
    drop(_g);
}