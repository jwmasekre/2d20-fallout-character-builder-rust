use fallout_2d20_core::{
    character::Character, constants::{
        SKILLS,
        SPECIAL_LABELS,
    }, db::Db, equip_bg_apparel, get_melee_str, render_inventory, render_weapons, states::{
        BackgroundState,
        EquipmentState,
        OriginState,
        PerkState,
        ReviewState,
        SkillState,
        SpecialState,
    }, structs::AppConfig
};
use imgui::Ui;
use sdl2::video::Window;
use crate::{
    AppScreen,
    theme::render_window
};

pub fn render_character_review(
    ui: &Ui,
    window: &Window,
    state: &mut ReviewState,
    background: &mut BackgroundState,
    equipment: &mut EquipmentState,
    db: &Db,
    character: &mut Character,
    screen: &mut AppScreen,
    origin: &mut OriginState,
    special: &mut SpecialState,
    skill: &mut SkillState,
    perk: &mut PerkState,
    cfg: &AppConfig,
) -> f32 {
    let Some((w, h, _token)) = render_window(ui, window, "##character_review", "Character Review", screen, origin, special, skill, perk, background, equipment, character, cfg)
        else { return 0.0 };

    ui.text("REVIEW");
    ui.separator();
    ui.spacing();

    if !state.loaded {
        equip_bg_apparel(character, equipment, background);
        state.loaded = true;
    }
    let Some(_scroll) = ui.child_window("##review_scroll")
        .size([w - 16.0 * cfg.ui_scale, h - 32.0 * cfg.ui_scale - 48.0 * cfg.ui_scale])
        .begin()
    else { return h };

    //let col_w = (w - 48.0 * cfg.ui_scale) / 2.0;
    let col_w_l = (w - 48.0 * cfg.ui_scale) / 2.0 - 60.0 * cfg.ui_scale;
    let col_w_r = (w - 48.0 * cfg.ui_scale) / 2.0 + 60.0 * cfg.ui_scale;

    ui.text_disabled("IDENTITY");
    ui.separator();
    ui.spacing();

    ui.columns(2, "##id_cols", false);
    ui.set_column_width(0, col_w_l);
    ui.set_column_width(1, col_w_r);

    let origin_name = if character.origin.is_some() {character.origin.clone().unwrap().name} else { String::new() };
    
    ui.text_disabled(format!("{:14}", "Name"));
    ui.same_line();
    ui.text(format!("{}",character.name));
    ui.text_disabled(format!("{:14}", "Level"));
    ui.same_line();
    ui.text(format!("{}",character.level));
    ui.text_disabled(format!("{:14}", "XP"));
    ui.same_line();
    ui.text(format!("{}",character.xp));
    ui.text_disabled(format!("{:14}", "Origin"));
    ui.same_line();
    ui.text(format!("{}",origin_name));
    ui.text_disabled(format!("{:14}", "Background"));
    ui.same_line();
    ui.text(format!("{}",character.background.clone().unwrap().name));

    ui.next_column();

    let melee_str = get_melee_str(character);

    let poison_str = if character.poison_dr == 99 {
        "Immune".to_string()
    } else {
        character.poison_dr.to_string()
    };
    ui.text_disabled(format!("{:14}", "Poison DR"));
    ui.same_line();
    ui.text(format!("{}",poison_str));
    ui.text_disabled(format!("{:14}", "Defense"));
    ui.same_line();
    ui.text(format!("{}",character.defense));
    ui.text_disabled(format!("{:14}", "Initiative"));
    ui.same_line();
    ui.text(format!("{}",character.initiative));
    ui.text_disabled(format!("{:14}", "HP"));
    ui.same_line();
    ui.text(format!("{}/{}",character.hp,character.hp_max));
    ui.text_disabled(format!("{:14}", "Melee"));
    ui.same_line();
    ui.text_wrapped(format!("{}",melee_str));

    ui.columns(1, "##id_cols_end", false);
    ui.spacing();

    ui.text_disabled("SPECIAL");
    ui.separator();
    ui.spacing();

    for i in 0..7 {
        ui.text(format!("   {}:{:4}   ",SPECIAL_LABELS[i].chars().next().unwrap(), character.special.special_block()[i].value));
        if i < 6 {
            ui.same_line();
        }
    }
    let spacer = "             ";
    ui.text_disabled(format!("{}{}{}{}{}  Luck Points:",spacer,spacer,spacer,spacer,spacer,));
    ui.same_line();
    ui.text(format!("{:2}/{:2}",character.luck_points,character.luck_points_max));

    ui.text_disabled("SKILLS");
    ui.separator();
    ui.spacing();

    let mut active_skills = vec![];
    for i in 0..17 {
        if character.skills.skill_block()[i].total > 0 {
            active_skills.push((SKILLS[i],character.skills.skill_block()[i].total, if character.skills.skill_block()[i].is_tagged() {"(Tag)"} else {"-----"}))
        }
    }
    let rows = ((active_skills.len() - 1) / 3) + 1;

    for i in 0..rows {
        ui.text(format!("  {:16} {} {}    ", active_skills[i].0, active_skills[i].1, active_skills[i].2));
        ui.same_line();
        ui.text(format!("  {:16} {} {}    ", active_skills[i+rows].0, active_skills[i+rows].1, active_skills[i+rows].2));
        if active_skills.len() > i + rows * 2 {
            ui.same_line();
            ui.text(format!("  {:16} {} {}", active_skills[i+rows*2].0, active_skills[i+rows*2].1, active_skills[i+rows*2].2));
        }
    }

    ui.text_disabled("DR");
    ui.separator();
    ui.spacing();
    ui.text_disabled("temporary display:");
    character.limb_dr.update_dr(character.base_dr.clone(), character.perk_ranks(144), character.junk.common + character.junk.uncommon + character.junk.rare, character.perk_ranks(172));
    let active_limbs = character.limb_dr.mut_active_limbs();
    for (limb, name) in active_limbs {
        let worn: Vec<String> = limb.equipped.iter().map(|a| a.name.clone()).collect();
        ui.text(format!("{:10} - P:{} E:{} R:{} - {}", name, limb.ph_dr, limb.en_dr, if limb.rd_dr < 99 {limb.rd_dr.to_string()} else {"Immune".to_string()}, worn.join(", ")));
    }
    state.debug_load = false;
    /*
    DR BLOCK
    */

    ui.text_disabled("WEAPONS");
    ui.separator();
    ui.spacing();

    render_weapons(ui, equipment.weapons.clone(), character, cfg);

    ui.spacing();

    ui.text_disabled("TRAITS/PERKS");
    ui.separator();
    ui.spacing();

    let character_type = if character.ghoul {
        if character.origin.clone().unwrap().id != 2 {
            format!("Ghoul ({})", origin_name)
        } else {
            "Ghoul".to_string()
        }
    } else if character.is_mutant() {
        if character.origin.clone().unwrap().id != 3 {
            format!("Super Mutant ({})", origin_name)
        } else {
            "Super Mutant".to_string()
        }
    } else if character.is_robot() {
        format!("Robot ({})", origin_name)
    } else {
        format!("Human ({})", origin_name)
    };
    let selected_traits: Vec<String> = character.traits.iter()
        .map(|t| t.clone().name)
        .collect();
    let selected_perks: Vec<String> = character.perks.iter()
        .map(|p| p.clone().name)
        .collect();

    ui.text_disabled(format!("{:14}", "Type"));
    ui.same_line();
    ui.text(format!("{}",character_type));
    ui.text_disabled(format!("{:14}", "Traits"));
    ui.same_line();
    ui.text(format!("{}",selected_traits.join(", ")));
    ui.text_disabled(format!("{:14}", "Perks"));
    ui.same_line();
    ui.text(format!("{}",selected_perks.join(", ")));

    ui.spacing();

    ui.text_disabled("INVENTORY");
    ui.separator();
    ui.spacing();

    render_inventory(ui, equipment.ammo.clone(), equipment.apparel.clone(), equipment.consumables.clone(), equipment.robot_modules.clone(), equipment.gear.clone(), equipment.junk.clone(), equipment.misc.clone(), character, db, cfg);


    ui.columns(1,"##end_eq", false);

    /*
    ui.separator();
    ui.separator();
    ui.text("DEBUG");
    ui.separator();
    ui.separator();
    ui.text_wrapped(format!("{:?}", character));
    ui.separator();
    ui.text_wrapped(format!("{:?}", equipment));
    ui.separator();
    ui.text_wrapped(format!("{:?}", background));
    */
    drop(_scroll);
    h
}