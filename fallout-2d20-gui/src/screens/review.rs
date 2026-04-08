use imgui::Ui;
use sdl2::video::Window;
use std::collections::HashMap;
use crate::db::Db;
use crate::AppScreen;
use crate::Theme;
use crate::BAR_HEIGHT;
use crate::screens::new_character::NewCharacterState;
use crate::screens::skills::SKILLS;
use crate::screens::special::SpecialState;
use crate::screens::skills::SkillsState;
use crate::screens::perks::PerksState;
use crate::screens::equipment::{EquipmentState, ResolvedWeapon};
use crate::screens::stats::ComputedStats;
use crate::screens::stats::build_melee_string;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Limb {
    // Organic / standard robot
    Head,
    Torso,   // organic
    Body,    // robot generic
    ArmLeft,
    ArmRight,
    LegLeft,
    LegRight,

    // Mr. Handy
    Optics,
    Arm1,
    Arm2,
    Arm3,
    Thruster,

    // Securitron
    Wheel,

    // Robobrain
    TrackLeft,
    TrackRight,
}

impl Limb {
    pub fn display_name(&self) -> &'static str {
        match self {
            Limb::Head      => "Head",
            Limb::Torso     => "Torso",
            Limb::Body      => "Body",
            Limb::ArmLeft   => "Left Arm",
            Limb::ArmRight  => "Right Arm",
            Limb::LegLeft   => "Left Leg",
            Limb::LegRight  => "Right Leg",
            Limb::Optics    => "Optics",
            Limb::Arm1      => "Arm 1",
            Limb::Arm2      => "Arm 2",
            Limb::Arm3      => "Arm 3",
            Limb::Thruster  => "Thruster",
            Limb::Wheel     => "Wheel",
            Limb::TrackLeft  => "Left Track",
            Limb::TrackRight => "Right Track",
        }
    }
}

pub const TRAIT_MR_HANDY:    i64 = 4;
pub const TRAIT_ROBOBRAIN:   i64 = 19;
pub const TRAIT_SECURITRON:  i64 = 20;

/// Returns the ordered list of valid limbs for this character.
/// `is_robot` — true if the Robot trait is active.
/// `trait_ids` — all currently active trait IDs.
pub fn resolve_limbs(is_robot: bool, trait_ids: &[i64]) -> Vec<Limb> {
    if !is_robot {
        return vec![
            Limb::Head,
            Limb::Torso,
            Limb::ArmLeft,
            Limb::ArmRight,
            Limb::LegLeft,
            Limb::LegRight,
        ];
    }

    if trait_ids.contains(&TRAIT_MR_HANDY) {
        return vec![
            Limb::Optics,
            Limb::Body,
            Limb::Arm1,
            Limb::Arm2,
            Limb::Arm3,
            Limb::Thruster,
        ];
    }

    if trait_ids.contains(&TRAIT_SECURITRON) {
        return vec![
            Limb::Head,
            Limb::Body,
            Limb::ArmLeft,
            Limb::ArmRight,
            Limb::Wheel,
        ];
    }

    if trait_ids.contains(&TRAIT_ROBOBRAIN) {
        return vec![
            Limb::Head,
            Limb::Body,
            Limb::ArmLeft,
            Limb::ArmRight,
            Limb::TrackLeft,
            Limb::TrackRight,
        ];
    }

    // Generic robot (e.g. Protectron, Assaultron, etc.)
    vec![
        Limb::Head,
        Limb::Body,
        Limb::ArmLeft,
        Limb::ArmRight,
        Limb::LegLeft,
        Limb::LegRight,
    ]
}

pub const APPAREL_TYPE_CLOTHING: i64 = 1;
pub const APPAREL_TYPE_OUTFIT: i64 = 2;
pub const APPAREL_TYPE_HEADGEAR: i64 = 3;
pub const APPAREL_TYPE_ARMOR: i64 = 4;
pub const APPAREL_TYPE_POWERARMOR: i64 = 5;
pub const APPAREL_TYPE_ROBOTARMOR: i64 = 6;

#[derive(Debug, Clone)]
pub struct LimbDr {
    pub limb: Limb,
    pub dr: i32,
}

#[derive(Debug, Clone)]
pub struct ResolvedApparel {
    pub id: i64,
    pub name: String,
    pub apparel_type: i64,
    pub phys_dr: i32,
    pub enrg_dr: i32,
    pub rads_dr: i32,
    pub covered_location_ids: Vec<i64>,  // from apparel_covers
    pub wgt: i32,
}

pub struct EquippedApparel {
    pub id: i64,
    pub name: String,
    pub covered_location_ids: Vec<i64>,
}

pub struct DamageResistanceState {
    pub limbs: Vec<LimbDr>,
}

#[derive(Debug, Clone, Default)]
pub struct Dr {
    pub phys: i32,
    pub enrg: i32,
    pub rads: i32,
}

impl Dr {
    /// Per-limb maximum across two DR sources
    pub fn layer_max(&self, other: &Dr) -> Dr {
        Dr {
            phys: self.phys.max(other.phys),
            enrg: self.enrg.max(other.enrg),
            rads: self.rads.max(other.rads),
        }
    }
}

/// Maps a body_locations.id to the Limb variant it corresponds to
/// for this character's limb set.
pub fn location_id_to_limb(location_id: i64, limbs: &[Limb]) -> Option<Limb> {
    // Canonical location IDs (match your body_locations table)
    let candidates: &[Limb] = match location_id {
        1  => &[Limb::Head],
        2  => &[Limb::ArmLeft],
        3  => &[Limb::ArmRight],
        4  => &[Limb::Torso, Limb::Body],
        5  => &[Limb::LegLeft, Limb::TrackLeft],
        6  => &[Limb::LegRight, Limb::TrackRight, Limb::Wheel],
        7  => &[Limb::Optics],
        8  => &[Limb::Arm1],
        9  => &[Limb::Arm2],
        10 => &[Limb::Arm3],
        11 => &[Limb::Body],
        12 => &[Limb::Thruster],
        13 => &[Limb::Wheel],
        _  => &[],
    };
    // Only return a candidate that this character actually has
    candidates.iter().find(|l| limbs.contains(l)).cloned()
}

pub fn limb_to_location_id(limb: &Limb) -> i64 {
    let candidate: i64 = match &limb {
        Limb::Head  => 1, 
        Limb::ArmLeft  => 2, 
        Limb::ArmRight  => 3, 
        Limb::Torso  => 4,
        Limb::Body  => 4, 
        Limb::LegLeft  => 5,
        Limb::TrackLeft  => 5, 
        Limb::LegRight  => 6,
        Limb::TrackRight  => 6,
        Limb::Wheel  => 6, 
        Limb::Optics  => 7, 
        Limb::Arm1  => 8, 
        Limb::Arm2  => 9, 
        Limb::Arm3 => 10, 
        Limb::Body => 11, 
        Limb::Thruster => 12, 
        Limb::Wheel => 13, 
        _  => 0,
    };
    candidate
}

impl DamageResistanceState {
    pub fn new(is_robot: bool, trait_ids: &[i64]) -> Self {
        Self {
            limbs: resolve_limbs(is_robot, trait_ids)
                .into_iter()
                .map(|limb| LimbDr { limb, dr: 0 })
                .collect(),
        }
    }

    /// Call when traits change (robot type swap, etc.)
    pub fn rebuild(&mut self, is_robot: bool, trait_ids: &[i64]) {
        *self = Self::new(is_robot, trait_ids);
    }
}

/// Returns a map of Limb → Dr after applying all apparel rules.
pub fn resolve_apparel_dr(
    pieces: &[ResolvedApparel],   // all selected apparel for this character
    limbs: &[Limb],
    is_robot: bool,
) -> (HashMap<Limb, Dr>, Vec<EquippedApparel>) {
    let mut dr_map: HashMap<Limb, Dr> = limbs.iter()
        .map(|l| (l.clone(), Dr::default()))
        .collect();
    let mut equipped_apparel:Vec<EquippedApparel> = vec![];

    if is_robot {
        // Robots only benefit from hats, and hats give no DR
        return (dr_map, vec![]);
    }

    // ── Determine what layer(s) to equip ─────────────────────────────────────

    let has_outfit   = pieces.iter().any(|p| p.apparel_type == APPAREL_TYPE_OUTFIT);
    let has_clothing = pieces.iter().any(|p| p.apparel_type == APPAREL_TYPE_CLOTHING);

    // Base layer: outfit wins over clothing; neither if outfit present
    let base_layer: Option<&ResolvedApparel> = if has_outfit {
        pieces.iter().find(|p| p.apparel_type == APPAREL_TYPE_OUTFIT)
    } else if has_clothing {
        pieces.iter().find(|p| p.apparel_type == APPAREL_TYPE_CLOTHING)
    } else {
        None
    };

    // Armor layer: only if no outfit
    let armor_pieces: Vec<&ResolvedApparel> = if !has_outfit {
        pieces.iter()
            .filter(|p| p.apparel_type == APPAREL_TYPE_ARMOR)
            .collect()
    } else {
        vec![]
    };

    // Headgear: apply if head is not already covered
    let head_covered_by_base = base_layer
        .map(|b| covers_limb(b, Limb::Head, limbs))
        .unwrap_or(false);
    let head_covered_by_armor = armor_pieces.iter()
        .any(|a| covers_limb(a, Limb::Head, limbs));

    let headgear: Option<&ResolvedApparel> = if !head_covered_by_base && !head_covered_by_armor {
        pieces.iter().find(|p| p.apparel_type == APPAREL_TYPE_HEADGEAR)
    } else {
        None
    };

    // ── Apply base layer ──────────────────────────────────────────────────────

    if let Some(base) = base_layer {
        apply_piece_to_map(base, limbs, &mut dr_map);
        let base_item = EquippedApparel {
            id: base.id,
            name: base.name.clone(),
            covered_location_ids: base.covered_location_ids.clone(),
        };
        equipped_apparel.push(base_item);
    }

    // ── Apply armor per limb (best physical DR wins ties broken by energy, then rads) ──

    // Group armor candidates by limb
    let mut limb_armor_candidates: HashMap<Limb, Vec<&ResolvedApparel>> = HashMap::new();
    for piece in &armor_pieces {
        for loc_id in &piece.covered_location_ids {
            if let Some(limb) = location_id_to_limb(*loc_id, limbs) {
                limb_armor_candidates.entry(limb).or_default().push(piece);
            }
        }
    }

    for (limb, candidates) in &limb_armor_candidates {
        let best = candidates.iter().copied()
            .max_by_key(|p| (p.phys_dr, p.enrg_dr, p.rads_dr))
            .unwrap(); // candidates is never empty here

        let armor_dr = Dr { phys: best.phys_dr, enrg: best.enrg_dr, rads: best.rads_dr };
        let entry = dr_map.entry(limb.clone()).or_default();
        let armor_item = EquippedApparel {
            id: best.id,
            name: best.name.clone(),
            covered_location_ids: best.covered_location_ids.clone(),
        };
        equipped_apparel.push(armor_item);

        *entry = entry.layer_max(&armor_dr);
    }

    // ── Apply headgear ────────────────────────────────────────────────────────

    if let Some(hg) = headgear {
        apply_piece_to_map(hg, limbs, &mut dr_map);
        let head_item = EquippedApparel {
            id: hg.id,
            name: hg.name.clone(),
            covered_location_ids: hg.covered_location_ids.clone(),
        };
        equipped_apparel.push(head_item);
    }

    (dr_map, equipped_apparel)
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn covers_limb(piece: &ResolvedApparel, target: Limb, limbs: &[Limb]) -> bool {
    piece.covered_location_ids.iter()
        .any(|&loc| location_id_to_limb(loc, limbs) == Some(target.clone()))
}

fn apply_piece_to_map(
    piece: &ResolvedApparel,
    limbs: &[Limb],
    dr_map: &mut HashMap<Limb, Dr>,
) {
    let piece_dr = Dr { phys: piece.phys_dr, enrg: piece.enrg_dr, rads: piece.rads_dr };
    for &loc_id in &piece.covered_location_ids {
        if let Some(limb) = location_id_to_limb(loc_id, limbs) {
            let entry = dr_map.entry(limb).or_default();
            *entry = entry.layer_max(&piece_dr);
        }
    }
}

pub fn load_resolved_apparel(db: &Db, apparel_ids: &[i64]) -> Vec<ResolvedApparel> {
    let id_json = serde_json::to_string(apparel_ids).unwrap_or_default();

    // Query 1 — base apparel data
    let apparel_rows = db.block_on(async {
        sqlx::query!(r#"
            SELECT id, name, type AS apparel_type,
                phys_dr, enrg_dr, rads_dr, wgt
            FROM apparel
            WHERE id IN (SELECT value FROM json_each(?1))
        "#, id_json)
        .fetch_all(&db.pool).await
    }).unwrap_or_default();

    // Query 2 — coverage
    let cover_rows = db.block_on(async {
        sqlx::query!(r#"
            SELECT apparel_id, location_id
            FROM apparel_covers
            WHERE apparel_id IN (SELECT value FROM json_each(?1))
        "#, id_json)
        .fetch_all(&db.pool).await
    }).unwrap_or_default();

    // Join in Rust
    let mut cover_map: HashMap<i64, Vec<i64>> = HashMap::new();
    for r in cover_rows {
        cover_map
            .entry(r.apparel_id.unwrap_or_default())
            .or_default()
            .push(r.location_id.unwrap_or_default());
    }

    apparel_rows.into_iter().map(|r| {
        let id = r.id.unwrap_or_default();
        ResolvedApparel {
            id,
            name: r.name.unwrap_or_default(),
            apparel_type: r.apparel_type.unwrap_or_default(),
            phys_dr: r.phys_dr.unwrap_or_default() as i32,
            enrg_dr: r.enrg_dr.unwrap_or_default() as i32,
            rads_dr: r.rads_dr.unwrap_or_default() as i32,
            covered_location_ids: cover_map.remove(&id).unwrap_or_default(),
            wgt: r.wgt.unwrap_or_default() as i32,
        }
    }).collect()
}

/// Each inner Vec is one row of the display grid.
pub fn limb_layout(limbs: &[Limb]) -> Vec<Vec<Limb>> {
    // Detect robot type by which limbs are present
    let has = |l: &Limb| limbs.contains(l);

    if has(&Limb::Optics) {
        // Mr. Handy
        vec![
            vec![Limb::Optics],
            vec![Limb::Body],
            vec![Limb::Arm1, Limb::Arm2, Limb::Arm3],
            vec![Limb::Thruster],
        ]
    } else if has(&Limb::Wheel) {
        // Securitron
        vec![
            vec![Limb::ArmLeft, Limb::Head, Limb::ArmRight],
            vec![Limb::Body],
            vec![Limb::Wheel],
        ]
    } else {
        // All others (organic, robobrain, generic robot, securitron-fallback)
        let torso_or_body = if has(&Limb::Torso) { Limb::Torso } else { Limb::Body };
        let leg_l = if has(&Limb::LegLeft)   { Limb::LegLeft }   else { Limb::TrackLeft };
        let leg_r = if has(&Limb::LegRight)  { Limb::LegRight }  else { Limb::TrackRight };

        vec![
            vec![Limb::Head],
            vec![Limb::ArmLeft, Limb::ArmRight],
            vec![torso_or_body],
            vec![leg_l, leg_r],
        ]
    }
    .into_iter()
    // Drop any row whose limbs aren't actually present on this character
    .map(|row| row.into_iter().filter(|l| has(l)).collect::<Vec<_>>())
    .filter(|row| !row.is_empty())
    .collect()
}

pub fn render_dr_block(
    ui: &Ui,
    limb: &Limb,
    dr: &Dr,
    base_health: i32,
    equipped_item: &str,
) {
    let label   = limb.display_name();
    let block_w = 100.0;
    let pad_w = 10.0;
    let cell_w = ( block_w / 2.0 ) - pad_w;
    let pad = 4.0;

    // Record top-left before the group
    let start = ui.cursor_screen_pos();

    ui.group(|| {
        // Header centered
        let head_w = ui.calc_text_size(label)[0];
        let head_x = start[0] + ((block_w - head_w) / 2.0) - pad_w;
        ui.set_cursor_screen_pos([head_x, ui.cursor_screen_pos()[1]]);
        ui.text_colored(ui.style_color(imgui::StyleColor::DragDropTarget), label);

        //ui.separator();

        // Row 1 — Physical | Energy
        render_dr_cell(ui, "PH", dr.phys, cell_w, pad_w);
        ui.same_line_with_spacing(0.0, pad_w);
        render_dr_cell(ui, "EN", dr.enrg, cell_w, pad_w);

        // Row 2 — Radiation | Health
        render_dr_cell(ui, "RD", dr.rads, cell_w, pad_w);
        ui.same_line_with_spacing(0.0, pad_w);
        render_dr_cell(ui, "HP", base_health, cell_w, pad_w);

        // equipped clothing
        let equip_w = ui.calc_text_size(equipped_item)[0];
        let equip_x = start[0] + ((block_w - equip_w) / 2.0) - pad_w;
        //let old_cursor_pos = ui.cursor_pos();
        ui.set_cursor_screen_pos([equip_x, ui.cursor_screen_pos()[1] + 2.0]);
        ui.text_disabled(equipped_item);
    });

    // Draw border around the completed group
    let end = ui.item_rect_max();
    let draw = ui.get_window_draw_list();
    let top_left_x = start[0] - pad;
    let top_left_y = start[1] - pad;
    let bot_right_x = end[0] + pad;
    let bot_right_y = end[1] - 18.0 + pad;
    draw.add_rect(
        [top_left_x, top_left_y],
        [bot_right_x, bot_right_y],
        ui.style_color(imgui::StyleColor::DragDropTarget)
    )
    .rounding(3.0)
    .build();
}

fn render_dr_cell(ui: &Ui, label: &str, value: i32, w: f32, _pad: f32) {
    ui.group(|| {
        let cell_w = w - 4.0;
        // Label dimmed, left side
        ui.text_disabled(label);
        ui.same_line_with_spacing(0.0, 4.0);
        // Value right-aligned within remaining space
        let val_str = value.to_string();
        let val_w   = ui.calc_text_size(&val_str)[0];
        let label_w = ui.calc_text_size(label)[0];
        let spacer  = (cell_w - label_w - val_w - 4.0).max(0.0);
        ui.set_cursor_pos([
            ui.cursor_pos()[0] + spacer,
            ui.cursor_pos()[1],
        ]);
        ui.text(&val_str);
    });
}

pub fn render_dr_table(
    ui: &Ui,
    limbs: &[Limb],
    dr_map: &HashMap<Limb, Dr>,
    equipped_apparel: Vec<EquippedApparel>,
) {
    let layout = limb_layout(limbs);
    let block_w = 82.0;
    let gap = 8.0;
    let h_gap = 24.0;  // ← increased from 8.0
    let v_gap = 6.0;
    
    ui.set_cursor_pos([
        ui.cursor_pos()[0],
        ui.cursor_pos()[1] + 6.0,
    ]);


    for row in &layout {
        let row_w = row.len() as f32 * block_w
            + (row.len().saturating_sub(1)) as f32 * h_gap;

        // Center within the column, not the whole window
        let avail      = ui.content_region_avail()[0];
        let col_origin = ui.cursor_pos()[0];
        let offset     = ((avail - row_w) / 2.0).max(0.0);

        ui.set_cursor_pos([col_origin + offset, ui.cursor_pos()[1]]);

        for (i, limb) in row.iter().enumerate() {
            if i > 0 {
                ui.same_line_with_spacing(0.0, h_gap);
            }
            let dr = dr_map.get(limb).cloned().unwrap_or_default();
            let current_limb = limb_to_location_id(limb);
            let equipped_item: &str = match equipped_apparel
                .iter()
                .find(
                    |x| x.covered_location_ids.contains(&current_limb)
                ) {
                    Some(item) => &item.name.clone(),
                    None => "",
            };
            render_dr_block(ui, limb, &dr, 0, equipped_item);
        }

        ui.set_cursor_pos([
            ui.cursor_pos()[0],
            ui.cursor_pos()[1] + v_gap,
        ])
        //ui.spacing();
        //ui.spacing();  // ← extra spacing between rows
    }
}

struct InventoryLine {
    name: String,
    weight: i32,
}

// ── XP helpers ────────────────────────────────────────────────────────────────

fn xp_for_level(level: i32) -> i32 {
    level * (level - 1) * 50
}

fn xp_to_next(level: i32) -> i32 {
    xp_for_level(level + 1) - xp_for_level(level)
}

pub fn render_review(
    ui: &Ui,
    window: &Window,
    nc: &NewCharacterState,
    special: &SpecialState,
    skills: &SkillsState,
    perks: &PerksState,
    stats: &ComputedStats,
    equipment: &EquipmentState,
    screen: &mut AppScreen,
    theme: &Theme,
    db: &crate::db::Db,
) {
    let (win_w, win_h) = window.size();
    let bar_h = BAR_HEIGHT;
    let content_h = win_h as f32 - bar_h;
    let w = (win_w as f32 * 0.85).min(1100.0);
    let h = content_h * 0.92;

    let Some(_win) = ui.window("##review")
        .title_bar(false)
        .resizable(false)
        .movable(false)
        .size([w, h], imgui::Condition::Always)
        .position(
            [(win_w as f32 - w) * 0.5, bar_h + (content_h - h) * 0.5],
            imgui::Condition::Always,
        )
        .begin()
    else { return; };

    ui.text("CHARACTER REVIEW");
    ui.separator();
    ui.spacing();

    // Scrollable content
    let footer_h = 48.0;
    let Some(_scroll) = ui.child_window("##review_scroll")
        .size([w - 16.0, h - footer_h - 16.0])
        .begin()
    else { return; };

    let col_w = (w - 48.0) / 2.0;

    // ── Identity block ────────────────────────────────────────────────────────
    section_header(ui, theme, "IDENTITY");

    let id_col_w = w / 2.0 - 24.0;

    let origin_name = nc.origins
        .get(nc.selected_origin_idx)
        .map(|o| o.name.as_str())
        .unwrap_or("—");

    let background_name = equipment.current_bg
        .as_ref()
        .map(|b| b.name.as_str())
        .unwrap_or("—");

    let current_xp  = xp_for_level(nc.level);
    let next_xp     = xp_for_level(nc.level + 1);
    let to_next     = xp_to_next(nc.level);

    // Two-column layout for identity and stats
    ui.columns(2, "##id_cols", false);
    ui.set_column_width(0, col_w);
    ui.set_column_width(1, col_w);

    kv(ui, theme, "Name",       &nc.name);
    kv(ui, theme, "Level",      &nc.level.to_string());
    kv(ui, theme, "XP",         &format!("{} / {} ({} to next)", current_xp, next_xp, to_next));
    kv(ui, theme, "Origin",     origin_name);
    kv(ui, theme, "Background", background_name);

    ui.next_column();

    let melee_str = build_melee_string(stats);

    kv(ui, theme, "Poison DR",       &format!("{}", &stats.poison_dr));
    kv(ui, theme, "Defense",       &format!("{}", &stats.defense));
    kv(ui, theme, "Initiative",       &format!("{}", &stats.initiative));
    kv(ui, theme, "HP",       &format!("{} / {}", &stats.max_hp, &stats.max_hp));
    kv(ui, theme, "Melee",       &melee_str);

    ui.columns(1, "##id_cols_end", false);
    ui.spacing();

    // ── SPECIAL ───────────────────────────────────────────────────────────────
    section_header(ui, theme, "SPECIAL");

    const STAT_LABELS: [&str; 7] = [
        "Strength", "Perception", "Endurance",
        "Charisma", "Intelligence", "Agility", "Luck",
    ];

    ui.columns(7, "##special_cols", false);
    let scol_w = (w - 48.0) / 7.0;
    for i in 0..7 { ui.set_column_width(i, scol_w); }

    // Headers
    for label in &STAT_LABELS {
        ui.text_colored(theme.text_dim, &label[..1]); // S P E C I A L initials
        ui.next_column();
    }
    // Values
    for i in 0..7 {
        let val = special.display_value(i);
        let base = special.base_values()[i];
        let modifier = special.modifier(i);
        if modifier > 0 {
            ui.text_colored(theme.text_desc, &format!("{}", val));
        } else {
            ui.text(&format!("{}", val));
        }
        ui.next_column();
    }
    ui.columns(1, "##special_cols_end", false);

    // ── Luck Points ───────────────────────────────────────────────────────────────
    /*
    let base_luck_points = special.display_value(6); // Luck stat index
    let gifted_penalty = if nc.selected_traits.iter().enumerate()
        .any(|(i, &sel)| sel && nc.traits.get(i).map(|t| t.id == 7).unwrap_or(false))
    { 7 } else { 0 };
    let max_luck_points = (base_luck_points - gifted_penalty).max(0);
    */

    ui.text_colored(theme.text_dim, format!(
        "Luck Points: {}/{}",
        stats.max_luck_pts, stats.max_luck_pts
    ));
    ui.spacing();

    // ── Skills + DR side by side ──────────────────────────────────────────────────
    section_header(ui, theme, "SKILLS & DAMAGE RESISTANCE");

    let skill_col_w = 260.0;
    let dr_col_w    = w - 48.0 - skill_col_w;

    ui.columns(2, "##skills_dr_cols", false);
    ui.set_column_width(0, skill_col_w);
    ui.set_column_width(1, dr_col_w);

    // ── Left: Skills ─────────────────────────────────────────────────────────────
    let skill_entries: Vec<(i32, bool)> = skills.skills.iter()
        .map(|s| (s.total(), s.tagged))
        .collect();

    for (idx, (total, tagged)) in skill_entries.iter().enumerate() {
        let label = if *tagged {
            format!("* {:.<22} {}", SKILLS[idx], total)
        } else {
            format!("  {:.<22} {}", SKILLS[idx], total)
        };
        if *tagged {
            ui.text_colored(theme.text_desc, &label);
        } else {
            ui.text(&label);
        }
    }

    ui.next_column();

    // ── Right: DR blocks ──────────────────────────────────────────────────────────
    let trait_ids: Vec<i64> = nc.traits.iter()
        .enumerate()
        .filter(|(i, _)| nc.selected_traits.get(*i).copied().unwrap_or(false))
        .filter_map(|(_, t)| Some(t.id))
        .collect();

    let is_robot = trait_ids.iter().any(|&id| {
        id == TRAIT_MR_HANDY || id == TRAIT_ROBOBRAIN || id == TRAIT_SECURITRON
        // expand with your generic Robot trait ID if you have one
    });

    let limbs = resolve_limbs(is_robot, &trait_ids);

    // Collect selected apparel IDs from equipment state
    let selected_apparel_ids: Vec<i64> = if let Some(bg) = &equipment.current_bg {
        bg.apparel_slots.iter()
            .zip(equipment.apparel_selections.iter())
            .flat_map(|(slot, sel)| {
                use crate::screens::equipment::{ApparelSlot, SlotSelection};
                match (slot, sel) {
                    (ApparelSlot::Fixed(opt), _) =>
                        vec![opt.apparel_id],
                    (ApparelSlot::Choice(opts), SlotSelection::Chosen(i)) if *i < opts.len() =>
                        vec![opts[*i].apparel_id],
                    (ApparelSlot::SingleOrDouble { single, double_choices }, SlotSelection::SingleOrDoubleChosen { take_single, double_picks }) => {
                        if *take_single {
                            vec![single.apparel_id]
                        } else {
                            double_picks.iter().enumerate()
                                .filter_map(|(di, pick)| {
                                    pick.and_then(|pi| double_choices.get(di)?.get(pi))
                                        .map(|o| o.apparel_id)
                                })
                                .collect()
                        }
                    }
                    (ApparelSlot::SingleOrPack { single, pack }, SlotSelection::SingleOrPackChosen(take_single)) => {
                        if *take_single {
                            vec![single.apparel_id]
                        } else {
                            pack.iter().map(|o| o.apparel_id).collect()
                        }
                    }
                    _ => vec![],
                }
            })
            .collect()
    } else {
        vec![]
    };

    let apparel_pieces = load_resolved_apparel(db, &selected_apparel_ids);
    let (dr_map, equipped_apparel) = resolve_apparel_dr(&apparel_pieces, &limbs, is_robot);

    render_dr_table(ui, &limbs, &dr_map, equipped_apparel);

    ui.columns(1, "##skills_dr_end", false);
    ui.spacing();

    let weapons: Vec<ResolvedWeapon> = vec![];

    // ── Weapons ───────────────────────────────────────────────────────────────────
    if equipment.current_bg.is_some() {
        section_header(ui, theme, "WEAPONS");

        let weapons = crate::screens::equipment::resolve_weapons_for_review(
            db, // you'll need to pass db into render_review
            equipment.current_bg.as_ref().unwrap(),
            &equipment.weapon_selections,
            special,
            skills,
        );
        //ui.text(format!("{}", ui.content_region_avail()[0]));

        if weapons.is_empty() {
            ui.text_colored(theme.text_dim, "  No weapons.");
        } else {
            // Column headers
            let table_w = ui.content_region_avail()[0];
            let table_min = 800.0;
            let table_max = 1080.0;
            let col_widths_min = [150.0_f32, 50.0, 30.0, 35.0, 35.0, 90.0, 45.0, 40.0, 35.0, 120.0, 120.0, 35.0];
            let col_widths_max = [220.0_f32, 55.0, 40.0, 45.0, 45.0, 132.0, 55.0, 50.0, 45.0, 176.0, 176.0, 45.0];
            let col_widths: [f32; 12];
            if table_w < table_min { col_widths = col_widths_min; }
            else if table_w > table_max { col_widths = col_widths_max; }
            else {
                let ratio = (table_w - table_min) / (table_max - table_min);
                col_widths = std::array::from_fn(|i| col_widths_min[i] + ((col_widths_max[i] - col_widths_min[i]) * ratio));
            }
            let headers    = ["Name", "Skill", "TN", "Tag", "Dmg", "Effects", "Type", "Rate", "Rng", "Qualities", "Ammo", "Wgt"];

            ui.columns(headers.len() as i32, "##weap_hdr", true);
            for (i, (hdr, cw)) in headers.iter().zip(col_widths.iter()).enumerate() {
                ui.set_column_width(i as i32, *cw);
                ui.text_colored(theme.text_dim, hdr);
                ui.next_column();
            }
            ui.separator();

            for weap in &weapons {
                let effects_str = weap.effects.iter().map(|e| {
                    match e.value {
                        Some(v) => format!("{}", e.name.replace("X", &v.to_string())),
                        None    => e.name.clone(),
                    }
                }).collect::<Vec<_>>().join(", ");

                let quals_str = weap.qualities.iter().map(|q| {
                    match q.value {
                        Some(v) => format!("{} {}", q.name, v),
                        None    => q.name.clone(),
                    }
                }).collect::<Vec<_>>().join(", ");

                let tag_str = if weap.tagged { "*" } else { "" };
                //let ammo_str = format!("{} ({})", weap.ammo_name, weap.ammo_quantity);
                let skill_str = match weap.skill.as_str() {
                    "Melee Weapons" => "MW",
                    "Unarmed" => "Un",
                    "Small Guns" => "SG",
                    "Big Guns" => "BG",
                    "Throwing" => "Th",
                    "Explosives" => "Ex",
                    "Energy Weapons" => "EW",
                    _ => "",
                }.to_string();

                let mut damage: i32 = weap.damage.parse().unwrap();

                if skill_str == "MW" {
                    damage += stats.melee_base;
                } else if skill_str == "Un" {
                    damage += stats.melee_unarmed + stats.melee_base;
                }

                let cells: &[&str] = &[
                    &weap.name,
                    //&weap.skill,
                    &skill_str,
                    &weap.target_number.to_string(),
                    tag_str,
                    &damage.to_string(),
                    &effects_str,
                    &weap.damage_type,
                    &weap.rate.to_string(),
                    &weap.range,
                    &quals_str,
                    //&ammo_str,
                    &weap.ammo_name,
                    &weap.weight.to_string(),
                ];

                for cell in cells {
                    ui.text_wrapped(cell);
                    ui.next_column();
                }
            }

            ui.columns(1, "##weap_end", false);
        }
        ui.spacing();
    }

    // ── Perks ─────────────────────────────────────────────────────────────────
    // Traits
    let selected_traits: Vec<&str> = nc.traits.iter()
        .enumerate()
        .filter(|(i, _)| nc.selected_traits.get(*i).copied().unwrap_or(false))
        .map(|(_, t)| t.name.as_str())
        .collect();

    if nc.is_ghoul {
        kv(ui, theme, "Type", "Ghoul");
    } else {
        use crate::screens::special::MutantType;
        let type_str = match nc.mutant_type {
            MutantType::StandardSuperMutant => "Super Mutant",
            MutantType::Nightkin            => "Nightkin",
            MutantType::None                => "Human",
        };
        kv(ui, theme, "Type", type_str);
    }

    if selected_traits.is_empty() {
        kv(ui, theme, "Traits", "None");
    } else {
        kv(ui, theme, "Traits", &selected_traits.join(", "));
    }
    ui.spacing();
    
    // Perks
    if !perks.char_perks.is_empty() {
        section_header(ui, theme, "PERKS");
        for pid in &perks.char_perks {
            if let Some(perk) = perks.all_perks.iter().find(|p| p.id == pid.perk_id) {
                ui.text(format!("  - {}", perk.name));
            }
        }
        ui.spacing();
    }

    // ── Inventory ─────────────────────────────────────────────────────────────────
    if let Some(bg) = &equipment.current_bg {
        section_header(ui, theme, "INVENTORY");

        let mut lines: Vec<(&str, Vec<InventoryLine>)> = vec![];

        // ── Weapons ──────────────────────────────────────────────────────────────
        let weapon_lines: Vec<InventoryLine> = weapons.iter()
            .map(|w| InventoryLine {
                name:   w.name.clone(),
                weight: w.weight,
            })
            .collect();

        // ── Ammo ─────────────────────────────────────────────────────────────────
        // Ammo weight is 0 per the system — list for visibility only
        let ammo_lines: Vec<InventoryLine> = weapons.iter()
            .filter(|w| !w.ammo_name.is_empty())
            .map(|w| InventoryLine {
                name:   format!("{} ({})", w.ammo_name, w.ammo_quantity),
                weight: w.ammo_wgt,
            })
            .collect();

        // ── Apparel ───────────────────────────────────────────────────────────────
        let apparel_lines: Vec<InventoryLine> = apparel_pieces.iter()
            .map(|a| InventoryLine {
                name:   a.name.clone(),
                weight: a.wgt,
            })
            .collect();

        // ── Consumables ───────────────────────────────────────────────────────────
        use crate::screens::equipment::{ConsumableSlot, SlotSelection};
        let consumable_lines: Vec<InventoryLine> = bg.consumable_slots.iter()
            .zip(equipment.consumable_selections.iter())
            .flat_map(|(slot, sel)| {
                let opts: Vec<&crate::screens::equipment::ConsumableOption> = match (slot, sel) {
                    (ConsumableSlot::Fixed(opt), _) =>
                        vec![opt],
                    (ConsumableSlot::Choice(opts), SlotSelection::Chosen(i)) if *i < opts.len() =>
                        vec![&opts[*i]],
                    (ConsumableSlot::ManyForOne(giveup, getone), SlotSelection::ManyForOneChosen(choice)) =>
                        if *choice == 0 { vec![getone] } else { giveup.iter().collect() },
                    _ => vec![],
                };
                opts.into_iter().map(|o| InventoryLine {
                    name:   o.name.clone(),
                    weight: o.wgt as i32,   // consumable weight fetched below if needed; 0 for now
                })
            })
            .collect();

        // ── Gear ──────────────────────────────────────────────────────────────────
        let gear_lines: Vec<InventoryLine> = bg.gear.iter()
            .map(|g| InventoryLine {
                name:   g.gear_name.clone(),
                weight: g.wgt as i32,
            })
            .collect();

        // ── Render table ──────────────────────────────────────────────────────────
        let categories: &[(&str, &Vec<InventoryLine>)] = &[
            ("Weapons",     &weapon_lines),
            ("Ammo",        &ammo_lines),
            ("Apparel",     &apparel_lines),
            ("Consumables", &consumable_lines),
            ("Gear",        &gear_lines),
        ];

        let name_col_w   = 320.0;
        let weight_col_w = 60.0;

        ui.columns(2, "##inv_cols", false);
        ui.set_column_width(0, name_col_w);
        ui.set_column_width(1, weight_col_w);

        let mut total_weight: i32 = 0;

        for (cat_name, items) in categories {
            if items.is_empty() { continue; }

            // Category header
            ui.text_colored(theme.text_dim, *cat_name);
            ui.next_column();
            ui.next_column();

            for item in *items {
                ui.text(format!("  {}", item.name));
                ui.next_column();
                if item.weight > 0 {
                    ui.text(item.weight.to_string());
                } else {
                    ui.text_colored(theme.text_dim, "—");
                }
                ui.next_column();
                total_weight += item.weight;
            }

            ui.spacing();
            ui.next_column();
            ui.next_column();
        }

        ui.separator();
        ui.next_column();
        ui.next_column();

        // ── Carry weight summary ──────────────────────────────────────────────────
        // Max carry = Strength × 10 (standard Fallout 2d20 rule)
        let max_carry     = stats.carry_weight;
        let over          = total_weight > max_carry;
        let carry_str     = format!("{} / {} lbs", total_weight, max_carry);

        ui.text_colored(theme.text_dim, "Total Weight");
        ui.next_column();
        if over {
            ui.text_colored([1.0, 0.3, 0.3, 1.0], carry_str);
        } else {
            ui.text(carry_str);
        }
        ui.next_column();

        ui.columns(1, "##inv_end", false);
        ui.spacing();
    }
    
    drop(_scroll);

    // ── Footer ────────────────────────────────────────────────────────────────
    ui.separator();
    ui.spacing();
    ui.set_cursor_pos([16.0, h - 36.0]);
    if ui.button("< Back") {
        *screen = AppScreen::Equipment;
    }
    ui.same_line();
    if ui.button("Create Character") {
        // TODO: serialize and save
        *screen = AppScreen::MainMenu;
    }

}

// ── Layout helpers ────────────────────────────────────────────────────────────

fn section_header(ui: &Ui, theme: &Theme, title: &str) {
    ui.text_colored(theme.text_desc, title);
    ui.separator();
    ui.spacing();
}

fn kv(ui: &Ui, theme: &Theme, key: &str, value: &str) {
    ui.text_colored(theme.text_dim, format!("{:<14}", key));
    ui.same_line();
    ui.text(value);
}

fn weapon_display_name(opt: &crate::screens::equipment::WeaponOption) -> String {
    let mut s = opt.name.clone();
    if let Some(m) = &opt.mod_name {
        s.push_str(&format!(" w/ {}", m));
    }
    if !opt.extra_mods.is_empty() {
        s.push_str(&format!(" + {}", opt.extra_mods.join(", ")));
    }
    s
}