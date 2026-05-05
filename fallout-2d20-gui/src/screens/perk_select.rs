use imgui::Ui;
use sdl2::video::Window;
use serde_json;
use fancy_regex::Regex;
use crate::db::Db;
use crate::AppScreen;
use crate::screens::skill_assignment::SKILLS;
use crate::theme::{render_text_wrapped, render_window};
use crate::screens::special_assignment::{SPECIAL_LABELS};
use crate::character::{Character, CompanionType, TagType, Perk};
//use crate::log_on_change;

#[derive(Debug)]
pub struct PerkState {
    pub perks: Vec<PerkRow>,
    pub taken_count: i32,
    pub perk_lim: i32,
    pub show_eligible_only: bool,
    pub show_taken: bool,
    pub show_taken_only: bool,
    pub filters: [bool; 8],
    pub pending_resolution: Option<(i32, bool, String)>,
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
            show_taken: true,
            show_taken_only: false,
            filters: [true; 8],
            pending_resolution: None,
        }
    }
    pub fn is_complete(&self) -> bool {
        self.perk_lim == self.taken_count
    }
    pub fn update(&mut self, character: &mut Character) {
        self.taken_count = character.perks.iter().map(|p| p.ranks).sum();
        self.perk_lim = character.level + if character.has_trait(10) { 1 } else { 0 };
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
                let special = character.special.special_block();
                let meets = match stat {
                    "strength"     => { special[0].value >= val }
                    "perception"   => { special[1].value >= val }
                    "endurance"    => { special[2].value >= val }
                    "charisma"     => { special[3].value >= val }
                    "intelligence" => { special[4].value >= val }
                    "agility"      => { special[5].value >= val }
                    "luck"         => { special[6].value >= val }
                    _ => { true }
                };
                if !meets { return false }
            }
            //can't have read a book at character creation
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
    pub fn begin_resolve(&self, perk: &PerkRow, add: bool, name: String) -> Option<PerkResolutionPopup> {
        let resolution = match perk.id {
            12 => Some(PerkResolution::BwLk { version: None }),
            45 => Some(PerkResolution::IntenseTraining { selected_stat: None }),
            83 => Some(PerkResolution::Skilled { skill_a: None, skill_b: None }),
            92 => Some(PerkResolution::Tag { selected_skill: None }),
            110 => Some(PerkResolution::MmCf { version: None }),
            _ => None,
        };
        if name == "".to_string() {
            resolution.map(|r| PerkResolutionPopup { perk_id: perk.id, perk_name: perk.name.clone(), resolution: r, perk_add: add, open: true })
        } else {
            resolution.map(|r| PerkResolutionPopup { perk_id: perk.id, perk_name: name, resolution: r, perk_add: add, open: true })
        }
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
    pub description: Vec<String>,
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
    //at one point, i had the regex outside of this function, and holy shit did it just nuke performance. we only retrieve perks once, so we don't need to do this every frame lol 
    //finds everything between each #: when multiple ranks 
    let desc_reg_pattern = Regex::new(r"\d:\s+(.+?)(?=\s+\d:|$)").unwrap();
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
            //fancy-regex uses more error handling so this gets complicated
            let desc_vec: Vec<String> = desc_reg_pattern.captures_iter(&r.description.clone().unwrap_or_default()).filter_map(|res| { match res {
                Ok(caps) => {
                    caps.get(1).map(|m| m.as_str().trim().to_string())
                }
                _ => None,
            }}).collect();
            PerkRow {
                id: r.id as i32,
                name: r.name.unwrap_or_default(),
                sourcebook: r.sourcebook.unwrap_or_default(),
                description: if desc_vec.len() > 0 {desc_vec} else {vec![r.description.unwrap_or_default()]},
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
    _screen: &mut AppScreen,
    _db: &Db,
    character: &mut Character,
    resolving: bool,
) -> f32 {
    let Some((w, h, _token)) = render_window(ui, window, "##perk_select", "Perk Select")
        else { return 0.0 };

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
    ui.checkbox("Show taken##tp", &mut state.show_taken);
    ui.same_line();
    ui.checkbox("Show taken only##to", &mut state.show_taken_only);
    if state.show_taken_only { state.show_taken = true }
    ui.same_line();
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
    let col_reqs = 240.0_f32;
    let col_ranks = 540.0_f32;
    let col_btns = 620.0_f32;

    //filtering perks
    let filtered: Vec<usize> = (0..state.perks.len())
        .filter(|&i| {
            let perk = &state.perks[i];
            state.perk_passes_filter(perk) && (!state.show_eligible_only || state.is_eligible(perk, &character) || (state.show_taken && state.is_taken(perk, character))) && (!state.show_taken_only || (state.show_taken_only && state.is_taken(perk, character)))
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
        let adrenaline_rush = id == 3 && character.special.strength.value >= 10;

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
        let warn_color = [0.35, 0.20, 0.10, 0.3_f32];

        let tint = if at_cap { cap_color } else if adrenaline_rush { warn_color } else if eligible && taken > 0 { rank_color } else if eligible { avail_color } else { unav_color };
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
            let _g = (!eligible || available || adrenaline_rush).then(|| ui.begin_disabled(true));
            if ui.button(format!("Take##take_{}", id)) {
                let cperk = Perk {
                    id,
                    name,
                    //will want to split out the different ranks at some point
                    desc: desc.clone(),
                    ranks: 1,
                };
                character.perks.push(cperk);
                state.pending_resolution = Some((id, true, perk.name.clone()));
                state.update(character);
            }
            drop(_g);
            ui.same_line();
            let _g2 = true.then(|| ui.begin_disabled(true));
            ui.button(format!("Drop##drop_{}", id));
            drop(_g2);
            if adrenaline_rush {
                ui.same_line();
                ui.set_cursor_pos([ui.cursor_pos()[0], ui.cursor_pos()[1] - 7.0]);
                ui.text_wrapped("This perk will have no effect...");
            }
        } else if at_cap {
            let _g = true.then(|| ui.begin_disabled(true));
            ui.button(format!("Rank+##rankp_{}", id));
            drop(_g);
            ui.same_line();
            if ui.button(format!("Drop##drop_{}", id)) {
                let cperk_len = character.perks.len();
                for i in 0..cperk_len {
                    if character.perks[i].id == id {
                        state.pending_resolution = Some((id, false, character.perks[i].name.clone()));
                        if character.perks[i].ranks > 1 {
                            character.perks[i].ranks -= 1;
                        } else {
                            character.perks.remove(i);
                        }
                        state.update(character);
                    }
                }
            }
        } else {
            let _g = (!eligible || available).then(|| ui.begin_disabled(true));
            if ui.button(format!("Rank+##rankp_{}", id)) {
                let cperk_len = character.perks.len();
                for i in 0..cperk_len {
                    if character.perks[i].id == id {
                        character.perks[i].ranks += 1;
                        state.update(character);
                        state.pending_resolution = Some((id, true, character.perks[i].name.clone()));
                    }
                }
            }
            drop(_g);
            ui.same_line();
            if ui.button(format!("Drop##drop_{}", id)) {
                let cperk_len = character.perks.len();
                for i in 0..cperk_len {
                    if character.perks[i].id == id {
                        state.pending_resolution = Some((id, false, character.perks[i].name.clone()));
                        if character.perks[i].ranks > 1 {
                            character.perks[i].ranks -= 1;
                        } else {
                            character.perks.remove(i);
                        }
                        state.update(character);
                    }
                }
            }
        }
        //description
        let y = ui.cursor_pos()[1];
        ui.set_cursor_pos([col_name + 8.0, y]);
        let mut text_vec = vec![];
        if desc.clone().len() > 1 {
            for (i, rank) in desc.iter().enumerate() {
                text_vec.push(format!("{}: {}", i + 1, *rank));
            }
        } else {
            text_vec.push(desc[0].clone());
        }
        let render_desc = text_vec.join(" ");
        render_text_wrapped(true, false, ui, &render_desc, col_name + 8.0, w - 24.0);
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
            let inc = popup.perk_add;
            if inc {
                ui.text("Select a \"Preference\"");
                ui.spacing();
                ui.set_next_item_width(220.0);

                let preview = version.clone().map(|s| s.to_perk_string().to_string()).unwrap_or("-- Select Preference --".to_string());

                if let Some(_cb) = ui.begin_combo("##bwlk_choice", preview) {
                    for option in [BwLk::BlackWidow,BwLk::LadyKiller].iter() {
                        let sel = *version == Some(option.clone());
                        if ui.selectable_config(option.to_perk_string())
                            .selected(sel)
                            .build() { version.replace(option.clone()); }
                        if sel {
                            ui.set_item_default_focus();
                        }
                    }
                }
            } else {
                ui.text(format!("Removing {}", popup.perk_name));
                version.replace(BwLk::BlackWidow);
            }
        }
        PerkResolution::IntenseTraining { selected_stat } => {
            //log_on_change!(state);
            let inc = popup.perk_add;
            if inc {
                ui.text("Increase one SPECIAL by 1:");
                ui.spacing();
                ui.set_next_item_width(220.0);

                let preview = selected_stat
                    .map(|i| SPECIAL_LABELS[i])
                    .unwrap_or("-- Select SPECIAL --");
                let mut at_max = [false; 7];
                let special = character.special.special_block();
                for i in 0..7 {
                    at_max[i] = special[i].max <= special[i].value;
                }

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
                            selected_stat.replace(i);
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
                let special = character.special.special_block();

                let mut options: Vec<(usize,i32)> = vec![];
                for (i, stat) in special.iter().enumerate() {
                    if stat.trained > 0 {
                        options.push((i,stat.trained))
                    }
                }

                let preview = selected_stat
                    .map(|i| SPECIAL_LABELS[i])
                    .unwrap_or("-- Select Trained SPECIAL --");

                if let Some(_cb) = ui.begin_combo("##it_dec", preview) {
                    for i in 0..options.len() {
                        let (index, number) = options[i];
                        let sel = *selected_stat == Some(index);
                        if ui.selectable_config(format!("{} ({})", SPECIAL_LABELS[index], number)).selected(sel).build() {
                            selected_stat.replace(index);
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
            let inc = popup.perk_add;
            if inc {
                ui.text("Select two skills to increase:");
                ui.text_disabled("The same skill can be selected twice");
                ui.spacing();
                ui.text("Skill 1:");
                ui.same_line();
                ui.set_next_item_width(200.0);
                let preview_a = skill_a.map(|i| SKILLS[i]).unwrap_or("-- Select --");
                let preview_b = skill_b.map(|i| SKILLS[i]).unwrap_or("-- Select --");
                let mut at_max = [(false, false); 17];
                for (i, skill) in character.skills.skill_block().iter().enumerate() {
                    at_max[i] = (skill.max <= skill.total, skill.max <= skill.total+1)
                }
                if let Some(_cb) = ui.begin_combo("##sk_a", preview_a) {
                    for i in 0..17 {
                        let (at, exceed) = at_max[i];
                        if at {
                            let _g = ui.begin_disabled(true);
                            ui.selectable_config(&format!("{} (at cap)", SKILLS[i])).build();
                            drop(_g);
                            continue;
                        } else if skill_b.unwrap_or(usize::MAX) == i && exceed {
                            let _g = ui.begin_disabled(true);
                            ui.selectable_config(&format!("{} (would exceed cap)", SKILLS[i])).build();
                            drop(_g);
                            continue;
                        }
                        let sel = *skill_a == Some(i);
                        if ui.selectable_config(SKILLS[i]).selected(sel).build() {
                            skill_a.replace(i);
                        }
                    }
                }
                ui.text("Skill 2:");
                ui.same_line();
                ui.set_next_item_width(200.0);
                if let Some(_cb) = ui.begin_combo("##sk_b", preview_b) {
                    for i in 0..17 {
                        let (at, exceed) = at_max[i];
                        if at {
                            let _g = ui.begin_disabled(true);
                            ui.selectable_config(&format!("{} (at cap)", SKILLS[i])).build();
                            drop(_g);
                            continue;
                        } else if skill_a.unwrap_or(usize::MAX) == i && exceed {
                            let _g = ui.begin_disabled(true);
                            ui.selectable_config(&format!("{} (would exceed cap)", SKILLS[i])).build();
                            drop(_g);
                            continue;
                        }
                        let sel = *skill_b == Some(i);
                        if ui.selectable_config(SKILLS[i]).selected(sel).build() {
                            skill_b.replace(i);
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
                        let sel = skill_a.unwrap_or(usize::MAX) == sk_a && skill_b.unwrap_or(usize::MAX) == sk_b;
                        if ui.selectable_config(format!("{}/{}",SKILLS[sk_a],SKILLS[sk_b])).selected(sel).build() {
                            skill_a.replace(sk_a);
                            skill_b.replace(sk_b);
                        }
                    }
                }
            }
        }
        PerkResolution::Tag { selected_skill } => {
            let inc = popup.perk_add;
            if inc {
                ui.text("Tag an additional skill:");
                ui.spacing();
                ui.set_next_item_width(220.0);

                let preview = selected_skill
                    .map(|i| SKILLS[i])
                    .unwrap_or("-- Select skill --");
                let options = character.skills.available_tags(character);
                let mut exceeds = [false; 17];
                for (i, skill) in character.skills.skill_block().iter().enumerate() {
                    exceeds[i] = skill.max <= skill.total + 1;
                }

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
                            selected_skill.replace(options[i]);
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
                            selected_skill.replace(options[i]);
                        }
                    }
                }
            }
        }
        PerkResolution::MmCf { version } => {
            let inc = popup.perk_add;
            if inc {
                ui.text("Select a Type");
                ui.spacing();
                ui.set_next_item_width(220.0);

                let preview = version.clone().map(|s| s.to_perk_string().to_string()).unwrap_or("-- Select Type --".to_string());

                if let Some(_cb) = ui.begin_combo("##mmcf_choice", preview) {
                    for option in [MmCf::MechanicalMenace, MmCf::ClassFreak].iter() {
                        let sel = *version == Some(option.clone());
                        if ui.selectable_config(option.to_perk_string())
                            .selected(sel)
                            .build() { version.replace(option.clone()); }
                        if sel {
                            ui.set_item_default_focus();
                        }
                    }
                }
            } else {
                ui.text(format!("Removing {}", popup.perk_name));
                version.replace(MmCf::ClassFreak);
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
    _state: &mut PerkState,
) {
    match &popup.resolution {
        PerkResolution::BwLk { version } => {
            if popup.perk_add {
                if let Some(perk) = character.perks.iter_mut().find(|p| p.id == popup.perk_id) {perk.name = version.clone().unwrap().to_perk_string().to_string();}
            }
        }
        PerkResolution::IntenseTraining { selected_stat } => {
            let inc = popup.perk_add;
            let dir = if inc { 1 } else { -1 };
            character.special.mut_special_block()[selected_stat.unwrap()].value += dir;
            character.special.mut_special_block()[selected_stat.unwrap()].trained += dir;
        }
        PerkResolution::Skilled { skill_a, skill_b } => {
            let inc = popup.perk_add;
            if inc {
                let mut skilled_update = [0; 17];
                if skill_a == skill_b {
                    skilled_update[skill_a.unwrap()] = 2;
                } else {
                    skilled_update[skill_a.unwrap()] = 1;
                    skilled_update[skill_b.unwrap()] = 1;
                }
                let skills = character.skills.mut_skill_block();
                for i in 0..17 {
                    skills[i].skilled.push(skilled_update[i]);
                    skills[i].total += skilled_update[i];
                }
            } else {
                if skill_a == skill_b {
                    let skills = character.skills.mut_skill_block();
                    let indices: Vec<usize> = skills[skill_a.unwrap()].skilled.iter().enumerate().filter_map(|(i,s)| if *s == 2 {Some(i)} else {None}).collect();
                    for i in 0..17 {
                        skills[i].skilled.remove(indices[0]);
                    }
                    skills[skill_a.unwrap()].total -= 2;
                } else {
                    let skills = character.skills.mut_skill_block();
                    let mut indices: Vec<usize> = vec![];
                    for i in 0..skills[0].skilled.len() {
                        if skills[skill_a.unwrap()].skilled[i] == 1 &&
                            skills[skill_b.unwrap()].skilled[i] == 1 {
                            indices.push(i)
                        }
                    }
                    for i in 0..17 {
                        skills[i].skilled.remove(indices[0]);
                    }
                    skills[skill_a.unwrap()].total -= 1;
                    skills[skill_b.unwrap()].total -= 1;
                }
            }
        }
        PerkResolution::Tag { selected_skill } => {
            let inc = popup.perk_add;
            let skills = character.skills.mut_skill_block();
            if inc {
                skills[selected_skill.unwrap()].tagged = TagType::Perk;
            } else {
                skills[selected_skill.unwrap()].tagged = TagType::None;
            }
            skills[selected_skill.unwrap()].update();
        }
        PerkResolution::MmCf { version } => {
            if popup.perk_add {
                if let Some(perk) = character.perks.iter_mut().find(|p| p.id == popup.perk_id) {perk.name = version.clone().unwrap().to_perk_string().to_string();}
            }
        }
    }
}