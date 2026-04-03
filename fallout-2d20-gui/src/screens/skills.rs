use imgui::Ui;
use sdl2::video::Window;
use crate::{AppScreen, BAR_HEIGHT};
use crate::screens::new_character::render_text_wrapped;
use crate::screens::special::{SpecialState, MutantType};

// ── Constants ─────────────────────────────────────────────────────────────────

// Full 18-skill list — adjust order to match your constants
pub const SKILLS: [&str; 17] = [
    "Athletics", "Barter", "Big Guns", "Energy Weapons", "Explosives",
    "Lockpick", "Medicine", "Melee Weapons", "Pilot", "Repair",
    "Science", "Small Guns", "Sneak", "Speech", "Survival",
    "Throwing", "Unarmed",
];

// Trait IDs that grant extra tag skills (match your DB)
// Trait 13 gives 2 extra; all others give 1
const TRAIT_EXTRA_TAG: &[i32] = &[1, 2, 5, 11, 12, 21, 24];
//const TRAIT_EXTRA_TAG_2: i32 = 13;
const TRAIT_TRIBAL: i32 = 27; // forbids tagging Science
// Traits whose extra tags are restricted to specific skills:
const TRAIT_BROTHERHOOD: &[i32] = &[1, 24];  // Energy Weapons/Repair/Science
const TRAIT_MINUTEMAN: i32 = 12;                       // Small Guns/Energy Weapons
const TRAIT_GOOD: i32 = 13;                         // Speech/Medicine/Repair/Science/Barter
const TRAIT_ANY_SKILL: &[i32] = &[5, 11, 21];           // any skill
// Limited max traits
//const TRAIT_LIMITED_COMBAT: i32 = 13;  // combat skills capped at 4
const TRAIT_MUTANT: &[i32] = &[3, 25]; // all skills capped at 4

const COMBAT_SKILLS: &[&str] = &[
    "Athletics", "Big Guns", "Energy Weapons", "Explosives",
    "Lockpick", "Melee Weapons", "Pilot", "Small Guns",
    "Sneak", "Survival", "Throwing", "Unarmed",
];

// ── State ─────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct SkillEntry {
    pub ranks: i32,
    pub tagged: bool,
}

impl SkillEntry {
    fn new() -> Self { Self { ranks: 0, tagged: false } }
    pub fn total(&self) -> i32 { self.ranks + if self.tagged { 2 } else { 0 } }
}

pub struct SkillsState {
    pub skills: [SkillEntry; 17],   // parallel to SKILLS array
    pub extra_tags: Vec<usize>,     // indices into SKILLS of chosen extra tags
    pub is_ghoul: bool,

    // Derived from traits (set externally each frame)
    pub extra_tag_count: usize,
    pub extra_tag_options: Vec<usize>,  // skill indices valid for extra tags
    pub forced_tag: Option<usize>,
    pub forbidden_tag: Option<usize>,
    pub limited_skills: Vec<usize>,     // indices whose max is 4
    pub all_limited: bool,              // true = ALL skills capped at 4

    // From SPECIAL
    pub intelligence: i32,
    pub level: i32,
    
    pub perk_tag_slots: usize,
    pub perk_skill_bonus: i32,
}

impl SkillsState {
    pub fn new(intelligence: i32, level: i32) -> Self {
        Self {
            skills: std::array::from_fn(|_| SkillEntry::new()),
            extra_tags: vec![],
            is_ghoul: false,
            extra_tag_count: 0,
            extra_tag_options: vec![],
            forced_tag: None,
            forbidden_tag: None,
            limited_skills: vec![],
            all_limited: false,
            intelligence,
            level,
            perk_tag_slots: 0,
            perk_skill_bonus: 0,
        }
    }

    pub fn max_skill_points(&self) -> i32 {
        9 + self.intelligence + self.level - 1
    }

    pub fn total_ranks(&self) -> i32 {
        self.skills.iter().map(|s| s.ranks).sum()
    }

    pub fn remaining_points(&self) -> i32 {
        self.max_skill_points() - self.total_ranks() + self.perk_skill_bonus
    }

    /// Max rank for a skill at current level (3..=6 clamp)
    pub fn max_rank(&self) -> i32 {
        self.level.clamp(3, 6)
    }

    /// Max rank for a specific skill index (respects limited traits)
    pub fn max_rank_for(&self, si: usize) -> i32 {
        let base = self.max_rank();
        if self.all_limited { return base.min(4); }
        if self.limited_skills.contains(&si) { return base.min(4); }
        base
    }

    pub fn total_tagged(&self) -> usize {
        self.skills.iter().enumerate()
        .filter(|(si, s)| s.tagged && self.forced_tag != Some(*si))
        .count()
    }

    pub fn base_tag_slots(&self) -> usize { 3 }

    pub fn total_tag_slots(&self) -> usize {
        self.base_tag_slots() + self.extra_tag_count + self.perk_tag_slots
    }

    /// Are all extra tag slots filled?
    pub fn extra_tags_complete(&self) -> bool {
        self.extra_tags.len() >= self.extra_tag_count
    }

    /// Is the standard tagging phase unlocked?
    pub fn can_tag_standard(&self) -> bool {
        self.extra_tag_count == 0 || self.extra_tags_complete()
    }

    /// Apply forced tags (ghoul → Survival)
    pub fn apply_forced_tags(&mut self) {
        if let Some(fi) = self.forced_tag {
            self.skills[fi].tagged = true;
        }
    }
}

// ── Trait analysis — call this each frame from main.rs ───────────────────────

/// Call once per frame to sync trait-derived flags into SkillsState
pub fn sync_trait_effects(
    state: &mut SkillsState,
    selected_trait_ids: &[i32],
    is_ghoul: bool,
) {
    let has = |id: i32| selected_trait_ids.contains(&id);

    state.is_ghoul = is_ghoul;

    // Extra tag count
    state.extra_tag_count = if has(TRAIT_GOOD) { 2 }
        else if TRAIT_EXTRA_TAG.iter().any(|&id| has(id)) { 1 }
        else { 0 };

    // Extra tag options
    state.extra_tag_options = if TRAIT_BROTHERHOOD.iter().any(|&id| has(id)) {
        skill_indices(&["Energy Weapons", "Repair", "Science"])
    } else if has(TRAIT_MINUTEMAN) {
        skill_indices(&["Small Guns", "Energy Weapons"])
    } else if has(TRAIT_GOOD) {
        skill_indices(&["Speech", "Medicine", "Repair", "Science", "Barter"])
    } else if TRAIT_ANY_SKILL.iter().any(|&id| has(id)) {
        (0..SKILLS.len()).collect()
    } else if is_ghoul {
        skill_indices(&["Survival"])
    } else {
        vec![]
    };

    // Forced tag
    state.forced_tag = if is_ghoul { skill_index("Survival") } else { None };

    // Forbidden tag
    state.forbidden_tag = if has(TRAIT_TRIBAL) {
        skill_index("Science")
    } else { None };

    // Limited skills
    state.all_limited = TRAIT_MUTANT.iter().any(|&id| has(id));
    state.limited_skills = if has(TRAIT_GOOD) && !state.all_limited {
        skill_indices(COMBAT_SKILLS)
    } else {
        vec![]
    };

    // Remove extra_tags that are no longer valid options
    state.extra_tags.retain(|&si| state.extra_tag_options.contains(&si));

    // Apply forced tags
    state.apply_forced_tags();
}

fn skill_index(name: &str) -> Option<usize> {
    SKILLS.iter().position(|&s| s == name)
}

fn skill_indices(names: &[&str]) -> Vec<usize> {
    names.iter().filter_map(|&n| skill_index(n)).collect()
}

// ── Render ────────────────────────────────────────────────────────────────────

pub fn render_skills(
    ui: &Ui,
    window: &Window,
    state: &mut SkillsState,
    screen: &mut AppScreen,
) {
    let (win_w, win_h) = window.size();
    let content_h = win_h as f32 - BAR_HEIGHT;
    let w = (win_w as f32 * 0.65).min(960.0);
    let h = win_h as f32 * 0.85;

    let Some(_tok) = ui.window("##skills")
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
    ui.text("SKILLS");
    ui.separator();
    ui.spacing();

    let remaining = state.remaining_points();
    let total_tagged = state.total_tagged();
    let tag_slots = state.total_tag_slots();

    // Skill points
    if remaining < 0 {
        render_text_wrapped(true, false, ui,
            &format!("Skill Points: {}/{} ({})",
                state.total_ranks(), state.max_skill_points(), remaining),
            0.0, w);
    } else {
        ui.text(format!("Skill Points: {}/{} ({} remaining)",
            state.total_ranks(), state.max_skill_points(), remaining));
    }

    // Tag skill counter
    ui.same_line_with_pos(w * 0.5);
    let tags_ok = total_tagged == tag_slots;
    render_text_wrapped(tags_ok, !tags_ok, ui,
        &format!("Tag Skills: {}/{}", total_tagged, tag_slots),
        0.0, w);

    ui.spacing();
    ui.separator();
    ui.spacing();

    // ── Phase 1: Extra tag skills ──────────────────────────────────
    if state.extra_tag_count > 0 {
        ui.text(format!(
            "Extra Tag Skills ({}/{}):  select from the options below before tagging standard skills",
            state.extra_tags.len(), state.extra_tag_count
        ));
        ui.spacing();

        for &si in &state.extra_tag_options.clone() {
            let is_chosen = state.extra_tags.contains(&si);
            let at_limit = state.extra_tags.len() >= state.extra_tag_count;
            let is_forced = state.forced_tag == Some(si);
            let disabled = (!is_chosen && at_limit) || is_forced;

            let _g = disabled.then(|| ui.begin_disabled(true));
            let mut checked = is_chosen || is_forced;
            if ui.checkbox(format!("{}##extratag_{}", SKILLS[si], si), &mut checked) {
                if checked {
                    if !state.extra_tags.contains(&si) {
                        state.extra_tags.push(si);
                        state.skills[si].tagged = true;
                    }
                } else {
                    state.extra_tags.retain(|&x| x != si);
                    // Only untag if not a forced tag
                    if state.forced_tag != Some(si) {
                        state.skills[si].tagged = false;
                    }
                }
            }
        }

        ui.spacing();
        ui.separator();
        ui.spacing();

        if !state.extra_tags_complete() {
            render_text_wrapped(true, false, ui,
                &format!("Select {} more extra tag skill(s) to continue.",
                    state.extra_tag_count - state.extra_tags.len()),
                0.0, w);
            render_footer(ui, h, screen, remaining, total_tagged, tag_slots, state.extra_tag_count, state.extra_tags_complete());
            return;
        }

        ui.text("Standard Tag Skills and Points:");
        ui.spacing();
    } else {
        ui.text("Tag Skills and Points:");
        ui.spacing();
    }

    // ── Phase 2: Skill list ───────────────────────────────────────
    // Column headers
    let col_skill  = 0.0_f32;
    let col_ranks  = 175.0_f32;
    let col_tag    = 290.0_f32;
    let col_total  = 330.0_f32;
    let col_max    = 420.0_f32;

    ui.text_disabled("Skill");
    ui.same_line_with_pos(col_ranks);  ui.text_disabled("Ranks");
    ui.same_line_with_pos(col_tag);    ui.text_disabled("Tag");
    ui.same_line_with_pos(col_total);  ui.text_disabled("Total");
    ui.same_line_with_pos(col_max);    ui.text_disabled("Max");
    ui.separator();

    let max_rank_base = state.max_rank();

    for si in 0..SKILLS.len() {
        let max_for = state.max_rank_for(si);
        // Effective max input = max_for minus the +2 from tag
        let tagged = state.skills[si].tagged;
        let tag_bonus = if tagged { 2 } else { 0 };
        let input_max = (max_for - tag_bonus).max(0);
        let ranks = state.skills[si].ranks;
        let total = state.skills[si].total();

        let is_forced  = state.forced_tag == Some(si);
        let is_forbidden = state.forbidden_tag == Some(si);
        let is_extra_tag = state.extra_tags.contains(&si);

        // Skill label
        ui.text(SKILLS[si]);
        ui.same_line_with_pos(col_ranks);

        // Ranks: - / value / +
        let can_dec = ranks > 0;
        let can_inc = ranks < input_max && remaining > 0;

        let _dec = (!can_dec).then(|| ui.begin_disabled(true));
        if ui.button(format!("-##r_{}", si)) {
            state.skills[si].ranks -= 1;
        }
        drop(_dec);
        ui.same_line();
        ui.text(format!("{:2}", ranks));
        ui.same_line();
        let _inc = (!can_inc).then(|| ui.begin_disabled(true));
        if ui.button(format!("+##r_{}", si)) {
            state.skills[si].ranks += 1;
        }
        drop(_inc);

        // Tag checkbox
        ui.same_line_with_pos(col_tag);
        let tag_at_limit = !tagged && total_tagged >= tag_slots;
        let tag_would_overflow = ranks > (max_for - 2).max(0);
        let tag_disabled = is_forbidden
            || is_extra_tag          // already handled in extra tag section
            || is_forced
            || tag_at_limit
            || tag_would_overflow;

        let _tg = tag_disabled.then(|| ui.begin_disabled(true));
        let mut tag_val = tagged;
        if ui.checkbox(format!("##tag_{}", si), &mut tag_val) {
            // Don't allow untagging forced or extra tags here
            if !is_forced && !is_extra_tag {
                state.skills[si].tagged = tag_val;
            }
        }
        drop(_tg);

        // Total
        ui.same_line_with_pos(col_total);
        if tagged {
            render_text_wrapped(false, true, ui, &format!("{}", total), 0.0, w);
        } else {
            ui.text(format!("{}", total));
        }

        // Max
        ui.same_line_with_pos(col_max);
        ui.text_disabled(format!("{}", max_for));

        // Forbidden/forced label
        if is_forbidden {
            ui.same_line();
            render_text_wrapped(true, false, ui, "[cannot tag]", 0.0, w);
        } else if is_forced {
            ui.same_line();
            render_text_wrapped(false, true, ui, "[forced]", 0.0, w);
        }
    }

    render_footer(ui, h, screen, remaining, total_tagged, tag_slots, state.extra_tag_count, state.extra_tags_complete());
}

// ── Footer ────────────────────────────────────────────────────────────────────

fn render_footer(ui: &Ui, win_h: f32, screen: &mut AppScreen, remaining: i32, total_tagged: usize, tag_slots: usize, extra_tag_count: usize, extra_tags_complete: bool) {
    let footer_y = win_h - 48.0;
    ui.set_cursor_pos([16.0, footer_y]);
    if ui.button("< Back") {
        *screen = AppScreen::Special;
    }
    ui.same_line();

    let skills_complete = remaining == 0
        && total_tagged == tag_slots
        && (extra_tag_count == 0 || extra_tags_complete);

    let _next_gate = (!skills_complete).then(|| ui.begin_disabled(true));
    // Next is only enabled when points and tags are satisfied
    // (caller should pass validation state, for now gated by button label)
    if ui.button("Next >") {
        * screen = AppScreen::Perks;
    }
    if !skills_complete {
        ui.same_line();
        let hint = if remaining != 0 {
            format!("{} skill point(s) unspent", remaining.abs())
        } else if total_tagged < tag_slots {
            format!("{} tag skill(s) unselected", tag_slots - total_tagged)
        } else {
            "Select extra tag skills first".to_string()
        };
        render_text_wrapped(true, false, ui, &hint, 0.0, win_h);
    }
}