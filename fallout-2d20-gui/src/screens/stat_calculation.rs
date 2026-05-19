use fallout_2d20_core::{
    character::Character,
    constants::{
        SKILLS,
        SPECIAL_LABELS
    },
    get_melee_str,
    states::{
        BackgroundState,
        EquipmentState,
        OriginState,
        PerkState,
        SkillState,
        SpecialState
    },
    structs::AppScreen
};
use imgui::Ui;
use sdl2::video::Window;
use crate::{ theme::{render_text_wrapped, render_window}};

pub fn render_stat_calculation(
    ui: &Ui,
    window: &Window,
    special: &mut SpecialState,
    skill: &mut SkillState,
    character: &mut Character,
    screen: &mut AppScreen,
    origin: &mut OriginState,
    perk: &mut PerkState,
    background: &mut BackgroundState,
    equipment: &mut EquipmentState,
) -> f32 {
    let Some((w, h, _token)) = render_window(ui, window, "##stat_calculation", "Calculated Stats", screen, origin, special, skill, perk, background, equipment, character)
        else { return 0.0 };

    let nocturnal = character.compute_stats();
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

    let base_dr = character.base_dr.clone();
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