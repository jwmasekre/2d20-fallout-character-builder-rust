use imgui::Ui;
use sdl2::video::Window;
use crate::{character::{Character, CompanionType, MeleeModifiers}, screens::{skill_assignment::{SKILLS, SkillState}, special_assignment::{SPECIAL_LABELS, SpecialState}}, theme::{render_text_wrapped, render_window}};

pub fn get_staggered_bonus(val: i32) -> i32 {
    match val {
        7..9 => 1,
        9..11 => 2,
        11.. => 3,
        _ => 0,
    }
}

pub struct BaseDR {
    pub ph_dr: i32,
    pub en_dr: i32,
    pub rd_dr: i32,
}

pub fn get_melee_str(character: &Character) -> String {
    let mut melee_string_vec: Vec<String> = vec![format!("+{}CD", character.melee_mod.melee)];
    if character.melee_mod.unarmed > 0 {
        melee_string_vec.push(format!("+{}CD unarmed", character.melee_mod.melee + character.melee_mod.unarmed))
    }
    if character.melee_mod.sneak > 0 {
        melee_string_vec.push(format!("+{}CD sneak", character.melee_mod.melee + character.melee_mod.sneak))
    }
    if character.melee_mod.unarmed > 0 && character.melee_mod.sneak > 0 {
        melee_string_vec.push(format!("+{}CD unarmed sneak", character.melee_mod.melee + character.melee_mod.sneak + character.melee_mod.unarmed))
    }
    melee_string_vec.join(", ")
}

pub fn compute_stats(character: &mut Character) -> (BaseDR, bool) {
    //quick reference for stats
    let str = character.special.strength.value;
    let per = character.special.perception.value;
    let end = character.special.endurance.value;
    let _cha = character.special.charisma.value;
    let _int = character.special.intelligence.value;
    let agi = character.special.agility.value;
    let lck = character.special.luck.value;

    //carry weight
    let strong_back = (character.perk_ranks(91)) * 25;
    character.carry_wgt_max = if character.has_any_trait(vec![4,19,20,23]) {
        150
    } else if character.has_trait(18) {
        225
    } else if character.has_trait(9) {
        150 + (5 * str) + strong_back
    } else {
        150 + (10 * str) + strong_back
    };
    //poison dr
    character.poison_dr = if character.is_mutant() || character.is_robot() {
        99
    } else if character.has_perk(87) {
        2
    } else {
        0
    };
    //rad dr
    let rd_dr = if character.is_mutant() || character.is_robot() {
        99
    } else {
        let atom = if character.origin.clone().unwrap().id == 13 { 1 } else { 0 };
        let rad_res = character.perk_ranks(73);
        atom + rad_res
    };
    //physical dr
    let barbarian = if character.has_perk(8) {
        get_staggered_bonus(str)
    } else { 0 };
    let toughness = character.perk_ranks(94);
    let evasive = if character.has_perk(167) {
        get_staggered_bonus(agi)
    } else { 0 };
    let ph_dr = barbarian + toughness + evasive;
    //energy dr
    let refractor = character.perk_ranks(74);
    let en_dr = evasive + refractor;

    let base_dr = BaseDR {
        ph_dr,
        en_dr,
        rd_dr,
    };
    //defense
    character.defense = if agi >= 9 { 2 } else { 1 };
    //initiative
    character.initiative = per + agi;
    //max hp
    character.hp_max = end + lck + character.perk_ranks(51) * end;
    let is_nocturnal = character.has_perk(111);
    //melee damage
    let brutal = if character.has_trait(8) { 1 } else { 0 };
    let built = if character.has_trait(23) { 1 } else { 0 };
    character.melee_mod = MeleeModifiers {
        melee: get_staggered_bonus(str) + brutal + built,
        unarmed: if character.has_perk(46) { 1 } else { 0 },
        sneak: if character.has_perk(61) { 2 } else { 0 },
    };
    //max luck points
    character.luck_points_max = if character.is_gifted() { lck } else { lck - 1 };
    //companion
    character.companion = if character.has_perk(28) {
        CompanionType::Dogmeat
    } else if character.has_perk(105) {
        CompanionType::Human
    } else if character.has_perk(118) {
        CompanionType::Robot
    } else {
        CompanionType::None
    };
    //the character struct doesn't have a base dr, so we need to return this
    (base_dr, is_nocturnal)
}

pub fn render_stat_calculation(
    ui: &Ui,
    window: &Window,
    special: &SpecialState,
    skill: &SkillState,
    character: &mut Character,
) -> f32 {
    let Some((w, h, _token)) = render_window(ui, window, "##stat_calculation", "Calculated Stats")
        else { return 0.0 };

    let (base_dr, nocturnal) = compute_stats(character);
    let char_spec = character.special.special_block();
    let char_skills = character.skills.skill_block();

    ui.text("STATS");
    ui.separator();
    ui.spacing();
    
    let mut issues: Vec<&str> = vec![];
    if !special.is_complete(character) {issues.push("!! SPECIAL needs attention !!")}
    if !skill.is_complete(character) {issues.push("!! Skills need attention !!")}
    if !issues.is_empty() {
        for issue in issues {
            render_text_wrapped(false, true, ui, issue, 36.0, w - 36.0);
        }
        ui.spacing();
        ui.separator();
        ui.spacing();
    }

    let list_h = h - 80.0;
    let Some(_child) = ui.child_window("##stats_scroll")
        .size([w - 16.0, list_h])
        .begin()
    else { return h };
    let d_col_w = (w - 24.0) * 0.5;

    ui.text("Derived");
    ui.separator();
    ui.spacing();

    ui.columns(2, "##derived_cols", false);
    ui.set_column_width(0, d_col_w);
    ui.set_column_width(1, d_col_w);

    ui.text(format!("Max Carry Weight: {}", character.carry_wgt_max));
    ui.text("Base Damage Resistance:");
    ui.text(format!(
        "    Phys: {}  Enrg: {}  Rads: {}  Poison: {}",
        base_dr.ph_dr,
        base_dr.en_dr,
        if base_dr.rd_dr == 99 { "Immune".to_string() } else { base_dr.rd_dr.to_string() },
        if character.poison_dr == 99 { "Immune".to_string() } else { character.poison_dr.to_string() },
    ));
    ui.text(format!("Max Luck Points: {}", character.luck_points_max));

    ui.next_column();

    ui.text(format!("Defense: {}", character.defense));
    ui.text(format!("Initiative: {}", character.initiative));
    if nocturnal {
        ui.text(format!("Max Health: {} ({} at night)", character.hp_max, character.hp_max + character.special.endurance.value))
    } else {
        ui.text(format!("Max Health: {}", character.hp_max));
    }
    let melee_string = get_melee_str(character);

    ui.text(format!("Melee Damage: {}", melee_string));

    ui.spacing();
    ui.separator();
    ui.spacing();

    ui.columns(1, "##special", false);

    ui.text("SPECIAL");
    ui.separator();
    ui.spacing();

    for i in 0..7 {
        ui.text(format!("   {}:{:4}   ",SPECIAL_LABELS[i].chars().next().unwrap(), char_spec[i].value));
        if i < 6 {
            ui.same_line();
        }
    }

    ui.spacing();
    ui.separator();
    ui.spacing();

    ui.text("Skills");
    ui.separator();
    ui.spacing();

    let mut active_skills = vec![];
    for i in 0..17 {
        if char_skills[i].total > 0 {
            active_skills.push((SKILLS[i],char_skills[i].total, if char_skills[i].is_tagged() {"(Tag)"} else {"-----"}))
        }
    }
    let rows = ((active_skills.len() - 1) / 3) + 1;

    for i in 0..rows {
        ui.text(format!("  {:20} {} {}  ", active_skills[i].0, active_skills[i].1, active_skills[i].2));
        ui.same_line();
        ui.text(format!("  {:20} {} {}  ", active_skills[i+rows].0, active_skills[i+rows].1, active_skills[i+rows].2));
        if active_skills.len() > i + rows * 2 {
            ui.same_line();
            ui.text(format!("  {:20} {} {}", active_skills[i+rows*2].0, active_skills[i+rows*2].1, active_skills[i+rows*2].2));
        }
    }

    ui.spacing();
    ui.separator();
    ui.spacing();

    ui.text("Perks");
    ui.separator();
    ui.spacing();

    ui.columns(2, "##perk_cols", false);
    ui.set_column_width(0, d_col_w);
    ui.set_column_width(1, d_col_w);

    let (perk_l, perk_r) = character.perks.split_at((character.perks.len() + 1) / 2);
    for perk in perk_l {
        ui.text(&perk.name);
        if perk.desc.len() > 1 {
            for i in 0..(perk.ranks as usize) {
                render_text_wrapped(false, true, ui, &format!("  {:2}  {}", i+1, perk.desc[i]), 0.0, d_col_w - 6.0);
            }
            ui.spacing();
        } else {
            render_text_wrapped(false, true, ui, &format!("  {:2}  {}", perk.ranks, perk.desc[0]), 0.0, d_col_w - 6.0);
            ui.spacing();
        }
    }
    ui.next_column();
    for perk in perk_r {
        ui.text(&perk.name);
        if perk.desc.len() > 1 {
            for i in 0..(perk.ranks as usize) {
                render_text_wrapped(false, true, ui, &format!("  {:2}  {}", i+1, perk.desc[i]), d_col_w + 6.0, w - 6.0);
            }
            ui.spacing();
        } else {
            render_text_wrapped(false, true, ui, &format!("  {:2}  {}", perk.ranks, perk.desc[0]), d_col_w + 6.0, w - 6.0);
            ui.spacing();
        }
        ui.spacing();
    }
    h
}