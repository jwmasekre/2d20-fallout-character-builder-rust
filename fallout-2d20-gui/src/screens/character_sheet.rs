
use fallout_2d20_core::{
    apply_level_change, character::{
        AmmoInv,
        ApparelType,
        Character,
        ConsumableType,
        RobotType,
        WeaponMods
    }, constants::{
        SKILLS,
        SPECIAL_LABELS
    }, db::{
        Db,
        load_perks
    }, export_character, get_melee_str, render_inventory, render_weapons, sanitize_filename, states::{
        BackgroundState,
        EquipmentState,
        InventoryTab,
        OriginState,
        PerkState,
        SheetState,
        SkillState,
        SpecialState
    }, structs::AppConfig, sync_derived_weapons
};
use imgui::Ui;
use sdl2::video::Window;
use crate::{
    AppScreen,
    theme::{render_expandable_block, render_window},
};

pub fn render_character_sheet(
    ui: &Ui,
    window: &Window,
    db: &Db,
    character: &mut Character,
    screen: &mut AppScreen,
    state: &mut SheetState,
    origin: &mut OriginState,
    special: &mut SpecialState,
    skill: &mut SkillState,
    perk: &mut PerkState,
    background: &mut BackgroundState,
    equipment: &mut EquipmentState,
    cfg: &AppConfig
) -> f32 {
    let Some((w, h, _token)) = render_window(ui, window, "##character_sheet", "Character Sheet", screen, origin, special,  skill, perk, background, equipment, character, cfg)
        else { return 0.0 };

    //log_on_change!(character);

    //could probably do columns here with wrapping
    ui.text(format!("{} --- {} ({})", character.name, character.player.name, character.party.name));
    ui.same_line();
    ui.text(format!("                {:4}xp", character.xp));
    ui.same_line();
    let mut char_clone = character.clone();
    char_clone.calculate_level();
    if char_clone.level < character.level {
        if ui.button("Delevel##level_down") {
            state.up = false;
            state.level = true;
        }
    } else if character.xp_next > 0 {
        ui.text(format!(" ({} to next) ", character.xp_next))
    } else {
        if ui.button("Level Up##level_up") {
            state.skill_choice = i32::MAX;
            state.perk_choice = i32::MAX;
            state.perks = load_perks(db);
            state.up = true;
            state.level = true;
        }
    }
    ui.same_line();
    ui.text(format!("Lv {}", character.level));
    ui.same_line();
    if ui.button("Add/Remove XP##xp_open") {
        state.xp_open = true;
        state.xp_amount = 0;
    }
    ui.same_line_with_pos(w - 140.0 * cfg.ui_scale);
    if ui.button("Notes##notes_open") {
        state.notes_open = true;
        state.notes_buf = character.notes.clone();
    }
    ui.same_line_with_pos(w - 88.0 * cfg.ui_scale);
    ui.text_disabled("|");
    ui.same_line_with_pos(w - 80.0 * cfg.ui_scale);
    if ui.button("Export##export") {
        let default_name = format!("{}.json", sanitize_filename(&character.name));
        if let Some(path) = rfd::FileDialog::new()
            .set_file_name(&default_name)
            .add_filter("JSON", &["json"])
            .save_file()
        {
            match export_character(&character, &path) {
                Ok(_) => {},
                Err(e) => eprintln!("Export failed: {e}"),
            };
        }
    }
    ui.separator();
    ui.spacing();

    let Some(_scroll) = ui.child_window("##sheet_scroll")
        .size([w - 16.0 * cfg.ui_scale, h - 92.0 * cfg.ui_scale])
        .begin()
    else { return h };
    
    let o_padding = 16.0 * cfg.ui_scale;
    let o_block_w = w - o_padding * 2.0;
    let o_gap = 8.0 * cfg.ui_scale;
    let o_col_w = (o_block_w - o_gap) / 2.0;
    let o_collapse_h = 44.0 * cfg.ui_scale;
    let o_expanded_h = 160.0 * cfg.ui_scale;

    let origin_h = if state.origin_expanded { o_expanded_h } else { o_collapse_h };
    let background_h = if state.background_expanded { o_expanded_h } else { o_collapse_h };
    let total_h = origin_h.max(background_h) + 16.0 * cfg.ui_scale;

    ui.set_cursor_pos([o_padding, ui.cursor_pos()[1]]);
    ui.child_window("##ob_block")
        .size([o_block_w, total_h])
        .border(false)
        .build(|| {
            render_expandable_block(
                ui,
                "##origin_col",
                o_col_w,
                origin_h,
                &mut state.origin_expanded,
                &character.origin.as_ref().map(|o| o.name.as_str()).unwrap_or("no origin"),
                character.origin.as_ref().map(|o| o.desc.as_str()),
                cfg,
            );
            ui.same_line_with_spacing(0.0, o_gap);
            render_expandable_block(
                ui,
                "##background_col",
                o_col_w,
                background_h,
                &mut state.background_expanded,
                character.background.as_ref().map(|b| b.name.as_str()).unwrap_or("no background"),
                character.background.as_ref().map(|b| b.desc.as_str()),
                cfg,
            );
    });
    ui.separator();
    ui.spacing();
    ui.separator();
    ui.spacing();

    for i in 0..7 {
        ui.text(format!("   {}:{:4}   ",SPECIAL_LABELS[i].chars().next().unwrap(), character.special.special_block()[i].value));
        ui.same_line();
    }
    if ui.button("-##lp_dec") {
        if character.luck_points > 0 {
            character.luck_points -= 1;
            db.update_lp(character);
        }
    }
    ui.same_line();
    ui.text(format!("{}/{}", character.luck_points, character.luck_points_max));
    ui.same_line();
    if ui.button("+##lp_inc") {
        if character.luck_points < character.luck_points_max {
            character.luck_points += 1;
            db.update_lp(character);
        }
    }
    if character.luck_points < 0 { character.luck_points = 0 };
    if character.luck_points > character.luck_points_max { character.luck_points = character.luck_points_max };
    ui.same_line();
    ui.text("LP");
       
    if character.origin.as_ref().map(|o| o.id).unwrap_or(0) == 13 {
        ui.same_line();
        if ui.button("-##rp_dec") {
            if character.rad_points > 0 {
                character.rad_points -= 1;
                db.update_rp(character);
            }
        }
        ui.same_line();
        ui.text(format!("{}/5", character.rad_points));
        ui.same_line();
        if ui.button("+##rp_inc") {
            if character.rad_points < 5 {
                character.rad_points += 1;
                db.update_rp(character);
            }
        }
        if character.rad_points < 0 { character.rad_points = 0 };
        if character.rad_points > 5 { character.luck_points = 5 };
        ui.same_line();
        ui.text("RP");
    }
    ui.separator();
    ui.spacing();
    ui.separator();
    ui.spacing();

    let skill_cursor = ui.cursor_pos();
    //let debug_skill_cursor = ui.cursor_screen_pos();
    //let debug_diff = [debug_skill_cursor[0] - skill_cursor[0], debug_skill_cursor[1] - skill_cursor[1]];
    ui.text("T  Skill              Ranks");
    ui.separator();
    for (i, skill) in character.skills.skill_block().iter().enumerate() {
        ui.text(format!("{}  {:.<20} {}", if skill.is_tagged() {"*"} else {" "}, SKILLS[i], skill.total));
    }

    let block_w = (w - 300.0 * cfg.ui_scale) / 5.0;
    let off_1 = skill_cursor[0] + 230.0 * cfg.ui_scale;
    let off_2 = skill_cursor[0] + 230.0 * cfg.ui_scale + block_w + 8.0 * cfg.ui_scale;
    let off_3 = skill_cursor[0] + 230.0 * cfg.ui_scale + (block_w + 8.0 * cfg.ui_scale) * 2.0;
    let off_4 = skill_cursor[0] + 230.0 * cfg.ui_scale + (block_w + 8.0 * cfg.ui_scale) * 3.0;
    let off_5 = skill_cursor[0] + 230.0 * cfg.ui_scale + (block_w + 8.0 * cfg.ui_scale) * 4.0;

    let def_str = format!("Defense: {}", character.defense);
    let init_str = format!("Initiative: {}", character.initiative);
    let hp_str1 = format!("HP:");
    let hp_str2 = format!("{}/{}", character.hp, character.hp_max);
    let melee_str = format!("Melee: {}", get_melee_str(character));
    let poison_str = format!("Poison DR: {}", if character.poison_dr < 99 {character.poison_dr.to_string()} else {"Immune".to_string()});
    let def_size = ui.calc_text_size(def_str.clone());
    let init_size = ui.calc_text_size(init_str.clone());
    let hp_size = ui.calc_text_size(hp_str1.clone())[0] + ui.calc_text_size(hp_str2.clone())[0] + 20.0 * cfg.ui_scale;
    let melee_size = ui.calc_text_size(melee_str.clone());
    let poison_size = ui.calc_text_size(poison_str.clone());
    let new_line = def_size[1] + 8.0 * cfg.ui_scale;
    let pos_1 = [off_1 + (block_w - def_size[0]) / 2.0, skill_cursor[1] + new_line];
    let pos_3 = [off_3 + (block_w - init_size[0]) / 2.0, skill_cursor[1] + new_line];
    let pos_5 = [off_5 + (block_w - hp_size) / 2.0, skill_cursor[1] + new_line];
    let pos_2 = [off_2 + (block_w - melee_size[0]) / 2.0, skill_cursor[1] + new_line * 2.0];
    let pos_4 = [off_4 + (block_w - poison_size[0]) / 2.0, skill_cursor[1] + new_line * 2.0];
    ui.set_cursor_pos(pos_1);
    ui.text(def_str);
    ui.set_cursor_pos(pos_3);
    ui.text(init_str);
    ui.set_cursor_pos(pos_5);
    ui.text(hp_str1);
    ui.same_line();
    if ui.button("-##hp_dec") {
        character.hp -= 1;
        if character.hp < 0 {
            character.hp = 0;
        }
        db.update_hp(character);
    }
    ui.same_line();
    ui.text(hp_str2);
    ui.same_line();
    if ui.button("+##hp_inc") {
        character.hp += 1;
        if character.hp > character.hp_max {
            character.hp = character.hp_max;
        }
        db.update_hp(character);
    }
    ui.set_cursor_pos(pos_2);
    ui.text(melee_str);
    ui.set_cursor_pos(pos_4);
    ui.text(poison_str);

    let head_p1 = [off_3, skill_cursor[1] + new_line * 3.0];
    //let head_p2 = [off_3 + block_w, skill_cursor[1] + new_line * 6.0];
    let a1_p1 = [off_2, skill_cursor[1] + new_line * 6.5];
    //let a1_p2 = [off_2 + block_w, skill_cursor[1] + new_line * 9.5];
    let a2_p1 = [off_3, skill_cursor[1] + new_line * 6.5];
    //let a2_p2 = [off_3 + block_w, skill_cursor[1] + new_line * 9.5];
    let a3_p1 = [off_4, skill_cursor[1] + new_line * 6.5];
    //let a3_p2 = [off_4 + block_w, skill_cursor[1] + new_line * 9.5];
    let body_p1 = [off_3, skill_cursor[1] + new_line * 10.0];
    //let body_p2 = [off_3 + block_w, skill_cursor[1] + new_line * 13.0];
    let l1_p1 = [off_2, skill_cursor[1] + new_line * 13.5];
    //let l1_p2 = [off_2 + block_w, skill_cursor[1] + new_line * 16.5];
    let l2_p1 = [off_3, skill_cursor[1] + new_line * 13.5];
    //let l2_p2 = [off_3 + block_w, skill_cursor[1] + new_line * 16.5];
    let l3_p1 = [off_4, skill_cursor[1] + new_line * 13.5];
    //let l3_p2 = [off_4 + block_w, skill_cursor[1] + new_line * 16.5];

    let (head_str, head_dr, head_eq) = if character.robot == RobotType::Handy {
        let limb = character.limb_dr.optics.clone();
        let equipped: Vec<String> = limb.equipped.iter().map(|a| a.name.clone()).collect();
        (
            "Optics",
            format!("P: {}  E: {}  R: Immune", limb.ph_dr, limb.en_dr),
            equipped.join(", ")
        )
    } else {
        let limb = character.limb_dr.head.clone();
        let equipped: Vec<String> = limb.equipped.iter().map(|a| a.name.clone()).collect();
        (
            "Head",
            format!("P: {}  E: {}  R: {}", limb.ph_dr, limb.en_dr, if character.is_mutant() || character.is_robot() || character.ghoul {"Immune".to_string()} else { limb.rd_dr.to_string() }),
            equipped.join(", ")
        )
    };
    let head_size = ui.calc_text_size(head_str);
    let head_dr_size = ui.calc_text_size(head_dr.clone());
    let head_eq_size = ui.calc_text_size(head_eq.clone());
    let (a1_str, a1_dr, a1_eq) = if character.robot == RobotType::Handy {
        let limb = character.limb_dr.arm_1.clone();
        let equipped: Vec<String> = limb.equipped.iter().map(|a| a.name.clone()).collect();
        (
            "Arm 1",
            format!("P: {} E: {} R: Immune", limb.ph_dr, limb.en_dr),
            equipped.join(", ")
        )
    } else {
        let limb = character.limb_dr.arm_left.clone();
        let equipped: Vec<String> = limb.equipped.iter().map(|a| a.name.clone()).collect();
        (
            "Left Arm",
            format!("P: {} E: {} R: {}", limb.ph_dr, limb.en_dr, if character.is_mutant() || character.is_robot() || character.ghoul {"Immune".to_string()} else { limb.rd_dr.to_string() }),
            equipped.join(", ")
        )
    };
    let a1_size = ui.calc_text_size(a1_str);
    let a1_dr_size = ui.calc_text_size(a1_dr.clone());
    let a1_eq_size = ui.calc_text_size(a1_eq.clone());
    let (a2_str, a2_dr, a2_eq) = if character.robot == RobotType::Handy {
        let limb = character.limb_dr.arm_2.clone();
        let equipped: Vec<String> = limb.equipped.iter().map(|a| a.name.clone()).collect();
        (
            Some("Arm 2"),
            Some(format!("P: {} E: {} R: Immune", limb.ph_dr, limb.en_dr)),
            Some(equipped.join(", "))
        )} else { (None, None, None) };
    let a2_size = if a2_str.is_some() { Some(ui.calc_text_size(a2_str.clone().unwrap())) } else { None };
    let a2_dr_size = if a2_dr.is_some() { Some(ui.calc_text_size(a2_dr.clone().unwrap())) } else { None };
    let a2_eq_size = if a2_eq.is_some() { Some(ui.calc_text_size(a2_eq.clone().unwrap())) } else { None };
    let (a3_str, a3_dr, a3_eq) = if character.robot == RobotType::Handy {
        let limb = character.limb_dr.arm_3.clone();
        let equipped: Vec<String> = limb.equipped.iter().map(|a| a.name.clone()).collect();
        (
            "Arm 3",
            format!("P: {} E: {} R: Immune", limb.ph_dr, limb.en_dr),
            equipped.join(", ")
        )
    } else {
        let limb = character.limb_dr.arm_right.clone();
        let equipped: Vec<String> = limb.equipped.iter().map(|a| a.name.clone()).collect();
        (
            "Right Arm",
            format!("P: {} E: {} R: {}", limb.ph_dr, limb.en_dr, if character.is_mutant() || character.is_robot() || character.ghoul {"Immune".to_string()} else { limb.rd_dr.to_string() }),
            equipped.join(", ")
        )
    };
    let a3_size = ui.calc_text_size(a3_str);
    let a3_dr_size = ui.calc_text_size(a3_dr.clone());
    let a3_eq_size = ui.calc_text_size(a3_eq.clone());
    let (body_str, body_dr, body_eq) = if character.is_robot() {
        let limb = character.limb_dr.body.clone();
        let equipped: Vec<String> = limb.equipped.iter().map(|a| a.name.clone()).collect();
        (
            "Body",
            format!("P: {} E: {} R: Immune", limb.ph_dr, limb.en_dr),
            equipped.join(", ")
        )
    } else {
        let limb = character.limb_dr.torso.clone();
        let equipped: Vec<String> = limb.equipped.iter().map(|a| a.name.clone()).collect();
        (
            "Torso",
            format!("P: {} E: {} R: {}", limb.ph_dr, limb.en_dr, if character.is_mutant() || character.ghoul {"Immune".to_string()} else { limb.rd_dr.to_string() }),
            equipped.join(", ")
        )
    };
    let body_size = ui.calc_text_size(body_str);
    let body_dr_size = ui.calc_text_size(body_dr.clone());
    let body_eq_size = ui.calc_text_size(body_eq.clone());
    let (l1_str, l1_dr, l1_eq) = match character.robot {
        RobotType::Handy | RobotType::Securitron => (None, None, None),
        RobotType::Robobrain => {
            let limb = character.limb_dr.track_left.clone();
            let equipped: Vec<String> = limb.equipped.iter().map(|a| a.name.clone()).collect();
            (
                Some("Left Track"),
                Some(format!("P: {} E: {} R: Immune", limb.ph_dr, limb.en_dr)),
                Some(equipped.join(", "))
            )
        },
        _ => {
            let limb = character.limb_dr.leg_left.clone();
            let equipped: Vec<String> = limb.equipped.iter().map(|a| a.name.clone()).collect();
            (
                Some("Left Leg"),
                Some(format!("P: {} E: {} R: {}", limb.ph_dr, limb.en_dr, if character.is_mutant() || character.is_robot() || character.ghoul {"Immune".to_string()} else { limb.rd_dr.to_string() })),
                Some(equipped.join(", "))
            )
        }
    };
    let l1_size = if l1_str.is_some() { Some(ui.calc_text_size(l1_str.clone().unwrap()))} else { None };
    let l1_dr_size = if l1_dr.is_some() { Some(ui.calc_text_size(l1_dr.clone().unwrap()))} else { None };
    let l1_eq_size = if l1_eq.is_some() { Some(ui.calc_text_size(l1_eq.clone().unwrap()))} else { None };
    let (l2_str, l2_dr, l2_eq) = match character.robot {
        RobotType::Handy => {
            let limb = character.limb_dr.thruster.clone();
            let equipped: Vec<String> = limb.equipped.iter().map(|a| a.name.clone()).collect();
            (
                Some("Thruster"),
                Some(format!("P: {} E: {} R: Immune", limb.ph_dr, limb.en_dr)),
                Some(equipped.join(", "))
            )
        },
        RobotType::Securitron => {
            let limb = character.limb_dr.wheel.clone();
            let equipped: Vec<String> = limb.equipped.iter().map(|a| a.name.clone()).collect();
            (
                Some("Wheel"),
                Some(format!("P: {} E: {} R: Immune", limb.ph_dr, limb.en_dr)),
                Some(equipped.join(", "))
            )
        },
        _ => (None, None, None),
    };
    let l2_size = if l2_str.is_some() { Some(ui.calc_text_size(l2_str.clone().unwrap()))} else { None };
    let l2_dr_size = if l2_dr.is_some() { Some(ui.calc_text_size(l2_dr.clone().unwrap()))} else { None };
    let l2_eq_size = if l2_eq.is_some() { Some(ui.calc_text_size(l2_eq.clone().unwrap()))} else { None };
    let (l3_str, l3_dr, l3_eq) = match character.robot {
        RobotType::Handy | RobotType::Securitron => (None, None, None),
        RobotType::Robobrain => {
            let limb = character.limb_dr.track_right.clone();
            let equipped: Vec<String> = limb.equipped.iter().map(|a| a.name.clone()).collect();
            (
                Some("Right Track"),
                Some(format!("P: {} E: {} R: Immune", limb.ph_dr, limb.en_dr)),
                Some(equipped.join(", "))
            )
        },
        _ => {
            let limb = character.limb_dr.leg_right.clone();
            let equipped: Vec<String> = limb.equipped.iter().map(|a| a.name.clone()).collect();
            (
                Some("Right Leg"),
                Some(format!("P: {} E: {} R: {}", limb.ph_dr, limb.en_dr, if character.is_mutant() || character.is_robot() || character.ghoul {"Immune".to_string()} else { limb.rd_dr.to_string() })),
                Some(equipped.join(", "))
            )
        }
    };
    let l3_size = if l3_str.is_some() { Some(ui.calc_text_size(l3_str.clone().unwrap()))} else { None };
    let l3_dr_size = if l3_dr.is_some() { Some(ui.calc_text_size(l3_dr.clone().unwrap()))} else { None };
    let l3_eq_size = if l3_eq.is_some() { Some(ui.calc_text_size(l3_eq.clone().unwrap()))} else { None };

    ui.set_cursor_pos([head_p1[0] + (block_w - head_size[0]) / 2.0, head_p1[1]]);
    ui.text(head_str);
    ui.set_cursor_pos([head_p1[0] + (block_w - head_dr_size[0]) / 2.0, head_p1[1] + new_line]);
    ui.text(head_dr);
    ui.set_cursor_pos([head_p1[0] + (block_w - head_eq_size[0]) / 2.0, head_p1[1] + new_line * 2.0]);
    ui.text_disabled(head_eq);
    ui.set_cursor_pos([a1_p1[0] + (block_w - a1_size[0]) / 2.0, a1_p1[1]]);
    ui.text(a1_str);
    ui.set_cursor_pos([a1_p1[0] + (block_w - a1_dr_size[0]) / 2.0, a1_p1[1] + new_line]);
    ui.text(a1_dr);
    ui.set_cursor_pos([a1_p1[0] + (block_w - a1_eq_size[0]) / 2.0, a1_p1[1] + new_line * 2.0]);
    ui.text_disabled(a1_eq);
    if a2_str.is_some() {
        ui.set_cursor_pos([a2_p1[0] + (block_w - a2_size.unwrap()[0]) / 2.0, a2_p1[1]]);
        ui.text(a2_str.unwrap());
    }
    if a2_dr.is_some() {
        ui.set_cursor_pos([a2_p1[0] + (block_w - a2_dr_size.unwrap()[0]) / 2.0, a2_p1[1] + new_line]);
        ui.text(a2_dr.unwrap());
    }
    if a2_eq.is_some() {
        ui.set_cursor_pos([a2_p1[0] + (block_w - a2_eq_size.unwrap()[0]) / 2.0, a2_p1[1] + new_line * 2.0]);
        ui.text_disabled(a2_eq.unwrap());
    }
    ui.set_cursor_pos([a3_p1[0] + (block_w - a3_size[0]) / 2.0, a3_p1[1]]);
    ui.text(a3_str);
    ui.set_cursor_pos([a3_p1[0] + (block_w - a3_dr_size[0]) / 2.0, a3_p1[1] + new_line]);
    ui.text(a3_dr);
    ui.set_cursor_pos([a3_p1[0] + (block_w - a3_eq_size[0]) / 2.0, a3_p1[1] + new_line * 2.0]);
    ui.text_disabled(a3_eq);
    ui.set_cursor_pos([body_p1[0] + (block_w - body_size[0]) / 2.0, body_p1[1]]);
    ui.text(body_str);
    ui.set_cursor_pos([body_p1[0] + (block_w - body_dr_size[0]) / 2.0, body_p1[1] + new_line]);
    ui.text(body_dr);
    ui.set_cursor_pos([body_p1[0] + (block_w - body_eq_size[0]) / 2.0, body_p1[1] + new_line * 2.0]);
    ui.text_disabled(body_eq);
    if l1_str.is_some() {
        ui.set_cursor_pos([l1_p1[0] + (block_w - l1_size.unwrap()[0]) / 2.0, l1_p1[1]]);
        ui.text(l1_str.unwrap());
    }
    if l1_dr.is_some() {
        ui.set_cursor_pos([l1_p1[0] + (block_w - l1_dr_size.unwrap()[0]) / 2.0, l1_p1[1] + new_line]);
        ui.text(l1_dr.unwrap());
    }
    if l1_eq.is_some() {
        ui.set_cursor_pos([l1_p1[0] + (block_w - l1_eq_size.unwrap()[0]) / 2.0, l1_p1[1] + new_line * 2.0]);
        ui.text_disabled(l1_eq.unwrap());
    }
    if l2_str.is_some() {
        ui.set_cursor_pos([l2_p1[0] + (block_w - l2_size.unwrap()[0]) / 2.0, l2_p1[1]]);
        ui.text(l2_str.unwrap());
    }
    if l2_dr.is_some() {
        ui.set_cursor_pos([l2_p1[0] + (block_w - l2_dr_size.unwrap()[0]) / 2.0, l2_p1[1] + new_line]);
        ui.text(l2_dr.unwrap());
    }
    if l2_eq.is_some() {
        ui.set_cursor_pos([l2_p1[0] + (block_w - l2_eq_size.unwrap()[0]) / 2.0, l2_p1[1] + new_line * 2.0]);
        ui.text_disabled(l2_eq.unwrap());
    }
    if l3_str.is_some() {
        ui.set_cursor_pos([l3_p1[0] + (block_w - l3_size.unwrap()[0]) / 2.0, l3_p1[1]]);
        ui.text(l3_str.unwrap());
    }
    if l3_dr.is_some() {
        ui.set_cursor_pos([l3_p1[0] + (block_w - l3_dr_size.unwrap()[0]) / 2.0, l3_p1[1] + new_line]);
        ui.text(l3_dr.unwrap());
    }
    if l3_eq.is_some() {
        ui.set_cursor_pos([l3_p1[0] + (block_w - l3_eq_size.unwrap()[0]) / 2.0, l3_p1[1] + new_line * 2.0]);
        ui.text_disabled(l3_eq.unwrap());
    }

    /*
    let debug_color: [[f32; 4]; 8] = [
        [1.0, 0.0, 0.0, 1.0], //red
        [1.0, 0.5, 0.0, 1.0], //orange
        [1.0, 1.0, 0.0, 1.0], //yellow
        [0.0, 1.0, 0.0, 1.0], //green
        [0.0, 1.0, 1.0, 1.0], //cyan
        [0.0, 0.5, 1.0, 1.0], //blue
        [0.5, 0.0, 1.0, 1.0], //purple
        [1.0, 0.0, 1.0, 1.0]  //magenta
    ];
    let draw_list = ui.get_window_draw_list();
    draw_list.add_line(debug_skill_cursor, [debug_skill_cursor[0], debug_skill_cursor[1] + new_line], debug_color[7]).build();
    for (i, pose) in [pos_1, pos_2, pos_3, pos_4, pos_5].iter().enumerate() {
        let pos = [pose[0] + debug_diff[0], pose[1] + debug_diff[1]];
        draw_list.add_line(pos, [pos[0], pos[1] + new_line], debug_color[i]).build();
    }
    for (i, pose) in [off_1, off_2, off_3, off_4, off_5].iter().enumerate() {
        let pos = pose + debug_diff[0];
        draw_list.add_line([pos, debug_skill_cursor[1]], [pos, debug_skill_cursor[1] + new_line], debug_color[i]).build();
        draw_list.add_rect([pos, debug_skill_cursor[1]], [pos + block_w, debug_skill_cursor[1] + new_line], debug_color[i]).build();
    }
    draw_list.add_line(debug_skill_cursor, [debug_skill_cursor[0] + 268.0, debug_skill_cursor[1]], debug_color[5]).build();
    */

    ui.separator();
    ui.spacing();
    ui.separator();
    ui.spacing();

    render_weapons(ui, character.weapons.clone(), character, cfg);
    if ui.button("Add/Remove Weapons##weapons_open") {
        state.weapons_open = true;
        state.weapon_selected = None;
        state.weapon_filter = String::new();
        state.weapon_list = db.get_all_weapons().unwrap_or_default();
    }

    ui.separator();
    ui.spacing();
    ui.separator();
    ui.spacing();

    let inv_cursor = ui.cursor_pos().clone();

    ui.child_window("##inv_block")
        //not sure how we go about calculating this
        .size([290.0 * cfg.ui_scale, 400.0 * cfg.ui_scale])
        .border(false)
        .build(|| {
            render_inventory(ui, character.ammo.clone(), character.apparel.clone(), character.consumables.clone(), character.robot_modules.clone(), character.gear.clone(), character.junk.clone(), character.misc.clone(), character, db, cfg);
            if ui.button("Add/Remove Items##inv_open") {
                state.inventory.open = true;
                state.inventory.filter = String::new();
                state.inventory.all_apparel = db.get_all_apparel().unwrap_or_default();
                state.inventory.all_ammo = db.get_all_ammo().unwrap_or_default();
                state.inventory.all_consumables = db.get_all_consumables().unwrap_or_default();
                state.inventory.all_modules = db.get_all_robot_modules().unwrap_or_default();
                state.inventory.all_gear = db.get_all_gear().unwrap_or_default();
            }
    });

    let t_padding = 16.0 * cfg.ui_scale;
    let t_block_w = w - t_padding * 2.0 - 300.0 * cfg.ui_scale;
    let t_gap = 8.0 * cfg.ui_scale;
    let p_col_w = (t_block_w - t_gap) / 2.0;
    let t_col_w = if character.traits.len() > 1 { p_col_w } else { t_block_w };
    let t_collapse_h = 44.0 * cfg.ui_scale;
    let t_expanded_h = 160.0 * cfg.ui_scale;

    let trait_h = if state.traits_expanded { t_expanded_h } else { t_collapse_h };

    ui.set_cursor_pos([inv_cursor[0] + t_padding + 300.0 * cfg.ui_scale, inv_cursor[1]]);
    if character.traits.len() < 1 {
        state.traits_expanded = false;
        ui.text_disabled("no traits");
    } else {
        ui.child_window("##t_block")
            .size([t_block_w, trait_h])
            .border(false)
            .build(|| {
                render_expandable_block(
                    ui,
                    "##t1_col",
                    t_col_w,
                    trait_h,
                    &mut state.traits_expanded,
                    &["Trait:".to_string(),character.traits[0].name.clone()].join(" "),
                    Some(&character.traits[0].desc),
                    cfg,
                );
                if character.traits.len() > 1 {
                    ui.same_line_with_spacing(0.0, t_gap);
                    render_expandable_block(
                        ui,
                        "##t2_col",
                        t_col_w,
                        trait_h,
                        &mut state.traits_expanded,
                        &["Trait:".to_string(),character.traits[1].name.clone()].join(" "),
                        Some(&character.traits[1].desc),
                        cfg,
                    );
                }
        });
    }
    if character.perks.len() < 1 {
        ui.text_disabled("no perks");
    } else {
        for i in 0..(state.perks_expanded.len() + 1) / 2 {
            let perk1_h = if state.perks_expanded[i*2] {
                t_expanded_h
            } else { t_collapse_h };
            let perk2_h = if if character.perks.len() - (i*2+1) > 0 { state.perks_expanded[i*2 + 1] } else { false } {
                t_expanded_h
            } else {
                t_collapse_h
            };
            let perk_h = perk1_h.max(perk2_h);

            match i {
                0 => {
                    ui.set_cursor_pos([inv_cursor[0] + 300.0 * cfg.ui_scale + t_padding, inv_cursor[1] + trait_h + t_padding]);
                    ui.child_window(format!("##p_block_{}", i))
                        .size([t_block_w, perk_h])
                        .border(false)
                        .build(|| {
                            let mut p1_desc: Vec<String> = vec![];
                            for j in 0..character.perks[i*2].ranks as usize {
                                if character.perks[i*2].desc.len() == 1 {
                                    p1_desc.push(format!("{}: {}", j+1, character.perks[i*2].desc[0]));
                                } else {
                                    p1_desc.push(format!("{}: {}", j+1, character.perks[i*2].desc[j]));
                                }
                            }
                            render_expandable_block(
                                ui,
                                &format!("##p{}_col", i*2),
                                p_col_w,
                                perk1_h,
                                &mut state.perks_expanded[i*2],
                                &character.perks[i*2].name,
                                Some(&p1_desc.join("\n")),
                                cfg,
                            );
                            if character.perks.len() - (i*2+1) > 0 {
                                let mut p2_desc: Vec<String> = vec![];
                                for j in 0..character.perks[i*2+1].ranks as usize {
                                    if character.perks[i*2+1].desc.len() == 1 {
                                        p2_desc.push(format!("{}: {}", j+1, character.perks[i*2+1].desc[0]));
                                    } else {
                                        p2_desc.push(format!("{}: {}", j+1, character.perks[i*2+1].desc[j]));
                                    }
                                }
                                ui.same_line_with_spacing(0.0, t_gap);
                                render_expandable_block(
                                    ui,
                                    &format!("##p{}_col", i*2+1),
                                    p_col_w,
                                    perk2_h,
                                    &mut state.perks_expanded[i*2+1],
                                    &character.perks[i*2+1].name,
                                    Some(&p2_desc.join("\n")),
                                    cfg,
                                );
                            }
                    });
                }
                _ => {
                    let mut  curr_h = inv_cursor[1] + trait_h + t_padding;
                    for j in 0..i {
                        curr_h += t_padding;
                        if state.perks_expanded[j*2] || state.perks_expanded[j*2+1] {
                            curr_h += t_expanded_h;
                        } else {
                            curr_h += t_collapse_h;
                        }
                    }
                    ui.set_cursor_pos([inv_cursor[0] + 300.0 * cfg.ui_scale + t_padding, curr_h]);
                    ui.child_window(format!("##p_block_{}", i))
                        .size([t_block_w, perk_h])
                        .border(false)
                        .build(|| {
                            let mut p1_desc: Vec<String> = vec![];
                            for j in 0..character.perks[i*2].ranks as usize {
                                if character.perks[i*2].desc.len() == 1 {
                                    p1_desc.push(format!("{}: {}", j+1, character.perks[i*2].desc[0]));
                                } else {
                                    p1_desc.push(format!("{}: {}", j+1, character.perks[i*2].desc[j]));
                                }
                            }
                            render_expandable_block(
                                ui,
                                &format!("##p{}_col", i*2),
                                p_col_w,
                                perk1_h,
                                &mut state.perks_expanded[i*2],
                                &character.perks[i*2].name,
                                Some(&p1_desc.join("\n")),
                                cfg,
                            );
                            if character.perks.len() - (i*2+1) > 0 {
                                let mut p2_desc: Vec<String> = vec![];
                                for j in 0..character.perks[i*2+1].ranks as usize {
                                    if character.perks[i*2+1].desc.len() == 1 {
                                        p2_desc.push(format!("{}: {}", j+1, character.perks[i*2+1].desc[0]));
                                    } else {
                                        p2_desc.push(format!("{}: {}", j+1, character.perks[i*2+1].desc[j]));
                                    }
                                }
                                ui.same_line_with_spacing(0.0, t_gap);
                                render_expandable_block(
                                    ui,
                                    &format!("##p{}_col", i*2+1),
                                    p_col_w,
                                    perk2_h,
                                    &mut state.perks_expanded[i*2+1],
                                    &character.perks[i*2+1].name,
                                    Some(&p2_desc.join("\n")),
                                    cfg
                                );
                            }
                    });
                }
            }
        }
    }
    drop(_scroll);

    if state.notes_open {
        //render_overlay(ui, window);
        let (win_w, win_h) = window.size();
        ui.window("##overlay")
            .title_bar(false)
            .resizable(false)
            .movable(false)
            .scrollable(false)
            .size([win_w as f32, win_h as f32], imgui::Condition::Always)
            .position([0.0, 0.0], imgui::Condition::Always)
            .bg_alpha(0.6)
            //.no_inputs()
            .build(|| {
                //ui.invisible_button("##blocker_btn", [win_w as f32, win_h as f32]);
                let nw = 500.0 * cfg.ui_scale;
                let nh = 400.0 * cfg.ui_scale;
                ui.set_cursor_pos([(win_w as f32 - nw) * 0.5, (win_h as f32 - nh) * 0.5]);
                ui.child_window("##notes")
                    .size([nw, nh])
                    .border(true)
                    .build(|| {
                        ui.text("Notes");
                        let close_x = ui.content_region_avail()[0] - 20.0 * cfg.ui_scale;
                        ui.same_line_with_pos(close_x);
                        if ui.button("X##notes_close") {
                            state.notes_open = false;
                        }
                        ui.separator();
                        ui.spacing();

                        let text_h = nh - 96.0 * cfg.ui_scale;
                        ui.input_text_multiline(
                            "##notes_input",
                            &mut state.notes_buf,
                            [ui.content_region_avail()[0], text_h],
                        ).build();

                        ui.spacing();
                        ui.separator();
                        ui.spacing();

                        if ui.button("Save##notes_save") {
                            character.notes = state.notes_buf.to_string();
                            state.notes_open = false;
                            db.update_notes(character);
                        }
                        ui.same_line();
                        if ui.button("Cancel##notes_cancel") {
                            state.notes_open = false;
                            // buf is discarded without writing back to character.notes
                        }
                    });
            });

    }

    if state.xp_open {
        let (win_w, win_h) = window.size();
        ui.window("##overlay")
            .title_bar(false)
            .resizable(false)
            .movable(false)
            .scrollable(false)
            .size([win_w as f32, win_h as f32], imgui::Condition::Always)
            .position([0.0, 0.0], imgui::Condition::Always)
            .bg_alpha(0.6)
            .build(|| {
                let xw = 280.0 * cfg.ui_scale;
                let xh = 160.0 * cfg.ui_scale;
                ui.set_cursor_pos( [(win_w as f32 - xw) * 0.5, (win_h as f32 - xh) * 0.5]);
                ui.child_window("##xp")
                    .size([xw, xh])
                    .border(true)
                    .build(|| {
                        ui.text("Experience Points");
                        let close_x = ui.content_region_avail()[0] - 20.0 * cfg.ui_scale;
                        ui.same_line_with_pos(close_x);
                        if ui.button("X##xp_close") {
                            state.xp_open = false;
                        }
                        ui.separator();
                        ui.spacing();

                        ui.text(format!("Current XP: {}", character.xp));
                        ui.spacing();

                        ui.set_next_item_width(ui.content_region_avail()[0]);
                        ui.input_int("##xp_amount", &mut state.xp_amount).build();

                        ui.spacing();
                        ui.separator();
                        ui.spacing();

                        // Add button
                        let c = ui.push_style_color(imgui::StyleColor::Button, [0.1, 0.45, 0.1, 1.0]);
                        let c2 = ui.push_style_color(imgui::StyleColor::ButtonHovered, [0.15, 0.6, 0.15, 1.0]);
                        if ui.button_with_size("Add##xp_add", [80.0 * cfg.ui_scale, 0.0]) {
                            if state.xp_amount > 0 {
                                character.xp += state.xp_amount;
                                state.xp_open = false;
                                character.calculate_xp_next();
                                db.update_xp(character);
                            }
                        }
                        drop(c); drop(c2);

                        ui.same_line();

                        // Remove button
                        let c = ui.push_style_color(imgui::StyleColor::Button, [0.55, 0.1, 0.1, 1.0]);
                        let c2 = ui.push_style_color(imgui::StyleColor::ButtonHovered, [0.75, 0.15, 0.15, 1.0]);
                        if ui.button_with_size("Remove##xp_remove", [80.0 * cfg.ui_scale, 0.0]) {
                            if state.xp_amount > 0 {
                                character.xp = (character.xp - state.xp_amount).max(0);
                                state.xp_open = false;
                                character.calculate_xp_next();
                                db.update_xp(character);
                            }
                        }
                        drop(c); drop(c2);

                        ui.same_line();

                        if ui.button_with_size("Cancel##xp_cancel", [80.0 * cfg.ui_scale, 0.0]) {
                            state.xp_open = false;
                        }
                });
        });
    }
    if state.level {
        let (win_w, win_h) = window.size();
        ui.window("##overlay")
            .title_bar(false)
            .resizable(false)
            .movable(false)
            .scrollable(false)
            .size([win_w as f32, win_h as f32], imgui::Condition::Always)
            .position([0.0,0.0], imgui::Condition::Always)
            .bg_alpha(0.6)
            .build(|| {
                let lw = 750.0 * cfg.ui_scale;
                let lh = 600.0 * cfg.ui_scale;
                ui.set_cursor_pos([(win_w as f32 - lw) * 0.5, (win_h as f32 - lh) * 0.5]);
                ui.child_window("##levelup")
                    .size([lw, lh])
                    .border(true)
                    .build(|| {
                        let title = if state.up {
                            format!("Level Up → Lv {}", character.level + 1)
                        } else {
                            format!("Delevel → Lv {}", character.level - 1)
                        };
                        ui.text(&title);
                        ui.separator();
                        ui.spacing();

                        let skill_half = 300.0 * cfg.ui_scale;
                        let perk_half = lw - skill_half - 24.0 * cfg.ui_scale;
                        // ── Left: Skill selection ─────────────────────────────────
                        ui.child_window("##lu_skills")
                            .size([skill_half, lh - 80.0 * cfg.ui_scale])
                            .begin()
                            .map(|_child| {
                                ui.text(if state.up { "Increase a Skill" } else { "Reduce a Skill" });
                                ui.separator();
                                ui.spacing();
                                let mut char_clone = character.clone();
                                char_clone.level += if state.up { 1 } else { -1 };
                                char_clone.skills.apply_max(&char_clone.clone());
                                let skills = char_clone.skills.skill_block();
                                for (i, skill) in skills.iter().enumerate() {
                                    // on levelup: only show skills not at cap
                                    // on delevel: only show skills with at least 1 rank
                                    let eligible = if state.up {
                                        skill.total < skill.max
                                    } else {
                                        skill.total > 0
                                    };

                                    let label = format!(
                                        "{}: {}/{}##skill_{}",
                                        SKILLS[i], skill.total, skill.max, i
                                    );

                                    let is_sel = state.skill_choice == i as i32;
                                    let _d = (!eligible).then(|| ui.begin_disabled(true));

                                    // highlight selected row
                                    if is_sel {
                                        let c = ui.push_style_color(
                                            imgui::StyleColor::Header,
                                            [0.15, 0.35, 0.15, 0.6],
                                        );
                                        if ui.selectable_config(&label).selected(true).build() {
                                            state.skill_choice = i as i32;
                                        }
                                        drop(c);
                                    } else {
                                        if ui.selectable_config(&label).selected(false).build() {
                                            if eligible { state.skill_choice = i as i32; }
                                        }
                                    }
                                    drop(_d);
                                }
                            });

                        ui.same_line();

                        // ── Right: Perk selection ─────────────────────────────────
                        ui.child_window("##lu_perks")
                            .size([perk_half, lh - 80.0 * cfg.ui_scale])
                            .begin()
                            .map(|_child| {
                                ui.text(if state.up { "Choose a Perk" } else { "Remove a Perk" });
                                ui.separator();
                                ui.spacing();

                                if state.up {
                                    // levelup: show eligible perks not yet at cap
                                    let target_level = character.level + 1;
                                    for perk in &state.perks {
                                        let taken = character.perks.iter()
                                            .find(|p| p.id == perk.id)
                                            .map(|p| p.ranks)
                                            .unwrap_or(0);
                                        let at_cap = taken >= perk.ranks;
                                        let meets_level = target_level >= perk.level_req
                                            && (taken == 0 || target_level >= perk.level_req + taken * perk.rank_range);

                                        if at_cap || !meets_level { continue; }

                                        let label = format!(
                                            "{} ({}/{})##plu_{}",
                                            perk.name, taken, perk.ranks, perk.id
                                        );
                                        let is_sel = state.perk_choice == perk.id;

                                        if is_sel {
                                            let c = ui.push_style_color(
                                                imgui::StyleColor::Header,
                                                [0.15, 0.35, 0.15, 0.6],
                                            );
                                            if ui.selectable_config(&label).selected(true).build() {
                                                state.perk_choice = perk.id;
                                            }
                                            drop(c);
                                            // show description inline
                                            if !perk.description.is_empty() {
                                                let desc = if perk.description.len() > 1 {
                                                    perk.description.iter().enumerate()
                                                        .map(|(i, d)| format!("{}: {}", i + 1, d))
                                                        .collect::<Vec<_>>().join("\n")
                                                } else {
                                                    perk.description[0].clone()
                                                };
                                                let y = ui.cursor_pos()[1];
                                                ui.set_cursor_pos([8.0 * cfg.ui_scale, y]);
                                                let _d = ui.begin_disabled(true);
                                                ui.text_wrapped(&desc);
                                                drop(_d);
                                            }
                                        } else {
                                            if ui.selectable_config(&label).selected(false).build() {
                                                state.perk_choice = perk.id;
                                            }
                                        }
                                        ui.separator();
                                    }
                                } else {
                                    // delevel: only show perks the character actually has
                                    for cperk in &character.perks {
                                        let label = format!(
                                            "{} (rank {})##pld_{}",
                                            cperk.name, cperk.ranks, cperk.id
                                        );
                                        let is_sel = state.perk_choice == cperk.id;

                                        if is_sel {
                                            let c = ui.push_style_color(
                                                imgui::StyleColor::Header,
                                                [0.45, 0.1, 0.1, 0.6],
                                            );
                                            if ui.selectable_config(&label).selected(true).build() {
                                                state.perk_choice = cperk.id;
                                            }
                                            drop(c);
                                        } else {
                                            if ui.selectable_config(&label).selected(false).build() {
                                                state.perk_choice = cperk.id;
                                            }
                                        }
                                    }
                                }
                            });

                        // ── Footer ────────────────────────────────────────────────
                        ui.spacing();
                        ui.separator();
                        ui.spacing();

                        let ready = state.skill_choice != i32::MAX && state.perk_choice != i32::MAX;

                        let confirm_label = if state.up { "Confirm Level Up" } else { "Confirm Delevel" };
                        let btn_color = if state.up {
                            ([0.1, 0.45, 0.1, 1.0], [0.15, 0.6, 0.15, 1.0])
                        } else {
                            ([0.55, 0.1, 0.1, 1.0], [0.75, 0.15, 0.15, 1.0])
                        };

                        let _d = (!ready).then(|| ui.begin_disabled(true));
                        let c = ui.push_style_color(imgui::StyleColor::Button, btn_color.0);
                        let c2 = ui.push_style_color(imgui::StyleColor::ButtonHovered, btn_color.1);

                        if ui.button_with_size(confirm_label, [160.0 * cfg.ui_scale, 0.0]) {
                            apply_level_change(character, state, db);
                            character.calculate_xp_next();
                            state.level = false;
                        }
                        drop(c); drop(c2);
                        drop(_d);

                        ui.same_line();
                        if ui.button_with_size("Cancel##lu_cancel", [80.0 * cfg.ui_scale, 0.0]) {
                            state.level = false;
                            state.skill_choice = i32::MAX;
                            state.perk_choice = i32::MAX;
                        }

                        // hint when not ready
                        if !ready {
                            ui.same_line();
                            ui.text_disabled(if state.skill_choice == i32::MAX && state.perk_choice == i32::MAX {
                                "Select a skill and perk to continue"
                            } else if state.skill_choice == i32::MAX {
                                "Select a skill to continue"
                            } else {
                                "Select a perk to continue"
                            });
                        }

                    });
            });
    }
    if state.weapons_open {
        let (win_w, win_h) = window.size();
        ui.window("##overlay")
            .title_bar(false)
            .resizable(false)
            .movable(false)
            .scrollable(false)
            .size([win_w as f32, win_h as f32], imgui::Condition::Always)
            .position([0.0, 0.0], imgui::Condition::Always)
            .bg_alpha(0.6)
            .build(|| {
                let ww = 700.0 * cfg.ui_scale;
                let wh = 600.0 * cfg.ui_scale;
                ui.set_cursor_pos([(win_w as f32 - ww) * 0.5, (win_h as f32 - wh) * 0.5]);
                ui.child_window("##weapons_modal")
                    .size([ww, wh])
                    .border(true)
                    .build(|| {
                        // ── Header ───────────────────────────────────────────
                        ui.text("Weapons");
                        let close_x = ui.content_region_avail()[0] - 20.0 * cfg.ui_scale;
                        ui.same_line_with_pos(close_x);
                        if ui.button("X##weapons_close") {
                            state.weapons_open = false;
                        }
                        ui.separator();
                        ui.spacing();

                        let half = (ww - 16.0 * cfg.ui_scale) / 2.0;
                        let list_h = wh - 96.0 * cfg.ui_scale;

                        // ── Left: current inventory ───────────────────────────
                        ui.child_window("##weap_inv")
                            .size([half, list_h])
                            .begin()
                            .map(|_| {
                                ui.text("Inventory");
                                ui.separator();
                                ui.spacing();

                                if character.weapons.is_empty() {
                                    ui.text_disabled("No weapons");
                                }

                                let mut to_remove: Option<usize> = None;
                                for (i, w) in character.weapons.iter().enumerate() {
                                    let prefix = if w.prefix.is_empty() {
                                        String::new()
                                    } else {
                                        format!("{} ", w.prefix)
                                    };
                                    ui.text(format!("{}{}", prefix, w.name));
                                    ui.same_line_with_pos(half - 96.0 * cfg.ui_scale);
                                    ui.text_disabled(&w.range);

                                    ui.same_line_with_pos(half - 96.0 * cfg.ui_scale + 42.0 * cfg.ui_scale);
                                    let c = ui.push_style_color(imgui::StyleColor::Button, [0.55, 0.1, 0.1, 1.0]);
                                    let c2 = ui.push_style_color(imgui::StyleColor::ButtonHovered, [0.75, 0.15, 0.15, 1.0]);
                                    if ui.button(format!("Rem##wrem_{}", i)) {
                                        to_remove = Some(i);
                                    }
                                    drop(c); drop(c2);

                                    // show mods if any are installed
                                    let installed: Vec<&WeaponMods> = w.mods.iter()
                                        .filter(|m| m.installed && m.id != 0)
                                        .collect();
                                    if !installed.is_empty() {
                                        for m in &installed {
                                            let y = ui.cursor_pos()[1];
                                            ui.set_cursor_pos([12.0 * cfg.ui_scale, y]);
                                            ui.text_disabled(format!("↳ {}", m.name));
                                        }
                                    }
                                    ui.separator();
                                }

                                if let Some(idx) = to_remove {
                                    character.weapons.remove(idx);
                                    sync_derived_weapons(character, db);
                                }
                            });

                        ui.same_line();

                        // ── Right: db list ────────────────────────────────────
                        ui.child_window("##weap_db")
                            .size([half, list_h])
                            .begin()
                            .map(|_| {
                                ui.text("Add Weapon");
                                ui.separator();
                                ui.spacing();

                                // filter input
                                ui.set_next_item_width(half - 16.0 * cfg.ui_scale);
                                ui.input_text("##wfilter", &mut state.weapon_filter).hint("Filter...").build();
                                ui.spacing();

                                let filter = state.weapon_filter.to_lowercase();
                                let mut current_skill = String::new();

                                for (id, name, skill, _range) in &state.weapon_list {
                                    if !filter.is_empty()
                                        && !name.to_lowercase().contains(&filter)
                                        && !skill.to_lowercase().contains(&filter)
                                    {
                                        continue;
                                    }

                                    // skill group header
                                    if *skill != current_skill {
                                        if !current_skill.is_empty() { ui.spacing(); }
                                        ui.text_disabled(skill);
                                        ui.separator();
                                        current_skill = skill.clone();
                                    }

                                    let is_sel = state.weapon_selected == Some(*id);
                                    let already_owned = character.weapons.iter().any(|w| w.id == *id);

                                    if already_owned {
                                        ui.text_disabled(format!("  {} [owned]", name));
                                    } else if is_sel {
                                        let c = ui.push_style_color(
                                            imgui::StyleColor::Header,
                                            [0.15, 0.35, 0.15, 0.6],
                                        );
                                        ui.selectable_config(format!("  {}##wsel_{}", name, id))
                                            .selected(true)
                                            .build();
                                        drop(c);
                                    } else {
                                        if ui.selectable_config(format!("  {}##wsel_{}", name, id))
                                            .selected(false)
                                            .build()
                                        {
                                            state.weapon_selected = Some(*id);
                                        }
                                    }
                                }
                            });

                        // ── Footer ────────────────────────────────────────────
                        ui.spacing();
                        ui.separator();
                        ui.spacing();

                        let _d = state.weapon_selected.is_none().then(|| ui.begin_disabled(true));
                        let c = ui.push_style_color(imgui::StyleColor::Button, [0.1, 0.45, 0.1, 1.0]);
                        let c2 = ui.push_style_color(imgui::StyleColor::ButtonHovered, [0.15, 0.6, 0.15, 1.0]);
                        if ui.button_with_size("Add Selected##weap_add", [140.0 * cfg.ui_scale, 0.0]) {
                            if let Some(wid) = state.weapon_selected {
                                match db.get_weapon_by_id(wid, character) {
                                    Ok(w) => {
                                        character.weapons.push(w);
                                        state.weapon_selected = None;
                                        sync_derived_weapons(character, db);
                                    }
                                    Err(e) => eprintln!("Failed to load weapon {}: {}", wid, e),
                                }
                            }
                        }
                        drop(c); drop(c2);
                        drop(_d);

                        ui.same_line();
                        if ui.button_with_size("Done##weap_done", [80.0 * cfg.ui_scale, 0.0]) {
                            state.weapons_open = false;
                            match db.save_character(character) {
                                Ok(_) => {},
                                Err(e) => eprintln!("Failed to save character: {e}"),
                            }
                        }

                        if state.weapon_selected.is_none() {
                            ui.same_line();
                            ui.text_disabled("Select a weapon from the list to add it");
                        }
                    });
            });

    }
    if state.inventory.open {
        let (win_w, win_h) = window.size();
        ui.window("##overlay")
            .title_bar(false)
            .resizable(false)
            .movable(false)
            .scrollable(false)
            .size([win_w as f32, win_h as f32], imgui::Condition::Always)
            .position([0.0, 0.0], imgui::Condition::Always)
            .bg_alpha(0.6)
            .build(|| {
                let iw = 960.0 * cfg.ui_scale;
                let ih = 600.0 * cfg.ui_scale;
                let inv = &mut state.inventory;
                ui.set_cursor_pos([(win_w as f32 - iw) * 0.5, (win_h as f32 - ih) * 0.5]);
                ui.child_window("##inv_modal")
                    .size([iw, ih])
                    .border(true)
                    .build(|| {
                        // ── Header ───────────────────────────────────────
                        ui.text("Inventory");
                        let close_x = ui.content_region_avail()[0] - 20.0 * cfg.ui_scale;
                        ui.same_line_with_pos(close_x);
                        if ui.button("X##inv_close") { inv.open = false; }
                        ui.separator();
                        ui.spacing();

                        // ── Tabs ─────────────────────────────────────────
                        let tabs = [
                            ("Ammo",         InventoryTab::Ammo),
                            ("Apparel",      InventoryTab::Apparel),
                            ("Consumables",  InventoryTab::Consumables),
                            ("Modules",      InventoryTab::RobotModules),
                            ("Gear",         InventoryTab::Gear),
                            ("Misc",         InventoryTab::Misc),
                        ];
                        for (label, tab) in &tabs {
                            let active = inv.tab == *tab;
                            if active {
                                let c = ui.push_style_color(imgui::StyleColor::Button, [0.2, 0.4, 0.2, 1.0]);
                                ui.button(label);
                                drop(c);
                            } else {
                                if ui.button(label) {
                                    inv.tab = tab.clone();
                                    inv.filter = String::new();
                                    inv.apparel_type_filter = None;
                                }
                            }
                            ui.same_line();
                        }
                        ui.new_line();
                        ui.separator();
                        ui.spacing();

                        let half = (iw - 16.0 * cfg.ui_scale) / 2.0;
                        let list_h = ih - 136.0 * cfg.ui_scale;

                        match inv.tab {

                            // ── AMMO ─────────────────────────────────────
                            InventoryTab::Ammo => {
                                ui.child_window("##ammo_inv").size([half, list_h]).begin().map(|_| {
                                    ui.text("Inventory");
                                    ui.separator(); ui.spacing();
                                    if character.ammo.is_empty() { ui.text_disabled("No ammo"); }
                                    let mut to_remove = None;
                                    for (i, a) in character.ammo.iter_mut().enumerate() {
                                        ui.text(format!("{}x {}", a.quantity, a.ammo.name));
                                        ui.same_line_with_pos(half - 100.0 * cfg.ui_scale);
                                        if ui.button(format!("-##ammo_dec_{}", i)) {
                                            a.quantity -= 1;
                                        }
                                        ui.same_line();
                                        if ui.button(format!("+##ammo_inc_{}", i)) {
                                            a.quantity += 1;
                                        }
                                        ui.same_line();
                                        let c  = ui.push_style_color(imgui::StyleColor::Button, [0.55, 0.1, 0.1, 1.0]);
                                        let c2 = ui.push_style_color(imgui::StyleColor::ButtonHovered, [0.75, 0.15, 0.15, 1.0]);
                                        if ui.button(format!("Remove##ammo_rem_{}", i)) { to_remove = Some(i); }
                                        drop(c); drop(c2);
                                        ui.separator();
                                    }
                                    if let Some(i) = to_remove { character.ammo.remove(i); }
                                });
                                ui.same_line();
                                ui.child_window("##ammo_db").size([half, list_h]).begin().map(|_| {
                                    ui.text("Add Ammo");
                                    ui.separator(); ui.spacing();
                                    ui.set_next_item_width(half - 16.0 * cfg.ui_scale);
                                    ui.input_text("##ammo_filter", &mut inv.filter).hint("Filter...").build();
                                    ui.text("Qty:"); ui.same_line();
                                    ui.set_next_item_width(60.0 * cfg.ui_scale);
                                    ui.input_int("##ammo_qty", &mut inv.ammo_qty).build();
                                    inv.ammo_qty = inv.ammo_qty.max(1);
                                    ui.spacing();
                                    let filter = inv.filter.to_lowercase();
                                    for a in &inv.all_ammo {
                                        if !filter.is_empty() && !a.name.to_lowercase().contains(&filter) { continue; }
                                        let already = character.ammo.iter().any(|ca| ca.ammo.id == a.id);
                                        let label = format!("{}##ammo_add_{}", a.name, a.id);
                                        let _d = already.then(|| ui.begin_disabled(true));
                                        if already {
                                            ui.text_disabled(format!("{} [in inventory]", a.name));
                                        } else if ui.selectable_config(&label).build() {
                                            character.ammo.push(AmmoInv {
                                                ammo: a.clone(),
                                                quantity: inv.ammo_qty,
                                            });
                                        }
                                        drop(_d);
                                    }
                                });
                            }

                            // ── APPAREL ───────────────────────────────────
                            InventoryTab::Apparel => {
                                ui.child_window("##app_inv").size([half, list_h]).begin().map(|_| {
                                    ui.text("Inventory");
                                    ui.separator(); ui.spacing();
                                    if character.apparel.is_empty() { ui.text_disabled("No apparel"); }
                                    let mut to_remove = None;
                                    for (i, a) in character.apparel.iter().enumerate() {
                                        let type_str = match a.apparel_type {
                                            ApparelType::Clothing   => "Clothing",
                                            ApparelType::Outfit     => "Outfit",
                                            ApparelType::Headgear   => "Headgear",
                                            ApparelType::Armor      => "Armor",
                                            ApparelType::PowerArmor => "Power Armor",
                                            ApparelType::RobotArmor => "Robot Armor",
                                        };
                                        ui.text(format!("[{}] {}", type_str, a.name));
                                        ui.same_line_with_pos(half - 64.0 * cfg.ui_scale);
                                        let c  = ui.push_style_color(imgui::StyleColor::Button, [0.55, 0.1, 0.1, 1.0]);
                                        let c2 = ui.push_style_color(imgui::StyleColor::ButtonHovered, [0.75, 0.15, 0.15, 1.0]);
                                        if ui.button(format!("Remove##app_rem_{}", i)) { to_remove = Some(i); }
                                        drop(c); drop(c2);
                                        ui.separator();
                                    }
                                    if let Some(i) = to_remove { character.apparel.remove(i); }
                                });
                                ui.same_line();
                                ui.child_window("##app_db").size([half, list_h]).begin().map(|_| {
                                    ui.text("Add Apparel");
                                    ui.separator(); ui.spacing();
                                    ui.set_next_item_width(half - 16.0 * cfg.ui_scale);
                                    ui.input_text("##app_filter", &mut inv.filter).hint("Filter...").build();
                                    ui.spacing();

                                    // type filter buttons
                                    let type_filters: &[(&str, Option<ApparelType>)] = &[
                                        ("All",         None),
                                        ("Clothing",    Some(ApparelType::Clothing)),
                                        ("Outfit",      Some(ApparelType::Outfit)),
                                        ("Headgear",    Some(ApparelType::Headgear)),
                                        ("Armor",       Some(ApparelType::Armor)),
                                        ("Power",       Some(ApparelType::PowerArmor)),
                                        ("Robot",       Some(ApparelType::RobotArmor)),
                                    ];
                                    for (label, filter_type) in type_filters {
                                        let active = inv.apparel_type_filter == *filter_type;
                                        if active {
                                            let c = ui.push_style_color(imgui::StyleColor::Button, [0.2, 0.35, 0.2, 1.0]);
                                            ui.button(label);
                                            drop(c);
                                        } else if ui.button(label) {
                                            inv.apparel_type_filter = filter_type.clone();
                                        }
                                        ui.same_line();
                                    }
                                    ui.new_line();
                                    ui.spacing();

                                    let filter = inv.filter.to_lowercase();
                                    let mut current_type = String::new();
                                    for a in &inv.all_apparel {
                                        if let Some(ref tf) = inv.apparel_type_filter {
                                            if a.apparel_type != *tf { continue; }
                                        }
                                        if !filter.is_empty() && !a.name.to_lowercase().contains(&filter) { continue; }

                                        let type_str = match a.apparel_type {
                                            ApparelType::Clothing   => "Clothing",
                                            ApparelType::Outfit     => "Outfit",
                                            ApparelType::Headgear   => "Headgear",
                                            ApparelType::Armor      => "Armor",
                                            ApparelType::PowerArmor => "Power Armor",
                                            ApparelType::RobotArmor => "Robot Armor",
                                        };
                                        if type_str != current_type {
                                            if !current_type.is_empty() { ui.spacing(); }
                                            ui.text_disabled(type_str);
                                            ui.separator();
                                            current_type = type_str.to_string();
                                        }

                                        let already = character.apparel.iter().any(|ca| ca.id == a.id);
                                        if already {
                                            ui.text_disabled(format!("  {} [owned]", a.name));
                                        } else if ui.selectable_config(
                                            format!("  {}##app_add_{}", a.name, a.id)
                                        ).build() {
                                            character.apparel.push(a.clone());
                                        }
                                    }
                                });
                            }

                            // ── CONSUMABLES ───────────────────────────────
                            InventoryTab::Consumables => {
                                ui.child_window("##con_inv").size([half, list_h]).begin().map(|_| {
                                    ui.text("Inventory");
                                    ui.separator(); ui.spacing();
                                    if character.consumables.is_empty() { ui.text_disabled("No consumables"); }
                                    let mut to_remove = None;
                                    for (i, c) in character.consumables.iter_mut().enumerate() {
                                        ui.text(format!("{}x {}", c.quantity, c.name));
                                        ui.same_line_with_pos(half - 104.0 * cfg.ui_scale);
                                        if ui.button(format!("-##con_dec_{}", i)) { c.quantity = (c.quantity - 1).max(1); }
                                        ui.same_line();
                                        if ui.button(format!("+##con_inc_{}", i)) { c.quantity += 1; }
                                        ui.same_line();
                                        let c2  = ui.push_style_color(imgui::StyleColor::Button, [0.55, 0.1, 0.1, 1.0]);
                                        let c3 = ui.push_style_color(imgui::StyleColor::ButtonHovered, [0.75, 0.15, 0.15, 1.0]);
                                        if ui.button(format!("Remove##con_rem_{}", i)) { to_remove = Some(i); }
                                        drop(c2); drop(c3);
                                        ui.separator();
                                    }
                                    if let Some(i) = to_remove { character.consumables.remove(i); }
                                });
                                ui.same_line();
                                ui.child_window("##con_db").size([half, list_h]).begin().map(|_| {
                                    ui.text("Add Consumable");
                                    ui.separator(); ui.spacing();
                                    ui.set_next_item_width(half - 16.0 * cfg.ui_scale);
                                    ui.input_text("##con_filter", &mut inv.filter).hint("Filter...").build();
                                    ui.spacing();
                                    let filter = inv.filter.to_lowercase();
                                    let mut current_type = String::new();
                                    for c in &inv.all_consumables {
                                        if !filter.is_empty() && !c.name.to_lowercase().contains(&filter) { continue; }
                                        let type_str = match c.consumable_type {
                                            ConsumableType::Chem        => "Chems",
                                            ConsumableType::Food        => "Food",
                                            ConsumableType::Beverage    => "Beverages",
                                            ConsumableType::Publication => "Publications",
                                            ConsumableType::Other       => "Other",
                                        };
                                        if type_str != current_type {
                                            if !current_type.is_empty() { ui.spacing(); }
                                            ui.text_disabled(type_str);
                                            ui.separator();
                                            current_type = type_str.to_string();
                                        }
                                        let already = character.consumables.iter().any(|cc| cc.id == c.id);
                                        if already {
                                            ui.text_disabled(format!("  {} [in inventory]", c.name));
                                        } else if ui.selectable_config(
                                            format!("  {}##con_add_{}", c.name, c.id)
                                        ).build() {
                                            character.consumables.push(c.clone());
                                        }
                                    }
                                });
                            }

                            // ── ROBOT MODULES ─────────────────────────────
                            InventoryTab::RobotModules => {
                                ui.child_window("##mod_inv").size([half, list_h]).begin().map(|_| {
                                    ui.text("Inventory");
                                    ui.separator(); ui.spacing();
                                    if character.robot_modules.is_empty() { ui.text_disabled("No modules"); }
                                    let mut to_remove = None;
                                    for (i, m) in character.robot_modules.iter().enumerate() {
                                        let inst = if m.installed { " [installed]" } else { "" };
                                        ui.text(format!("{}{}", m.name, inst));
                                        ui.same_line_with_pos(half - 64.0 * cfg.ui_scale);
                                        let c  = ui.push_style_color(imgui::StyleColor::Button, [0.55, 0.1, 0.1, 1.0]);
                                        let c2 = ui.push_style_color(imgui::StyleColor::ButtonHovered, [0.75, 0.15, 0.15, 1.0]);
                                        if ui.button(format!("Remove##mod_rem_{}", i)) { to_remove = Some(i); }
                                        drop(c); drop(c2);
                                        ui.separator();
                                    }
                                    if let Some(i) = to_remove { character.robot_modules.remove(i); }
                                });
                                ui.same_line();
                                ui.child_window("##mod_db").size([half, list_h]).begin().map(|_| {
                                    ui.text("Add Module");
                                    ui.separator(); ui.spacing();
                                    ui.set_next_item_width(half - 16.0 * cfg.ui_scale);
                                    ui.input_text("##mod_filter", &mut inv.filter).hint("Filter...").build();
                                    ui.spacing();
                                    let filter = inv.filter.to_lowercase();
                                    for m in &inv.all_modules {
                                        if !filter.is_empty() && !m.name.to_lowercase().contains(&filter) { continue; }
                                        let already = character.robot_modules.iter().any(|cm| cm.id == m.id);
                                        if already {
                                            ui.text_disabled(format!("{} [owned]", m.name));
                                        } else if ui.selectable_config(
                                            format!("{}##mod_add_{}", m.name, m.id)
                                        ).build() {
                                            character.robot_modules.push(m.clone());
                                        }
                                    }
                                });
                            }

                            // ── GEAR ──────────────────────────────────────
                            InventoryTab::Gear => {
                                ui.child_window("##gear_inv").size([half, list_h]).begin().map(|_| {
                                    ui.text("Inventory");
                                    ui.separator(); ui.spacing();
                                    if character.gear.is_empty() { ui.text_disabled("No gear"); }
                                    let mut to_remove = None;
                                    for (i, g) in character.gear.iter_mut().enumerate() {
                                        ui.text(format!("{}x {}", g.quantity, g.name));
                                        ui.same_line_with_pos(half - 104.0 * cfg.ui_scale);
                                        if ui.button(format!("-##gear_dec_{}", i)) { g.quantity = (g.quantity - 1).max(1); }
                                        ui.same_line();
                                        if ui.button(format!("+##gear_inc_{}", i)) { g.quantity += 1; }
                                        ui.same_line();
                                        let c  = ui.push_style_color(imgui::StyleColor::Button, [0.55, 0.1, 0.1, 1.0]);
                                        let c2 = ui.push_style_color(imgui::StyleColor::ButtonHovered, [0.75, 0.15, 0.15, 1.0]);
                                        if ui.button(format!("Remove##gear_rem_{}", i)) { to_remove = Some(i); }
                                        drop(c); drop(c2);
                                        ui.separator();
                                    }
                                    if let Some(i) = to_remove { character.gear.remove(i); }
                                });
                                ui.same_line();
                                ui.child_window("##gear_db").size([half, list_h]).begin().map(|_| {
                                    ui.text("Add Gear");
                                    ui.separator(); ui.spacing();
                                    ui.set_next_item_width(half - 16.0 * cfg.ui_scale);
                                    ui.input_text("##gear_filter", &mut inv.filter).hint("Filter...").build();
                                    ui.spacing();
                                    let filter = inv.filter.to_lowercase();
                                    for g in &inv.all_gear {
                                        if !filter.is_empty() && !g.name.to_lowercase().contains(&filter) { continue; }
                                        let already = character.gear.iter().any(|cg| cg.id == g.id);
                                        if already {
                                            ui.text_disabled(format!("{} [in inventory]", g.name));
                                        } else if ui.selectable_config(
                                            format!("{}##gear_add_{}", g.name, g.id)
                                        ).build() {
                                            character.gear.push(g.clone());
                                        }
                                    }
                                });
                            }

                            // ── MISC ──────────────────────────────────────
                            InventoryTab::Misc => {
                                ui.child_window("##misc_inv").size([half, list_h]).begin().map(|_| {
                                    ui.text("Misc Items");
                                    ui.separator(); ui.spacing();
                                    if character.misc.is_empty() { ui.text_disabled("No misc items"); }
                                    let mut to_remove = None;
                                    for (i, m) in character.misc.iter().enumerate() {
                                        ui.text(m);
                                        ui.same_line_with_pos(half - 64.0 * cfg.ui_scale);
                                        let c  = ui.push_style_color(imgui::StyleColor::Button, [0.55, 0.1, 0.1, 1.0]);
                                        let c2 = ui.push_style_color(imgui::StyleColor::ButtonHovered, [0.75, 0.15, 0.15, 1.0]);
                                        if ui.button(format!("Remove##misc_rem_{}", i)) { to_remove = Some(i); }
                                        drop(c); drop(c2);
                                        ui.separator();
                                    }
                                    if let Some(i) = to_remove { character.misc.remove(i); }
                                });
                                ui.same_line();
                                ui.child_window("##misc_add").size([half, list_h]).begin().map(|_| {
                                    ui.text("Add Item");
                                    ui.separator(); ui.spacing();
                                    ui.text("Item name:");
                                    ui.set_next_item_width(half - 16.0 * cfg.ui_scale);
                                    ui.input_text("##misc_buf", &mut inv.misc_buf).build();
                                    ui.spacing();
                                    let empty = inv.misc_buf.trim().is_empty();
                                    let _d = empty.then(|| ui.begin_disabled(true));
                                    let c  = ui.push_style_color(imgui::StyleColor::Button, [0.1, 0.45, 0.1, 1.0]);
                                    let c2 = ui.push_style_color(imgui::StyleColor::ButtonHovered, [0.15, 0.6, 0.15, 1.0]);
                                    if ui.button("Add##misc_add_btn") {
                                        let trimmed = inv.misc_buf.trim().to_string();
                                        if !trimmed.is_empty() {
                                            character.misc.push(trimmed);
                                            inv.misc_buf = String::new();
                                        }
                                    }
                                    drop(c); drop(c2);
                                    drop(_d);
                                });
                            }
                        }

                        // ── Footer ────────────────────────────────────────
                        ui.spacing();
                        ui.separator();
                        ui.spacing();
                        if ui.button_with_size("Done##inv_done", [80.0 * cfg.ui_scale, 0.0]) {
                            inv.open = false;
                            // db.save_character(character).ok();
                        }
                    });
            });
    }

    h
}