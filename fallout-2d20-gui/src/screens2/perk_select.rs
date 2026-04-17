use std::iter::repeat;
use imgui::Ui;
use sdl2::video::Window;
use serde_json;
use crate::db::Db;
use crate::main2::AppScreen;
use crate::screens2::skill_assignment::SKILLS;
use crate::theme::{render_text_wrapped, render_window};
use crate::screens2::special_assignment::{SPECIAL_LABELS};
use crate::character::{Character, CompanionType, TagType, Perk};

pub struct PerkState {
    pub perks: Vec<PerkRow>,
    pub taken_count: i32,
    pub perk_lim: i32,
    pub show_eligible_only: bool,
    pub filters: [bool; 8],
    pub pending_resolution: Option<(i32, bool)>,
}
impl PerkState {
    pub fn new(db: &Db, character: &Character) -> Self {
        let perks = load_perks(db);
        let taken_count = character.perks.iter().map(|p| p.ranks).sum();
        let perk_lim = character.level + if character.has_trait(10) { 1 } else { 0 };
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
    pub fn update(&self, character: &mut Character) -> Self {
        let perks = self.perks.to_vec();
        let taken_count = character.perks.iter().map(|p| p.ranks).sum();
        let perk_lim = character.level + if character.has_trait(10) { 1 } else { 0 };
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
        character.has_perk(perk.id)
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
            if lower.contains("daring nature") && character.has_perk(25) ||
                lower.contains("cautious nature") && character.has_perk(18) ||
                lower.contains("robot") && character.is_robot() ||
                lower.contains("ghoul") && character.ghoul ||
                lower.contains("rads") && (character.is_robot() || character.ghoul || character.is_mutant()) ||
                lower.contains("companion") && character.companion != CompanionType::None { return false }
        }
        true
    }
    pub fn begin_resolve(&self, perk: &PerkRow, add: bool) -> Option<PerkResolutionPopup> {
        let resolution = match perk.id {
            12 => Some(PerkResolution::BwLk { version: None }),
            45 => Some(PerkResolution::IntenseTraining { selected_stat: None }),
            83 => Some(PerkResolution::Skilled { skill_a: None, skill_b: None }),
            92 => Some(PerkResolution::Tag { selected_skill: None }),
            110 => Some(PerkResolution::MmCf { version: None }),
            _ => None,
        };
        resolution.map(|r| PerkResolutionPopup { perk_id: perk.id, perk_name: perk.name.clone(), resolution: r, perk_add: add, open: true })
    }
    fn is_resolution_complete(popup: &PerkResolutionPopup) -> bool {
        match &popup.resolution {
            PerkResolution::BwLk { version } => version.is_some(),
            PerkResolution::IntenseTraining { selected_stat } => selected_stat.is_some(),
            PerkResolution::Skilled { skill_a, skill_b } => skill_a.is_some() && skill_b.is_some(),
            PerkResolution::Tag { selected_skill } => selected_skill.is_some(),
            PerkResolution::MmCf { version } => version.is_some(),
        }
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

#[derive(PartialEq, Clone)]
pub enum BwLk {
    BlackWidow,
    LadyKiller,
}

impl BwLk {
    fn to_perk_string(&self) -> &str {
        match self {
            BwLk::BlackWidow => "Black Widow (masc)",
            BwLk::LadyKiller => "Lady Killer (fem)"
        }
    }
}

#[derive(PartialEq, Clone)]
pub enum MmCf {
    MechanicalMenace,
    ClassFreak,
}

impl MmCf {
    fn to_perk_string(&self) -> &str {
        match self {
            MmCf::MechanicalMenace => "Mechanical Menace (robots)",
            MmCf::ClassFreak => "Class Freak (mut. humans)"
        }
    }
}

pub enum PerkResolution {
    IntenseTraining {
        selected_stat: Option<usize>
    },
    Skilled {
        skill_a: Option<usize>,
        skill_b: Option<usize>,
    },
    Tag {
        selected_skill: Option<usize>,
    },
    BwLk {
        version: Option<BwLk>,
    },
    MmCf {
        version: Option<MmCf>,
    },
}

pub struct PerkResolutionPopup {
    pub perk_id: i32,
    pub perk_name: String,
    pub resolution: PerkResolution,
    pub perk_add: bool,
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

    ui.text("PERKS");
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
        let perk = &state.perks[perk_index];
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
                    desc: vec![desc.clone()],
                    ranks: 1,
                };
                character.perks.push(cperk);
                state.update(character);
                state.pending_resolution = Some((id, true));
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
                let cperk_len = character.perks.len();
                for i in 0..cperk_len {
                    if character.perks[i].id == id {
                        if character.perks[i].ranks > 1 {
                            character.perks[i].ranks -= 1;
                        } else {
                            character.perks.remove(i);
                        }
                        state.update(character);
                        state.pending_resolution = Some((id, false));
                    }
                }
            }
        } else {
            let _g = (!eligible || available).then(|| ui.begin_disabled(true));
            let cperk_len = character.perks.len();
            if ui.button(format!("Rank+##rankp_{}", id)) {
                for i in 0..cperk_len {
                    if character.perks[i].id == id {
                        character.perks[i].ranks += 1;
                        state.update(character);
                        state.pending_resolution = Some((id, true));
                    }
                }
            }
            drop(_g);
            ui.same_line();
            if ui.button(format!("Drop##drop_{}", id)) {
                for i in 0..cperk_len {
                    if character.perks[i].id == id {
                        if character.perks[i].ranks > 1 {
                            character.perks[i].ranks -= 1;
                        } else {
                            character.perks.remove(i);
                        }
                        state.update(character);
                        state.pending_resolution = Some((id, false));
                    }
                }
            }
        }
        //description
        let y = ui.cursor_pos()[1];
        ui.set_cursor_pos([col_name + 8.0, y]);
        render_text_wrapped(true, false, ui, &desc, col_name + 8.0, w - 24.0);
        //requirements and limits
        if !reqs.is_empty() || !lims.is_empty() {
            let y = ui.cursor_pos()[1];
            ui.set_cursor_pos([col_name + 8.0, y]);
            let req_str = if reqs.is_empty() {
                "none".to_string()
            } else {
                reqs.join(", ")
            };
            let lim_str = if lims.is_empty() {
                "none".to_string()
            } else {
                lims.join(", ")
            };
            render_text_wrapped(true, false, ui, &format!("Req: {} | Limits: {}", req_str, lim_str), col_name + 8.0, w - 24.0);
        }
        ui.separator();
        ui.spacing();
    }
    drop(_child);
    drop(_res_guard);

    return h
}

pub fn render_perk_resolution(
    ui: &Ui,
    window: &Window,
    popup: &mut PerkResolutionPopup,
    state: &mut PerkState,
    character: &mut Character,
) -> Option<bool> {
    //if the popup closes for whatever reason, return false
    if !popup.open { return Some(false); }
    let (win_w, win_h) = window.size();
    let (pw, ph) = (380.0_f32, 220.0_f32);

    let Some(_token) = ui.window(format!("##resolve_{}", popup.perk_id))
        .title_bar(false)
        .resizable(false)
        .movable(true)
        .size([pw, ph], imgui::Condition::Always)
        .position([(win_w as f32 - pw) * 0.5, (win_h as f32 - ph) * 0.5], imgui::Condition::Appearing)
        .begin()
    else { return None; };

    //title bar
    ui.text(format!("Resolve: {}", popup.perk_name));
    ui.same_line_with_pos(pw - 32.0);
    if ui.button(format!("X##res_close_{}", popup.perk_id)) {
        popup.open = false;
        return Some(false);
    }
    ui.separator();
    ui.spacing();

    match &mut popup.resolution {
        PerkResolution::BwLk { version } => {
            ui.text("Select a \"Preference\"");
            ui.spacing();
            ui.set_next_item_width(220.0);

            let preview = version.clone().map(|s| s.to_perk_string().to_string()).unwrap_or("-- Select Preference --".to_string());

            if let Some(_cb) = ui.begin_combo("##bwlk_choice", preview) {
                for (i, option) in [BwLk::BlackWidow,BwLk::LadyKiller].iter().enumerate() {
                    let sel = *version == Some(option.clone());
                    if ui.selectable_config(option.to_perk_string())
                        .selected(sel)
                        .build() { *version = Some(option.clone()); }
                    if sel {
                        ui.set_item_default_focus();
                    }
                }
            }
        }
        PerkResolution::IntenseTraining { selected_stat } => {
            let (res,inc) = state.pending_resolution.unwrap();
            if inc {
                ui.text("Increase one SPECIAL by 1:");
                ui.spacing();
                ui.set_next_item_width(220.0);

                let preview = selected_stat
                    .map(|i| SPECIAL_LABELS[i])
                    .unwrap_or("-- Select SPECIAL --");

                let at_max: [bool; 7] = [
                    character.special.strength.max <= character.special.strength.value,
                    character.special.perception.max <= character.special.perception.value,
                    character.special.endurance.max <= character.special.endurance.value,
                    character.special.charisma.max <= character.special.charisma.value,
                    character.special.intelligence.max <= character.special.intelligence.value,
                    character.special.agility.max <= character.special.agility.value,
                    character.special.luck.max <= character.special.luck.value,
                ];

                if let Some(_cb) = ui.begin_combo("##it_stat", preview) {
                    for i in 0..7 {
                        if at_max[i] {
                            let _g = ui.begin_disabled(true);
                            ui.selectable_config(&format!("{} (at cap)", SPECIAL_LABELS[i])).build();
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
            } else {
                ui.text("Select a trained SPECIAL to reduce:");
                ui.spacing();
                ui.set_next_item_width(220.0);

                let mut options: Vec<usize> = vec![];
                options.extend(repeat(0).take(character.special.strength.trained.try_into().unwrap()));
                options.extend(repeat(1).take(character.special.perception.trained.try_into().unwrap()));
                options.extend(repeat(2).take(character.special.endurance.trained.try_into().unwrap()));
                options.extend(repeat(3).take(character.special.charisma.trained.try_into().unwrap()));
                options.extend(repeat(4).take(character.special.intelligence.trained.try_into().unwrap()));
                options.extend(repeat(5).take(character.special.agility.trained.try_into().unwrap()));
                options.extend(repeat(6).take(character.special.luck.trained.try_into().unwrap()));

                let preview = selected_stat
                    .map(|i| SPECIAL_LABELS[i])
                    .unwrap_or("-- Select Trained SPECIAL --");

                if let Some(_cb) = ui.begin_combo("##it_dec", preview) {
                    for i in 0..options.len() {
                        let sel = *selected_stat == Some(options[i]);
                        if ui.selectable_config(SPECIAL_LABELS[options[i]]).selected(sel).build() {
                            *selected_stat = Some(options[i]);
                        }
                    }
                }
                if *selected_stat == Some(4) {
                    ui.spacing();
                    ui.text_wrapped("Remember to update your skills on the previous page")
                }
            }
        }
        PerkResolution::Skilled { skill_a, skill_b } => {
            let (res,inc) = state.pending_resolution.unwrap();
            if inc {
                ui.text("Select two skills to increase:");
                ui.text_disabled("The same skill can be selected twice");
                ui.spacing();
                ui.text("Skill 1:");
                ui.same_line();
                ui.set_next_item_width(200.0);
                let preview_a = skill_a.map(|i| SKILLS[i]).unwrap_or("-- Select --");
                let preview_b = skill_b.map(|i| SKILLS[i]).unwrap_or("-- Select --");
                
                let at_max: [(bool, bool); 17] = [
                    (character.skills.athletics.max <= character.skills.athletics.total,character.skills.athletics.max <= character.skills.athletics.total+1),
                    (character.skills.barter.max <= character.skills.barter.total,character.skills.barter.max <= character.skills.barter.total+1),
                    (character.skills.big_guns.max <= character.skills.big_guns.total,character.skills.big_guns.max <= character.skills.big_guns.total+1),
                    (character.skills.energy_weapons.max <= character.skills.energy_weapons.total,character.skills.energy_weapons.max <= character.skills.energy_weapons.total+1),
                    (character.skills.explosives.max <= character.skills.explosives.total,character.skills.explosives.max <= character.skills.explosives.total+1),
                    (character.skills.lockpick.max <= character.skills.lockpick.total,character.skills.lockpick.max <= character.skills.lockpick.total+1),
                    (character.skills.medicine.max <= character.skills.medicine.total,character.skills.medicine.max <= character.skills.medicine.total+1),
                    (character.skills.melee_weapons.max <= character.skills.melee_weapons.total,character.skills.melee_weapons.max <= character.skills.melee_weapons.total+1),
                    (character.skills.pilot.max <= character.skills.pilot.total,character.skills.pilot.max <= character.skills.pilot.total+1),
                    (character.skills.repair.max <= character.skills.repair.total,character.skills.repair.max <= character.skills.repair.total+1),
                    (character.skills.science.max <= character.skills.science.total,character.skills.science.max <= character.skills.science.total+1),
                    (character.skills.small_guns.max <= character.skills.small_guns.total,character.skills.small_guns.max <= character.skills.small_guns.total+1),
                    (character.skills.sneak.max <= character.skills.sneak.total,character.skills.sneak.max <= character.skills.sneak.total+1),
                    (character.skills.speech.max <= character.skills.speech.total,character.skills.speech.max <= character.skills.speech.total+1),
                    (character.skills.survival.max <= character.skills.survival.total,character.skills.survival.max <= character.skills.survival.total+1),
                    (character.skills.throwing.max <= character.skills.throwing.total,character.skills.throwing.max <= character.skills.throwing.total+1),
                    (character.skills.unarmed.max <= character.skills.unarmed.total,character.skills.unarmed.max <= character.skills.unarmed.total+1),
                ];
                if let Some(_cb) = ui.begin_combo("##sk_a", preview_a) {
                    for i in 0..17 {
                        let (at, exceed) = at_max[i];
                        if at {
                            let _g = ui.begin_disabled(true);
                            ui.selectable_config(&format!("{} (at cap)", SPECIAL_LABELS[i])).build();
                            drop(_g);
                            continue;
                        } else if skill_b.unwrap() == i && exceed {
                            let _g = ui.begin_disabled(true);
                            ui.selectable_config(&format!("{} (would exceed cap)", SPECIAL_LABELS[i])).build();
                            drop(_g);
                            continue;
                        }
                        let sel = *skill_a == Some(i);
                        if ui.selectable_config(SPECIAL_LABELS[i]).selected(sel).build() {
                            *skill_a = Some(i);
                        }
                    }
                }
                if let Some(_cb) = ui.begin_combo("##sk_b", preview_b) {
                    for i in 0..17 {
                        let (at, exceed) = at_max[i];
                        if at {
                            let _g = ui.begin_disabled(true);
                            ui.selectable_config(&format!("{} (at cap)", SPECIAL_LABELS[i])).build();
                            drop(_g);
                            continue;
                        } else if skill_a.unwrap() == i && exceed {
                            let _g = ui.begin_disabled(true);
                            ui.selectable_config(&format!("{} (would exceed cap)", SPECIAL_LABELS[i])).build();
                            drop(_g);
                            continue;
                        }
                        let sel = *skill_b == Some(i);
                        if ui.selectable_config(SPECIAL_LABELS[i]).selected(sel).build() {
                            *skill_b = Some(i);
                        }
                    }
                }
            } else {
                ui.text("Select a Skilled skill to reduce:");
                ui.spacing();
                ui.set_next_item_width(220.0);

                let options = character.skills.zip_skilled();

                let preview = format!("{}/{}",skill_a.map(|i| SKILLS[i]).unwrap_or("-- Select --"),skill_b.map(|i| SKILLS[i]).unwrap_or("-- Select --"));

                if let Some(_cb) = ui.begin_combo("##sk_dec", preview) {
                    for i in 0..options.len() {
                        let (sk_a, sk_b) = options[i];
                        let sel = skill_a.unwrap() == sk_a && skill_b.unwrap() == sk_b;
                        if ui.selectable_config(format!("{}/{}",SKILLS[sk_a],SKILLS[sk_b])).selected(sel).build() {
                            *skill_a = Some(sk_a);
                            *skill_b = Some(sk_b);
                        }
                    }
                }
            }
        }
        PerkResolution::Tag { selected_skill } => {
            let (res,inc) = state.pending_resolution.unwrap();
            if inc {
                ui.text("Tag an additional skill:");
                ui.spacing();
                ui.set_next_item_width(220.0);

                let preview = selected_skill
                    .map(|i| SKILLS[i])
                    .unwrap_or("-- Select skill --");
                let options = character.skills.available_tags(character);
                let exceeds: [bool; 17] = [
                    character.skills.athletics.max <= character.skills.athletics.total+1,
                    character.skills.barter.max <= character.skills.barter.total+1,
                    character.skills.big_guns.max <= character.skills.big_guns.total+1,
                    character.skills.energy_weapons.max <= character.skills.energy_weapons.total+1,
                    character.skills.explosives.max <= character.skills.explosives.total+1,
                    character.skills.lockpick.max <= character.skills.lockpick.total+1,
                    character.skills.medicine.max <= character.skills.medicine.total+1,
                    character.skills.melee_weapons.max <= character.skills.melee_weapons.total+1,
                    character.skills.pilot.max <= character.skills.pilot.total+1,
                    character.skills.repair.max <= character.skills.repair.total+1,
                    character.skills.science.max <= character.skills.science.total+1,
                    character.skills.small_guns.max <= character.skills.small_guns.total+1,
                    character.skills.sneak.max <= character.skills.sneak.total+1,
                    character.skills.speech.max <= character.skills.speech.total+1,
                    character.skills.survival.max <= character.skills.survival.total+1,
                    character.skills.throwing.max <= character.skills.throwing.total+1,
                    character.skills.unarmed.max <= character.skills.unarmed.total+1,
                ];

                if let Some(_cb) = ui.begin_combo("##tag_skill", preview) {
                    for i in 0..options.len() {
                        let disabled = exceeds[options[i]];
                        if disabled {
                            let _g = ui.begin_disabled(true);
                            ui.selectable_config(&format!("{} (would exceed max ranks)", SKILLS[options[i]])).build();
                            drop(_g);
                            continue;
                        }
                        let sel = *selected_skill == Some(options[i]);
                        if ui.selectable_config(SKILLS[options[i]]).selected(sel).build() {
                            *selected_skill = Some(options[i]);
                        }
                    }
                }
            } else {
                ui.text("Select a tag to remove:");
                ui.spacing();
                ui.set_next_item_width(220.0);

                let options = character.skills.perk_tagged();

                let preview = selected_skill
                    .map(|i| SKILLS[i])
                    .unwrap_or("-- Select Tagged Skill --");

                if let Some(_cb) = ui.begin_combo("##untag_skill", preview) {
                    for i in 0..options.len() {
                        let sel = *selected_skill == Some(options[i]);
                        if ui.selectable_config(SKILLS[options[i]]).selected(sel).build() {
                            *selected_skill = Some(options[i]);
                        }
                    }
                }
            }
        }
        PerkResolution::MmCf { version } => {
            ui.text("Select a Type");
            ui.spacing();
            ui.set_next_item_width(220.0);

            let preview = version.clone().map(|s| s.to_perk_string().to_string()).unwrap_or("-- Select Type --".to_string());

            if let Some(_cb) = ui.begin_combo("##mmcf_choice", preview) {
                for (i, option) in [MmCf::MechanicalMenace, MmCf::ClassFreak].iter().enumerate() {
                    let sel = *version == Some(option.clone());
                    if ui.selectable_config(option.to_perk_string())
                        .selected(sel)
                        .build() { *version = Some(option.clone()); }
                    if sel {
                        ui.set_item_default_focus();
                    }
                }
            }
        }
    }
    ui.spacing();
    ui.separator();
    ui.spacing();

    let complete = PerkState::is_resolution_complete(popup);
    let _g = (!complete).then(|| ui.begin_disabled(true));
    if ui.button(format!("Confirm##res_confirm_{}", popup.perk_id)) {
        apply_resolution(popup, character, state);
        popup.open = false;
        return Some(true)
    }
    drop(_g);

    if !complete {
        ui.same_line();
        ui.text_disabled("Make a selection.");
    }

    None
}

fn apply_resolution(
    popup: &PerkResolutionPopup,
    character: &mut Character,
    state: &mut PerkState,
) {
    match &popup.resolution {
        PerkResolution::BwLk { version } => {
            if let Some(perk) = character.perks.iter_mut().find(|p| p.id == popup.perk_id) {perk.name = version.clone().unwrap().to_perk_string().to_string();}
        }
        PerkResolution::IntenseTraining { selected_stat } => {
            let (res,inc) = state.pending_resolution.unwrap();
            let dir = if inc { 1 } else { -1 };
            match selected_stat.unwrap() {
                0 => {character.special.strength.value += dir; character.special.strength.trained += dir},
                1 => {character.special.perception.value += dir; character.special.perception.trained += dir},
                2 => {character.special.endurance.value += dir; character.special.endurance.trained += dir},
                3 => {character.special.charisma.value += dir; character.special.charisma.trained += dir},
                4 => {character.special.intelligence.value += dir; character.special.intelligence.trained += dir},
                5 => {character.special.agility.value += dir; character.special.agility.trained += dir},
                6 => {character.special.luck.value += dir; character.special.luck.trained += dir},
                _ => {},
            }
        }
        PerkResolution::Skilled { skill_a, skill_b } => {
            let (res,inc) = state.pending_resolution.unwrap();
            if inc {
                let mut skilled_update = [0; 17];
                if skill_a == skill_b {
                    skilled_update[skill_a.unwrap()] = 2;
                } else {
                    skilled_update[skill_a.unwrap()] = 1;
                    skilled_update[skill_b.unwrap()] = 1;
                }
                character.skills.athletics.skilled.push(skilled_update[0]);
                character.skills.athletics.total += skilled_update[0];
                character.skills.barter.skilled.push(skilled_update[1]);
                character.skills.barter.total += skilled_update[1];
                character.skills.big_guns.skilled.push(skilled_update[2]);
                character.skills.big_guns.total += skilled_update[2];
                character.skills.energy_weapons.skilled.push(skilled_update[3]);
                character.skills.energy_weapons.total += skilled_update[3];
                character.skills.explosives.skilled.push(skilled_update[4]);
                character.skills.explosives.total += skilled_update[4];
                character.skills.lockpick.skilled.push(skilled_update[5]);
                character.skills.lockpick.total += skilled_update[5];
                character.skills.medicine.skilled.push(skilled_update[6]);
                character.skills.medicine.total += skilled_update[6];
                character.skills.melee_weapons.skilled.push(skilled_update[7]);
                character.skills.melee_weapons.total += skilled_update[7];
                character.skills.pilot.skilled.push(skilled_update[8]);
                character.skills.pilot.total += skilled_update[8];
                character.skills.repair.skilled.push(skilled_update[9]);
                character.skills.repair.total += skilled_update[9];
                character.skills.science.skilled.push(skilled_update[10]);
                character.skills.science.total += skilled_update[10];
                character.skills.small_guns.skilled.push(skilled_update[11]);
                character.skills.small_guns.total += skilled_update[11];
                character.skills.sneak.skilled.push(skilled_update[12]);
                character.skills.sneak.total += skilled_update[12];
                character.skills.speech.skilled.push(skilled_update[13]);
                character.skills.speech.total += skilled_update[13];
                character.skills.survival.skilled.push(skilled_update[14]);
                character.skills.survival.total += skilled_update[14];
                character.skills.throwing.skilled.push(skilled_update[15]);
                character.skills.throwing.total += skilled_update[15];
                character.skills.unarmed.skilled.push(skilled_update[16]);
                character.skills.unarmed.total += skilled_update[16];
            } else {
                if skill_a == skill_b {
                    let indices: Vec<usize> = match skill_a.unwrap() {
                        0 => character.skills.athletics.skilled.iter().enumerate().filter_map(|(i,s)| if *s == 2 {Some(i)} else {None}).collect(),
                        1 => character.skills.barter.skilled.iter().enumerate().filter_map(|(i,s)| if *s == 2 {Some(i)} else {None}).collect(),
                        2 => character.skills.big_guns.skilled.iter().enumerate().filter_map(|(i,s)| if *s == 2 {Some(i)} else {None}).collect(),
                        3 => character.skills.energy_weapons.skilled.iter().enumerate().filter_map(|(i,s)| if *s == 2 {Some(i)} else {None}).collect(),
                        4 => character.skills.explosives.skilled.iter().enumerate().filter_map(|(i,s)| if *s == 2 {Some(i)} else {None}).collect(),
                        5 => character.skills.lockpick.skilled.iter().enumerate().filter_map(|(i,s)| if *s == 2 {Some(i)} else {None}).collect(),
                        6 => character.skills.medicine.skilled.iter().enumerate().filter_map(|(i,s)| if *s == 2 {Some(i)} else {None}).collect(),
                        7 => character.skills.melee_weapons.skilled.iter().enumerate().filter_map(|(i,s)| if *s == 2 {Some(i)} else {None}).collect(),
                        8 => character.skills.pilot.skilled.iter().enumerate().filter_map(|(i,s)| if *s == 2 {Some(i)} else {None}).collect(),
                        9 => character.skills.repair.skilled.iter().enumerate().filter_map(|(i,s)| if *s == 2 {Some(i)} else {None}).collect(),
                        10 => character.skills.science.skilled.iter().enumerate().filter_map(|(i,s)| if *s == 2 {Some(i)} else {None}).collect(),
                        11 => character.skills.small_guns.skilled.iter().enumerate().filter_map(|(i,s)| if *s == 2 {Some(i)} else {None}).collect(),
                        12 => character.skills.sneak.skilled.iter().enumerate().filter_map(|(i,s)| if *s == 2 {Some(i)} else {None}).collect(),
                        13 => character.skills.speech.skilled.iter().enumerate().filter_map(|(i,s)| if *s == 2 {Some(i)} else {None}).collect(),
                        14 => character.skills.survival.skilled.iter().enumerate().filter_map(|(i,s)| if *s == 2 {Some(i)} else {None}).collect(),
                        15 => character.skills.throwing.skilled.iter().enumerate().filter_map(|(i,s)| if *s == 2 {Some(i)} else {None}).collect(),
                        16 => character.skills.unarmed.skilled.iter().enumerate().filter_map(|(i,s)| if *s == 2 {Some(i)} else {None}).collect(),
                        _ => {vec![]}
                    };
                    character.skills.athletics.skilled.remove(indices[0]);
                    character.skills.barter.skilled.remove(indices[0]);
                    character.skills.big_guns.skilled.remove(indices[0]);
                    character.skills.energy_weapons.skilled.remove(indices[0]);
                    character.skills.explosives.skilled.remove(indices[0]);
                    character.skills.lockpick.skilled.remove(indices[0]);
                    character.skills.medicine.skilled.remove(indices[0]);
                    character.skills.melee_weapons.skilled.remove(indices[0]);
                    character.skills.pilot.skilled.remove(indices[0]);
                    character.skills.repair.skilled.remove(indices[0]);
                    character.skills.science.skilled.remove(indices[0]);
                    character.skills.small_guns.skilled.remove(indices[0]);
                    character.skills.sneak.skilled.remove(indices[0]);
                    character.skills.speech.skilled.remove(indices[0]);
                    character.skills.survival.skilled.remove(indices[0]);
                    character.skills.throwing.skilled.remove(indices[0]);
                    character.skills.unarmed.skilled.remove(indices[0]);
                    character.skills.athletics.total -= 2;
                    character.skills.barter.total -= 2;
                    character.skills.big_guns.total -= 2;
                    character.skills.energy_weapons.total -= 2;
                    character.skills.explosives.total -= 2;
                    character.skills.lockpick.total -= 2;
                    character.skills.medicine.total -= 2;
                    character.skills.melee_weapons.total -= 2;
                    character.skills.pilot.total -= 2;
                    character.skills.repair.total -= 2;
                    character.skills.science.total -= 2;
                    character.skills.small_guns.total -= 2;
                    character.skills.sneak.total -= 2;
                    character.skills.speech.total -= 2;
                    character.skills.survival.total -= 2;
                    character.skills.throwing.total -= 2;
                    character.skills.unarmed.total -= 2;
                } else {
                    let mut indices_a = vec![];
                    let mut indices_b = vec![];
                    for (i, skill) in [skill_a,skill_b].iter().enumerate() {
                        let indices: Vec<usize> = match skill.unwrap() {
                            0 => character.skills.athletics.skilled.iter().enumerate().filter_map(|(i,s)| if *s == 1 {Some(i)} else {None}).collect(),
                            1 => character.skills.barter.skilled.iter().enumerate().filter_map(|(i,s)| if *s == 1 {Some(i)} else {None}).collect(),
                            2 => character.skills.big_guns.skilled.iter().enumerate().filter_map(|(i,s)| if *s == 1 {Some(i)} else {None}).collect(),
                            3 => character.skills.energy_weapons.skilled.iter().enumerate().filter_map(|(i,s)| if *s == 1 {Some(i)} else {None}).collect(),
                            4 => character.skills.explosives.skilled.iter().enumerate().filter_map(|(i,s)| if *s == 1 {Some(i)} else {None}).collect(),
                            5 => character.skills.lockpick.skilled.iter().enumerate().filter_map(|(i,s)| if *s == 1 {Some(i)} else {None}).collect(),
                            6 => character.skills.medicine.skilled.iter().enumerate().filter_map(|(i,s)| if *s == 1 {Some(i)} else {None}).collect(),
                            7 => character.skills.melee_weapons.skilled.iter().enumerate().filter_map(|(i,s)| if *s == 1 {Some(i)} else {None}).collect(),
                            8 => character.skills.pilot.skilled.iter().enumerate().filter_map(|(i,s)| if *s == 1 {Some(i)} else {None}).collect(),
                            9 => character.skills.repair.skilled.iter().enumerate().filter_map(|(i,s)| if *s == 1 {Some(i)} else {None}).collect(),
                            10 => character.skills.science.skilled.iter().enumerate().filter_map(|(i,s)| if *s == 1 {Some(i)} else {None}).collect(),
                            11 => character.skills.small_guns.skilled.iter().enumerate().filter_map(|(i,s)| if *s == 1 {Some(i)} else {None}).collect(),
                            12 => character.skills.sneak.skilled.iter().enumerate().filter_map(|(i,s)| if *s == 1 {Some(i)} else {None}).collect(),
                            13 => character.skills.speech.skilled.iter().enumerate().filter_map(|(i,s)| if *s == 1 {Some(i)} else {None}).collect(),
                            14 => character.skills.survival.skilled.iter().enumerate().filter_map(|(i,s)| if *s == 1 {Some(i)} else {None}).collect(),
                            15 => character.skills.throwing.skilled.iter().enumerate().filter_map(|(i,s)| if *s == 1 {Some(i)} else {None}).collect(),
                            16 => character.skills.unarmed.skilled.iter().enumerate().filter_map(|(i,s)| if *s == 1 {Some(i)} else {None}).collect(),
                            _ => {vec![]}
                        };
                        if i == 0 { indices_a = indices } else { indices_b = indices }
                    }
                    let mut index: usize = 0;
                    for i in 0..indices_a.len() {
                        if indices_a[i] == 1 && indices_b[i] == 1 {
                            index = i;
                            break;
                        }
                    }
                    character.skills.athletics.skilled.remove(index);
                    character.skills.barter.skilled.remove(index);
                    character.skills.big_guns.skilled.remove(index);
                    character.skills.energy_weapons.skilled.remove(index);
                    character.skills.explosives.skilled.remove(index);
                    character.skills.lockpick.skilled.remove(index);
                    character.skills.medicine.skilled.remove(index);
                    character.skills.melee_weapons.skilled.remove(index);
                    character.skills.pilot.skilled.remove(index);
                    character.skills.repair.skilled.remove(index);
                    character.skills.science.skilled.remove(index);
                    character.skills.small_guns.skilled.remove(index);
                    character.skills.sneak.skilled.remove(index);
                    character.skills.speech.skilled.remove(index);
                    character.skills.survival.skilled.remove(index);
                    character.skills.throwing.skilled.remove(index);
                    character.skills.unarmed.skilled.remove(index);
                    character.skills.athletics.total -= 1;
                    character.skills.barter.total -= 1;
                    character.skills.big_guns.total -= 1;
                    character.skills.energy_weapons.total -= 1;
                    character.skills.explosives.total -= 1;
                    character.skills.lockpick.total -= 1;
                    character.skills.medicine.total -= 1;
                    character.skills.melee_weapons.total -= 1;
                    character.skills.pilot.total -= 1;
                    character.skills.repair.total -= 1;
                    character.skills.science.total -= 1;
                    character.skills.small_guns.total -= 1;
                    character.skills.sneak.total -= 1;
                    character.skills.speech.total -= 1;
                    character.skills.survival.total -= 1;
                    character.skills.throwing.total -= 1;
                    character.skills.unarmed.total -= 1;
                }
            }
        }
        PerkResolution::Tag { selected_skill } => {
            let (res,inc) = state.pending_resolution.unwrap();
            let dir = if inc { 1 } else { -1 };
            match selected_skill.unwrap() {
                0 => {character.skills.athletics.tagged == TagType::Perk; character.skills.athletics.update()},
                1 => {character.skills.barter.tagged == TagType::Perk; character.skills.barter.update()},
                2 => {character.skills.big_guns.tagged == TagType::Perk; character.skills.big_guns.update()},
                3 => {character.skills.energy_weapons.tagged == TagType::Perk; character.skills.energy_weapons.update()},
                4 => {character.skills.explosives.tagged == TagType::Perk; character.skills.explosives.update()},
                5 => {character.skills.lockpick.tagged == TagType::Perk; character.skills.lockpick.update()},
                6 => {character.skills.medicine.tagged == TagType::Perk; character.skills.medicine.update()},
                7 => {character.skills.melee_weapons.tagged == TagType::Perk; character.skills.melee_weapons.update()},
                8 => {character.skills.pilot.tagged == TagType::Perk; character.skills.pilot.update()},
                9 => {character.skills.repair.tagged == TagType::Perk; character.skills.repair.update()},
                10 => {character.skills.science.tagged == TagType::Perk; character.skills.science.update()},
                11 => {character.skills.small_guns.tagged == TagType::Perk; character.skills.small_guns.update()},
                12 => {character.skills.sneak.tagged == TagType::Perk; character.skills.sneak.update()},
                13 => {character.skills.speech.tagged == TagType::Perk; character.skills.speech.update()},
                14 => {character.skills.survival.tagged == TagType::Perk; character.skills.survival.update()},
                15 => {character.skills.throwing.tagged == TagType::Perk; character.skills.throwing.update()},
                16 => {character.skills.unarmed.tagged == TagType::Perk; character.skills.unarmed.update()},
                _ => {},
            }
        }
        PerkResolution::MmCf { version } => {
            if let Some(perk) = character.perks.iter_mut().find(|p| p.id == popup.perk_id) {perk.name = version.clone().unwrap().to_perk_string().to_string();}
        }
    }
}