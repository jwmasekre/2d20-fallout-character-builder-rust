use std::thread::current;

use imgui::Ui;
use sdl2::video::Window;
use serde_json;
use crate::db::Db;
use crate::AppScreen;
use crate::theme::{render_text_wrapped, render_window};
use crate::screens2::special_assignment::SPECIAL_LABELS;
use crate::character::{Character, CompanionType, MutantType, Perk, RobotType};

pub struct PerkState {
    pub perks: Vec<PerkRow>,
    pub taken_count: i32,
    pub perk_lim: i32,
    pub show_eligible_only: bool,
    pub filters: [bool; 8],
    pub pending_resolution: Option<i32>,
}
impl PerkState {
    pub fn new(db: &Db, character: &Character) -> Self {
        let perks = load_perks(db);
        let taken_count = character.perks.iter().map(|p| p.ranks).sum();
        let perk_lim = character.level + if character.traits.iter().any(|t| t.id == 10) { 1 } else { 0 };
        Self {
            perks,
            taken_count,
            perk_lim,
            show_eligible_only: false,
            filters: [true; 8],
            pending_resolution: None,
        }
    }
    pub fn is_complete(&self) -> bool {
        self.perk_lim == self.taken_count
    }
    pub fn update(&self, character: &Character) -> Self {
        let perks = self.perks.to_vec();
        let taken_count = character.perks.iter().map(|p| p.ranks).sum();
        let perk_lim = character.level + if character.traits.iter().any(|t| t.id == 10) { 1 } else { 0 };
        let show_eligible_only = self.show_eligible_only;
        let filters = self.filters;
        let pending_resolution = self.pending_resolution;
        Self {
            perks,
            taken_count,
            perk_lim,
            show_eligible_only,
            filters,
            pending_resolution,
        }
    }
    fn perk_filter_indices(perk: &PerkRow) -> Vec<usize> {
        let stat_reqs: Vec<&str> = perk.reqs
            .iter()
            .filter(|r| r.contains(':'))
            .map(|r| r
                .splitn(2,':')
                .next()
                .unwrap_or("")
                .trim())
            .collect();
        if stat_reqs.is_empty() {
            return vec![0];
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
    fn perk_passes_filter(&self, perk: &PerkRow) -> bool {
        Self::perk_filter_indices(perk)
            .iter()
            .any(|&i| self.filters[i])
    }
    fn is_taken(&self, perk: &PerkRow, character: &Character,) -> bool {
        character.perks.iter().any(|p| p.id == perk.id)
    }
    fn is_eligible(&self, perk: &PerkRow, character: &Character,) -> bool {
        let taken = self.is_taken(perk, character);
        let taken_ranks = if taken { character.perks.iter().find(|p| p.id == perk.id).map(|p| p.ranks).unwrap_or(0).into() } else { 0 };
        //at max
        if taken_ranks >= perk.ranks { return false; }
        let next_rank_lvl = perk.level_req + taken_ranks * perk.rank_range;
        //doesn't meet level requirement for next rank
        if character.level < next_rank_lvl { return false }
        //special requirements
        for req in &perk.reqs {
            if req.contains(':') {
                let parts: Vec<&str> = req.splitn(2,':').collect();
                if parts.len() != 2 { continue; }
                let stat: &str = parts[0].trim();
                let val: i32 = parts[1].trim().parse().unwrap_or(0);
                let meets = match stat {
                    "strength"     => { character.special.strength.value >= val }
                    "perception"   => { character.special.strength.value >= val }
                    "endurance"    => { character.special.strength.value >= val }
                    "charisma"     => { character.special.strength.value >= val }
                    "intelligence" => { character.special.strength.value >= val }
                    "agility"      => { character.special.strength.value >= val }
                    "luck"         => { character.special.strength.value >= val }
                    _ => { true }
                };
                if !meets { return false }
            }
            if req.trim().to_lowercase() == "book" {
                return false
            }
        }
        //other limits
        for limit in &perk.limits {
            let lower = limit.to_lowercase();
            if lower.contains("daring nature") && character.perks.iter().any(|p| p.id == 25) ||
                lower.contains("cautious nature") && character.perks.iter().any(|p| p.id == 18) ||
                lower.contains("robot") && character.is_robot() ||
                lower.contains("ghoul") && character.ghoul ||
                lower.contains("rads") && (character.is_robot() || character.ghoul || character.is_mutant()) ||
                lower.contains("companion") && character.companion != CompanionType::None { return false }
        }
        true
    }
}

#[derive(Debug, Clone)]
pub struct PerkRow {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub level_req: i32,
    pub ranks: i32,
    pub rank_range: i32,
    pub reqs: Vec<String>,
    pub limits: Vec<String>,
    pub sourcebook: String,
}

pub fn load_perks(db: &Db) -> Vec<PerkRow> {
    let result = db.block_on(async {
        sqlx::query!(
            r#"
            SELECT p.id, p.name, p.description, p.ranks, p.rank_range, p.level_req, p.reqs, p.limits, s.name AS sourcebook
            FROM perks p
            JOIN sourcebooks s ON s.id = p.sourcebook_id
            ORDER BY s.id, p.name
            "#
        ).fetch_all(&db.pool).await
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
                id: r.id as i32,
                name: r.name.unwrap_or_default(),
                sourcebook: r.sourcebook.unwrap_or_default(),
                description: r.description.unwrap_or_default(),
                level_req: r.level_req.unwrap_or_default() as i32,
                ranks: r.ranks.unwrap_or_default() as i32,
                rank_range: r.rank_range.unwrap_or_default() as i32,
                limits,
                reqs,
            }
        }).collect(),
        Err(e) => { eprintln!("Failed to load perks: {e}"); vec![] }
    }
}

//perks that need to be resolved
const PERK_INTENSE_TRAINING: i32 = 45;
const PERK_SKILLED: i32 = 83;
const PERK_TAG: i32 = 92;
const PERK_BW_LK: i32 = 12;
const PERK_MM_CF: i32 = 110;

pub enum SkilledMode {
    TwoToOne,
    OneToTwo,
}

pub enum BwLk {
    BlackWidow,
    LadyKiller,
}

pub enum MmCf {
    MechanicalMenace,
    ClassFreak,
}

pub enum PerkResolution {
    IntenseTraining {
        selected_stat: Option<usize>
    },
    Skilled {
        mode: SkilledMode,
        skill_a: Option<usize>,
        skill_b: Option<usize>,
    },
    Tag {
        selected_skill: Option<usize>,
    },
    BwLk {
        version: BwLk,
    },
    MmCf {
        version: MmCf,
    },
}

pub struct PerkResolutionPopup {
    pub perk_id: i32,
    pub perk_name: String,
    pub resolution: PerkResolution,
    pub open: bool,
}

const FILTER_LABELS: [&str; 8] = ["X", "S", "P", "E", "C", "I", "A", "L"];

pub fn render_perk_select(
    ui: &Ui,
    window: &Window,
    state: &mut PerkState,
    screen: &mut AppScreen,
    db: &Db,
    character: &mut Character,
    resolving: bool,
) -> f32 {
    let (w, h) = render_window(ui, window, "##perk_select", "Perk Select");

    ui.text("Perks");
    ui.separator();
    ui.spacing();

    let remaining = state.perk_lim - state.taken_count;

    if remaining <= 0 {
        render_text_wrapped(false, true, ui, &format!("Perks: {}/{}", state.taken_count, state.perk_lim), 0.0, w);
    } else {
        ui.text(format!("Perks: {}/{} ({} remaining)", state.taken_count, state.perk_lim, remaining));
    }

    ui.spacing();

    let _res_guard = resolving.then(|| ui.begin_disabled(true));
    if resolving {
        render_text_wrapped(false, true, ui, "resolve the perk popup before continuing...", 0.0, w);
        ui.spacing();
    }

    //filters
    ui.checkbox("Show eligible only##eo", &mut state.show_eligible_only);
    ui.same_line();
    ui.text_disabled("|");
    ui.same_line();
    ui.text_disabled("Filter by SPECIAL:");
    ui.same_line();

    for i in 0..8 {
        if ui.checkbox(&format!("{}##sf_{}", FILTER_LABELS[i], i), &mut state.filters[i]) {}
        if i < 7 { ui.same_line(); }
    }

    ui.spacing();
    ui.separator();
    ui.spacing();

    //perk list
    let list_h = h - 140.0;
    let Some(_child) = ui.child_window("##perk_scroll")
        .size([w - 16.0, list_h])
        .begin()
    else { return h; };

    let col_name = 0.0_f32;
    let col_reqs = 260.0_f32;
    let col_ranks = 460.0_f32;
    let col_btns = 540.0_f32;

    //filtering perks
    let filtered: Vec<usize> = (0..state.perks.len())
        .filter(|&i| {
            let perk = &state.perks[i];
            state.perk_passes_filter(perk) && (!state.show_eligible_only || state.is_eligible(perk, &character))
        }).collect();
    //track which sourcebook is currently being printed
    let mut current_label = String::new();
    for &perk_index in &filtered {
        let perk = state.perks[perk_index];
        let id = perk.id;
        let name = perk.name.clone();
        let desc = perk.description.clone();
        let level = perk.level_req;
        let max = perk.ranks;
        let reqs = perk.reqs.clone();
        let lims = perk.limits.clone();
        let rank_rng = perk.rank_range;
        let taken = if state.is_taken(&perk, &character) { character.perks.iter().find(|p| p.id == id).map(|p| p.ranks).unwrap_or(0).into()} else { 0 };
        let eligible = state.is_eligible(&perk, &character);
        let at_cap = taken >= max;
        let available = remaining <= 0;

        //create dividers for each sourcebook
        let src = state.perks[perk_index].sourcebook.clone();
        if current_label != src {
            ui.separator();
            ui.text_disabled(format!(" ----- {} -----", src));
            ui.separator();
            current_label = src;
        }

        //tint the row based on state
        let cursor = ui.cursor_pos();
        let draw_list = ui.get_window_draw_list();
        let win_pos = ui.window_pos();
        let abs_x = win_pos[0] + cursor[0];
        let abs_y = win_pos[1] + cursor[1] - ui.scroll_y();
        let button_h = ui.clone_style().frame_padding[1] * 2.0 + ui.text_line_height();

        let cap_color = [0.15, 0.35, 0.15, 0.3_f32];
        let rank_color = [0.20, 0.40, 0.50, 0.3_f32];
        let avail_color = [0.10, 0.25, 0.10, 0.2_f32];
        let unav_color = [0.0, 0.0, 0.0, 0.0_f32];

        let tint = if at_cap { cap_color } else if eligible && taken > 0 { rank_color } else if eligible { avail_color } else { unav_color };
        let rect_fill = imgui::ImColor32::from_rgba_f32s(tint[0], tint[1], tint[2], tint[3]);
        if tint[3] > 0.0 {
            draw_list.add_rect_filled_multicolor(
                [abs_x - 4.0, abs_y - 4.0],
                [abs_x + w - 24.0, abs_y + button_h + 4.0],
                rect_fill, rect_fill, rect_fill, rect_fill
            );
        }

        //name and pips
        let pips = "*".repeat(taken as usize) + &"¤".repeat((max - taken) as usize);
        if at_cap {
            render_text_wrapped(false, true, ui, &format!("{} {}", name, pips), col_name, col_reqs);
        } else if eligible {
            ui.text(format!("{} {}", name, pips));
        } else {
            render_text_wrapped(true, false, ui, &format!("{} {}", name, pips), col_name, col_reqs);
        }

        //level requirements
        let mut lvl_string = String::new();
        for i in 0..max {
            if i == 0 { continue } else {
                let next_lvl = format!("/{}", level + (i * rank_rng));
                lvl_string.push_str(&next_lvl);
            }
        }
        ui.same_line_with_pos(col_reqs);
        ui.text_disabled(format!("lv {}{}", level, lvl_string));
        ui.same_line_with_pos(col_ranks);
        ui.text_disabled(format!("{}/{}", taken, max));

        //buttons
        ui.same_line_with_pos(col_btns);
        if taken == 0 {
            let _g = (!eligible || available).then(|| ui.begin_disabled(true));
            if ui.button(format!("Take##take_{}", id)) {
                let cperk = Perk {
                    id,
                    name,
                    //will want to split out the different ranks at some point
                    desc: vec![desc],
                    ranks: 1,
                };
                character.perks.push(cperk);
                state.update(&character);
            }
            drop(_g);
            ui.same_line();
            let _g2 = true.then(|| ui.begin_disabled(true));
            ui.button(format!("Drop##drop_{}", id));
            drop(_g2);
        } else if at_cap {
            let _g = true.then(|| ui.begin_disabled(true));
            ui.button(format!("Rank+##rankp_{}", id));
            drop(_g);
            ui.same_line();
            if ui.button(format!("Drop##drop_{}", id)) {
                for (i, cperk) in character.perks.iter_mut().enumerate() {
                    if cperk.id == id && cperk.ranks > 0 {
                        cperk.ranks -= 1;
                        state.update(character);
                    } else if cperk.id == id {
                        character.perks.remove(i);
                        state.update(character);
                    }
                }
            }
        } else {
            let _g = (!eligible || available).then(|| ui.begin_disabled(true));
            if ui.button(format!("Rank+##rankp_{}", id)) {

            }
        }
    }

    return h
}