use imgui::Ui;
use sdl2::video::Window;
use crate::{AppScreen, BAR_HEIGHT};
use crate::screens::new_character::render_text_wrapped;
use crate::screens::special::SpecialState;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PerkRow {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub level_req: i32,
    pub ranks: i32,
    pub rank_range: i32,   // additional levels required per rank
    pub reqs: Vec<String>, // e.g. "strength: 6", "book"
    pub limits: Vec<String>, // e.g. "no ghoul", "no robot"
}

#[derive(Debug, Clone)]
pub struct CharPerk {
    pub perk_id: i64,
    pub ranks: i32,
}

// ── State ─────────────────────────────────────────────────────────────────────

pub struct PerksState {
    pub all_perks: Vec<PerkRow>,
    pub char_perks: Vec<CharPerk>,  // taken perks

    // Character context
    pub level: i32,
    pub special: [i32; 7],         // display values (post-modifier)
    pub is_ghoul: bool,
    pub is_robot: bool,
    pub is_super_mutant: bool,
    pub has_companion: bool,
    pub has_trait_swift_learner: bool, // trait 10 = +1 perk slot

    // Filter state
    pub show_eligible_only: bool,
    pub special_filters: [bool; 8], // X, S, P, E, C, I, A, L
}

impl PerksState {
    pub fn new(
        all_perks: Vec<PerkRow>,
        level: i32,
        special: [i32; 7],
        is_ghoul: bool,
        is_robot: bool,
        is_super_mutant: bool,
        has_companion: bool,
        has_trait_swift_learner: bool,
    ) -> Self {
        Self {
            all_perks,
            char_perks: vec![],
            level,
            special,
            is_ghoul,
            is_robot,
            is_super_mutant,
            has_companion,
            has_trait_swift_learner,
            show_eligible_only: false,
            special_filters: [true; 8],
        }
    }

    pub fn max_perks(&self) -> i32 {
        self.level + if self.has_trait_swift_learner { 1 } else { 0 }
    }

    pub fn perks_taken(&self) -> i32 {
        self.char_perks.len() as i32
    }

    pub fn perks_remaining(&self) -> i32 {
        self.max_perks() - self.perks_taken()
    }

    pub fn get_ranks(&self, perk_id: i64) -> i32 {
        self.char_perks.iter()
            .find(|p| p.perk_id == perk_id)
            .map(|p| p.ranks)
            .unwrap_or(0)
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
                let val: i32 = parts[1].trim().parse().unwrap_or(0);
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

    fn special_value_for(&self, stat: &str) -> i32 {
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
    let w = (win_w as f32 * 0.80).min(1100.0);
    let h = win_h as f32 * 0.90;

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

    for &pi in &filtered {
        // Re-borrow inside loop to avoid holding state borrow
        let perk_id   = state.all_perks[pi].id;
        let perk_name = state.all_perks[pi].name.clone();
        let perk_desc = state.all_perks[pi].description.clone();
        let perk_lvl  = state.all_perks[pi].level_req;
        let perk_max  = state.all_perks[pi].ranks;
        let perk_reqs = state.all_perks[pi].reqs.clone();
        let perk_lims = state.all_perks[pi].limits.clone();

        let ranks_taken = state.get_ranks(perk_id);
        let eligible    = state.is_eligible(&state.all_perks[pi].clone());
        let at_cap      = ranks_taken >= perk_max;
        let no_slots    = state.perks_remaining() <= 0;

        // Row background tint based on status
        let cursor = ui.cursor_pos();
        let draw_list = ui.get_window_draw_list();
        let win_pos = ui.window_pos();
        let abs_x = win_pos[0] + cursor[0];
        let abs_y = win_pos[1] + cursor[1] - ui.scroll_y();
        let row_h = 80.0_f32;

        let tint = if at_cap {
            [0.15, 0.35, 0.15, 0.3_f32]  // fully taken — green tint
        } else if eligible && ranks_taken > 0 {
            [0.20, 0.40, 0.50, 0.3_f32]  // rank available — blue tint
        } else if eligible {
            [0.10, 0.25, 0.10, 0.2_f32]  // available — subtle green
        } else {
            [0.0, 0.0, 0.0, 0.0_f32]     // unavailable — no tint
        };

        if tint[3] > 0.0 {
            draw_list.add_rect_filled(
                [abs_x - 4.0, abs_y - 2.0],
                [abs_x + w - 24.0, abs_y + row_h],
                imgui::ImColor32::from_rgba_f32s(tint[0], tint[1], tint[2], tint[3]),
            );
        }

        // Perk name + rank pips
        let pips = "★".repeat(ranks_taken as usize) + &"☆".repeat((perk_max - ranks_taken) as usize);
        if at_cap {
            render_text_wrapped(false, true, ui,
                &format!("{} {}", perk_name, pips), col_name, col_reqs);
        } else if eligible {
            ui.text(format!("{} {}", perk_name, pips));
        } else {
            render_text_wrapped(true, false, ui,
                &format!("{} {}", perk_name, pips), col_name, col_reqs);
        }

        // Level req
        ui.same_line_with_pos(col_reqs);
        ui.text_disabled(format!("Lvl {}", perk_lvl));

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