use imgui::Ui;
use sdl2::video::Window;
use crate::{AppScreen, render_placeholder};
use crate::screens::special::SpecialState;
use crate::screens::skills::{ SkillsState, SKILLS };
use crate::screens::perks::PerksState;
use crate::screens::new_character::NewCharacterState;

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Staggered bonus: used for melee damage and some DR perks
/// Returns bonus CDs based on a SPECIAL stat value
pub fn get_staggered_bonus(val: i32) -> i32 {
    match val {
        7..8   => 1,
        9..10  => 2,
        10..   => 3,
        _      => 0,
    }
}

fn has_trait(char_state: &NewCharacterState, id: i64) -> bool {
    char_state.traits.iter()
        .zip(char_state.selected_traits.iter())
        .any(|(t, &sel)| t.id == id && sel)
}

fn has_perk(perks: &PerksState, id: i64) -> bool {
    perks.has_perk(id)
}

fn perk_ranks(perks: &PerksState, id: i64) -> i32 {
    perks.get_ranks(id) as i32
}

// ── Computed Stats ────────────────────────────────────────────────────────────

pub struct ComputedStats {
    pub carry_weight:  i32,
    pub ph_dr:         i32,
    pub en_dr:         i32,
    pub rd_dr:         i32,  // 99 = immune
    pub poison_dr:     i32,  // 99 = immune
    pub defense:       i32,
    pub initiative:    i32,
    pub max_hp:        i32,
    pub is_nocturnal:  bool, // night HP = max_hp + END
    pub melee_base:    i32,
    pub melee_unarmed: i32,  // extra CD for unarmed (Iron Fist)
    pub melee_sneak:   i32,  // extra CD for sneak (Ninja)
    pub max_luck_pts:  i32,
    pub has_companion: bool,
}

pub fn compute_stats(
    special: &SpecialState,
    traits: &NewCharacterState,
    perks: &PerksState,
) -> ComputedStats {
    let sp = |i: usize| special.display_value(i);
    let str = sp(0);
    let _per = sp(1);
    let end = sp(2);
    let _cha = sp(3);
    let _int = sp(4);
    let agi = sp(5);
    let lck = sp(6);

    // ── Carry Weight ──────────────────────────────────────────────
    let strong_back = perk_ranks(perks, 91) * 25;
    let carry_weight = if [4i64, 19, 20, 23].iter().any(|&id| has_trait(traits, id)) {
        150
    } else if has_trait(traits, 18) {
        225
    } else if has_trait(traits, 9) {
        150 + (5 * str) + strong_back
    } else {
        150 + (10 * str) + strong_back
    };

    // ── Poison DR ─────────────────────────────────────────────────
    let poison_dr = if [3i64, 4, 18, 19, 20, 21, 23, 25].iter().any(|&id| has_trait(traits, id)) {
        99
    } else if has_perk(perks, 87) {
        perk_ranks(perks, 87) * 2
    } else {
        0
    };

    // ── Radiation DR ──────────────────────────────────────────────
    let rd_dr = if [2i64, 3, 4, 18, 19, 20, 21, 23, 25].iter().any(|&id| has_trait(traits, id)) {
        99
    } else {
        let child_of_atom = if has_trait(traits, 22) { 1 } else { 0 };
        let rad_resistance = perk_ranks(perks, 73);
        child_of_atom + rad_resistance
    };

    // ── Physical DR ───────────────────────────────────────────────
    let barbarian  = if has_perk(perks, 8)   { get_staggered_bonus(str) } else { 0 };
    let toughness  = perk_ranks(perks, 94);
    let evasive_ph = if has_perk(perks, 167) { get_staggered_bonus(agi) } else { 0 };
    let ph_dr = barbarian + toughness + evasive_ph;

    // ── Energy DR ─────────────────────────────────────────────────
    let refractor  = perk_ranks(perks, 74);
    let evasive_en = if has_perk(perks, 167) { get_staggered_bonus(agi) } else { 0 };
    let en_dr = refractor + evasive_en;

    // ── Defense ───────────────────────────────────────────────────
    let defense = if agi >= 9 { 2 } else { 1 };

    // ── Initiative ────────────────────────────────────────────────
    let initiative = _per + agi;

    // ── Max HP ────────────────────────────────────────────────────
    let life_giver = perk_ranks(perks, 51);
    let max_hp = end + lck + life_giver * end;
    let is_nocturnal = has_perk(perks, 111);

    // ── Melee Damage ─────────────────────────────────────────────
    let brutal_trait  = if has_trait(traits, 8)  { 1 } else { 0 };
    let built_trait   = if has_trait(traits, 23) { 1 } else { 0 };
    let melee_base    = get_staggered_bonus(str) + brutal_trait + built_trait;
    let melee_unarmed = if has_perk(perks, 46) { 1 } else { 0 };
    let melee_sneak   = if has_perk(perks, 61) { 2 } else { 0 };

    // ── Luck Points ───────────────────────────────────────────────
    let gifted = if has_trait(traits, 7) { 1 } else { 0 };
    let max_luck_pts = lck - gifted;

    // ── Companion ─────────────────────────────────────────────────
    let has_companion = has_perk(perks, 28) || has_perk(perks, 105);

    ComputedStats {
        carry_weight, ph_dr, en_dr, rd_dr, poison_dr,
        defense, initiative, max_hp, is_nocturnal,
        melee_base, melee_unarmed, melee_sneak,
        max_luck_pts, has_companion,
    }
}

// ── Validation ────────────────────────────────────────────────────────────────

pub struct ValidationResult {
    pub valid: bool,
    pub issues: Vec<&'static str>,
}

pub fn validate_all(
    skills: &SkillsState,
    perks: &PerksState,
) -> ValidationResult {
    let mut issues = Vec::new();

    // Unspent skill points
    if skills.remaining_points() > 0 {
        issues.push("Unspent skill points on the Skills page");
    }

    // Unselected tag skills
    let tagged = skills.skills.iter().filter(|s| s.tagged).count();
    let tag_slots = skills.base_tag_slots() + skills.extra_tag_count + skills.perk_tag_slots;
    if tagged < tag_slots {
        issues.push("Not all tag skills have been selected");
    }

    // Unspent perk slots
    if perks.perks_remaining() > 0 {
        issues.push("Unspent perk points on the Perks page");
    }

    ValidationResult {
        valid: issues.is_empty(),
        issues,
    }
}

// ── Render ────────────────────────────────────────────────────────────────────

pub fn render_stats(
    ui: &Ui,
    window: &Window,
    special: &SpecialState,
    skills: &SkillsState,
    perks: &PerksState,
    traits: &NewCharacterState,
    screen: &mut AppScreen,
) {
    let (win_w, win_h) = window.size();
    let w = (win_w as f32 * 0.65).min(960.0);
    let h = win_h as f32 * 0.85;
    let bar_h = crate::BAR_HEIGHT;
    let content_h = win_h as f32 - bar_h;

    let Some(_tok) = ui.window("##stats")
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

    ui.text("STATS");
    ui.separator();
    ui.spacing();

    let stats = compute_stats(special, traits, perks);
    let validation = validate_all(skills, perks);

    // ── Validation warnings ───────────────────────────────────────
    if !validation.valid {
        for issue in &validation.issues {
            ui.text_colored([1.0, 0.4, 0.4, 1.0], format!("⚠ {}", issue));
        }
        ui.spacing();
        ui.separator();
        ui.spacing();
    }

    // ── Scrollable content ────────────────────────────────────────
    let list_h = h - 80.0;
    let Some(_child) = ui.child_window("##stats_scroll")
        .size([w - 16.0, list_h])
        .begin()
    else { return; };

    let col2 = (w - 16.0) * 0.5;

    // ── Left column ───────────────────────────────────────────────
    ui.text("DERIVED STATS");
    ui.separator();
    ui.spacing();

    ui.text(format!("Carry Weight:   {}", stats.carry_weight));
    ui.spacing();

    ui.text("Base Damage Resistance:");
    ui.text(format!(
        "  Physical: {}   Energy: {}   Radiation: {}   Poison: {}",
        stats.ph_dr,
        stats.en_dr,
        if stats.rd_dr == 99 { "Immune".to_string() } else { stats.rd_dr.to_string() },
        if stats.poison_dr == 99 { "Immune".to_string() } else { stats.poison_dr.to_string() },
    ));
    ui.spacing();

    ui.text(format!("Defense:        {}", stats.defense));
    ui.text(format!("Initiative:     {}", stats.initiative));

    // HP — show night variant if Nocturnal Fortitude
    if stats.is_nocturnal {
        ui.text(format!(
            "Health:         {} ({} at night)",
            stats.max_hp,
            stats.max_hp + special.display_value(2)
        ));
    } else {
        ui.text(format!("Health:         {}", stats.max_hp));
    }

    // Melee damage
    let melee_str = build_melee_string(&stats);
    ui.text(format!("Melee Damage:   {}", melee_str));

    ui.text(format!("Luck Points:    {}", stats.max_luck_pts));

    ui.spacing();
    ui.spacing();

    // ── SPECIAL column ────────────────────────────────────────────
    ui.text("SPECIAL");
    ui.separator();
    ui.spacing();

    const STAT_LABELS: [&str; 7] = [
        "Strength", "Perception", "Endurance",
        "Charisma", "Intelligence", "Agility", "Luck",
    ];
    for i in 0..7 {
        ui.text(format!("  {:12} {}", STAT_LABELS[i], special.display_value(i)));
    }

    ui.spacing();
    ui.spacing();

    // ── Skills ────────────────────────────────────────────────────
    ui.text("SKILLS");
    ui.separator();
    ui.spacing();

    for (i, skill) in skills.skills.iter().enumerate() {
        let skill_total = skill.ranks + if skill.tagged {2} else {0};
        if skill_total > 0 {
            let tag_str = if skill.tagged { " (Tag)" } else { "" };
            ui.text(format!("  {:22} {}{}", SKILLS[i], skill_total, tag_str));
        }
    }

    ui.spacing();
    ui.spacing();

    // ── Perks ─────────────────────────────────────────────────────
    if !perks.char_perks.is_empty() {
        ui.text("PERKS");
        ui.separator();
        ui.spacing();

        for cp in &perks.char_perks {
            if let Some(perk) = perks.all_perks.iter().find(|p| p.id == cp.perk_id) {
                let rank_str = if perk.ranks > 1 {
                    format!(" (Rank {})", cp.ranks)
                } else {
                    String::new()
                };
                ui.text(format!("  {}{}", perk.name, rank_str));
            }
        }
    }

    drop(_child);

    // ── Footer ────────────────────────────────────────────────────
    let footer_y = h - 44.0;
    ui.set_cursor_pos([16.0, footer_y]);
    if ui.button("< Back") {
        *screen = AppScreen::Perks;
    }
    ui.same_line();
    let _g = (!validation.valid).then(|| ui.begin_disabled(true));
    if ui.button("Next >") {
        //*screen = AppScreen::Equipment;
        render_placeholder(&ui, &window, "equipment", screen);
    }
    drop(_g);

    if !validation.valid {
        ui.same_line();
        ui.text_colored([1.0, 0.4, 0.4, 1.0], "Resolve issues above before continuing.");
    }
}

// ── Melee string builder ──────────────────────────────────────────────────────

fn build_melee_string(stats: &ComputedStats) -> String {
    let mut parts = vec![format!("+{}CD", stats.melee_base)];

    if stats.melee_unarmed > 0 && stats.melee_sneak > 0 {
        parts.push(format!(
            "(+{}CD unarmed, +{}CD sneak, +{}CD unarmed sneak)",
            stats.melee_unarmed,
            stats.melee_sneak,
            stats.melee_unarmed + stats.melee_sneak,
        ));
    } else if stats.melee_unarmed > 0 {
        parts.push(format!("(+{}CD unarmed)", stats.melee_unarmed));
    } else if stats.melee_sneak > 0 {
        parts.push(format!("(+{}CD sneak attacks)", stats.melee_sneak));
    }

    parts.join(" ")
}