
use imgui::Ui;
use sdl2::video::Window;
use std::path::Path;
use anyhow::Result;
use crate::{
    AppScreen,
    character::{Character, RobotType},
    db::Db,
    log_on_change,
    screens::{
        background_select::{BackgroundState, EquipmentState}, character_review::{render_inventory, render_weapons}, origin_select::OriginState, perk_select::PerkState, skill_assignment::{SKILLS, SkillState}, special_assignment::{SPECIAL_LABELS, SpecialState}, stat_calculation::get_melee_str
    },
    theme::render_window
};

pub struct SheetState {
    origin_expanded: bool,
    background_expanded: bool,
    traits_expanded: bool,
    perks_expanded: Vec<bool>,
}

impl SheetState {
    pub fn new() -> Self {
        Self {
            origin_expanded: false,
            background_expanded: false,
            traits_expanded: false,
            perks_expanded: vec![],
        }
    }
    pub fn new_character(&mut self, character: &Character) {
        self.perks_expanded = character.perks.iter().map(|_| false).collect();
    }
}

pub fn export_character(character: &Character, path: &Path) -> Result<()> {
    let json = serde_json::to_string_pretty(character)?;
    std::fs::write(path, json)?;
    Ok(())
}

pub fn sanitize_filename(name: &str) -> String {
    let reserved_names = [
        "CON", "PRN", "AUX", "NUL",
        "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
        "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    let sanitized: String = name.chars().map(|c| match c {
        '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
        c if c.is_control() => '_',
        ' ' => '_',
        c => c,
    }).collect();
    let sanitized = sanitized.trim_matches('.').to_string();
    let upper = sanitized.to_uppercase();
    let base = upper.split('.').next().unwrap_or("");
    if sanitized.is_empty() || reserved_names.contains(&base) {
        "character".to_string()
    } else {
        sanitized
    }
}

pub fn render_expandable_block(
    ui: &Ui,
    id: &str,
    w: f32,
    h: f32,
    expanded: &mut bool,
    title: &str,
    contents: Option<&str>,
) {
    ui.child_window(id)
        .size([w,h])
        .border(true)
        .build(|| {
            ui.set_cursor_pos([8.0, 8.0]);

            let arrow = if *expanded { "v" } else { ">" };
            let header = format!("{} {}##hdr_{}", arrow, title, id);
            let c1 = ui.push_style_color(
                imgui::StyleColor::Button,
                ui.style_color(imgui::StyleColor::ChildBg)
            );
            let c2 = ui.push_style_color(
                imgui::StyleColor::ButtonHovered,
                ui.style_color(imgui::StyleColor::FrameBgHovered)
            );
            let c3 = ui.push_style_color(
                imgui::StyleColor::ButtonActive,
                ui.style_color(imgui::StyleColor::FrameBgActive)
            );
            if ui.button_with_size(&header, [w - 16.0, 28.0]) {
                *expanded = !*expanded;
            }
            drop(c1);
            drop(c2);
            drop(c3);

            if *expanded {
                ui.spacing();
                ui.separator();
                ui.spacing();

                let desc = contents.unwrap_or("no description");
                let text_w = w - 24.0;
                ui.set_next_item_width(text_w);
                ui.text_wrapped(desc);
            }
        });
}

pub fn render_character_sheet(
    ui: &Ui,
    window: &Window,
    _db: &Db,
    character: &mut Character,
    screen: &mut AppScreen,
    state: &mut SheetState,
    origin: &mut OriginState,
    special: &mut SpecialState,
    skill: &mut SkillState,
    perk: &mut PerkState,
    background: &mut BackgroundState,
    equipment: &mut EquipmentState,
) -> f32 {
    let Some((w, h, _token)) = render_window(ui, window, "##character_sheet", "Character Sheet", screen, origin, special,  skill, perk, background, equipment, character)
        else { return 0.0 };

    log_on_change!(character);

    //could probably do columns here with wrapping
    ui.text(format!("{} --- {} ({})", character.name, character.player.name, character.party.name));
    ui.same_line();
    ui.text(format!("                {:4}xp ({} to next)  Lv {}  |here you would add xp|", character.xp, character.xp_next, character.level));
    ui.same_line_with_pos(w - 80.0);
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
        .size([w - 16.0, h - 84.0])
        .begin()
    else { return h };
    
    let o_padding = 16.0_f32;
    let o_block_w = w - o_padding * 2.0;
    let o_gap = 8.0_f32;
    let o_col_w = (o_block_w - o_gap) / 2.0;
    let o_collapse_h = 44.0_f32;
    let o_expanded_h = 160.0_f32;

    let origin_h = if state.origin_expanded { o_expanded_h } else { o_collapse_h };
    let background_h = if state.background_expanded { o_expanded_h } else { o_collapse_h };
    let total_h = origin_h.max(background_h) + 16.0;

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
        }
    }
    ui.same_line();
    ui.text(format!("{}/{}", character.luck_points, character.luck_points_max));
    ui.same_line();
    if ui.button("+##lp_inc") {
        if character.luck_points < character.luck_points_max {
            character.luck_points += 1;
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
            }
        }
        ui.same_line();
        ui.text(format!("{}/5", character.rad_points));
        ui.same_line();
        if ui.button("+##rp_inc") {
            if character.rad_points < 5 {
                character.rad_points += 1;
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

    let block_w = (w - 300.0) / 5.0;
    let off_1 = skill_cursor[0] + 230.0_f32;
    let off_2 = skill_cursor[0] + 230.0 + block_w + 8.0;
    let off_3 = skill_cursor[0] + 230.0 + (block_w + 8.0) * 2.0;
    let off_4 = skill_cursor[0] + 230.0 + (block_w + 8.0) * 3.0;
    let off_5 = skill_cursor[0] + 230.0 + (block_w + 8.0) * 4.0;

    let def_str = format!("Defense: {}", character.defense);
    let init_str = format!("Initiative: {}", character.initiative);
    let hp_str = format!("HP: {}/{}", character.hp, character.hp_max);
    let melee_str = format!("Melee: {}", get_melee_str(character));
    let poison_str = format!("Poison DR: {}", if character.poison_dr < 99 {character.poison_dr.to_string()} else {"Immune".to_string()});
    let def_size = ui.calc_text_size(def_str.clone());
    let init_size = ui.calc_text_size(init_str.clone());
    let hp_size = ui.calc_text_size(hp_str.clone());
    let melee_size = ui.calc_text_size(melee_str.clone());
    let poison_size = ui.calc_text_size(poison_str.clone());
    let new_line = def_size[1] + 8.0;
    let pos_1 = [off_1 + (block_w - def_size[0]) / 2.0, skill_cursor[1] + new_line];
    let pos_3 = [off_3 + (block_w - init_size[0]) / 2.0, skill_cursor[1] + new_line];
    let pos_5 = [off_5 + (block_w - hp_size[0]) / 2.0, skill_cursor[1] + new_line];
    let pos_2 = [off_2 + (block_w - melee_size[0]) / 2.0, skill_cursor[1] + new_line * 2.0];
    let pos_4 = [off_4 + (block_w - poison_size[0]) / 2.0, skill_cursor[1] + new_line * 2.0];
    ui.set_cursor_pos(pos_1);
    ui.text(def_str);
    ui.set_cursor_pos(pos_3);
    ui.text(init_str);
    ui.set_cursor_pos(pos_5);
    ui.text(hp_str);
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
        ui.text(a2_eq.unwrap());
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
    ui.text(body_eq);
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
        ui.text(l1_eq.unwrap());
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
        ui.text(l2_eq.unwrap());
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
        ui.text(l3_eq.unwrap());
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

    render_weapons(ui, character.weapons.clone(), character);

    ui.separator();
    ui.spacing();
    ui.separator();
    ui.spacing();

    let inv_cursor = ui.cursor_pos().clone();

    ui.child_window("##inv_block")
        .size([290.0, 400.0])
        .border(false)
        .build(|| {
            render_inventory(ui, character.ammo.clone(), character.apparel.clone(), character.consumables.clone(), character.robot_modules.clone(), character.gear.clone(), character.junk.clone(), character.misc.clone(), character);
    });

    let t_padding = 16.0_f32;
    let t_block_w = w - t_padding * 2.0 - 300.0;
    let t_gap = 8.0_f32;
    let t_col_w = (t_block_w - t_gap) / 2.0;
    let t_collapse_h = 44.0_f32;
    let t_expanded_h = 160.0_f32;

    let trait_h = if state.traits_expanded { t_expanded_h } else { t_collapse_h };

    ui.set_cursor_pos([inv_cursor[0] + t_padding + 300.0, inv_cursor[1]]);
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
                );
            }
    });

    for i in 0..(state.perks_expanded.len() + 1) / 2 {
        let perk1_h = if state.perks_expanded[i*2] {
            t_expanded_h
        } else { t_collapse_h };
        let perk2_h = if if state.perks_expanded.len()%2 == 0 { state.perks_expanded[i*2 + 1] } else { false } {
            t_expanded_h
        } else {
            t_collapse_h
        };
        let perk_h = perk1_h.max(perk2_h);

        match i {
            0 => {
                ui.set_cursor_pos([inv_cursor[0] + 300.0 + t_padding, inv_cursor[1] + trait_h + t_padding]);
                ui.child_window(format!("##p_block_{}", i))
                    .size([t_block_w, perk_h])
                    .border(false)
                    .build(|| {
                        let mut p1_desc: Vec<String> = vec![];
                        for j in 0..character.perks[i*2].ranks as usize {
                            p1_desc.push(format!("{}: {}", j+1, character.perks[i*2].desc[j]));
                        }
                        render_expandable_block(
                            ui,
                            &format!("##p{}_col", i*2),
                            t_col_w,
                            perk1_h,
                            &mut state.perks_expanded[i*2],
                            &character.perks[i*2].name,
                            Some(&p1_desc.join("\n")),
                        );
                        if character.perks.len()%2 == 0 {
                            let mut p2_desc: Vec<String> = vec![];
                            for j in 0..character.perks[i*2+1].ranks as usize {
                                p2_desc.push(format!("{}: {}", j+1, character.perks[i*2+1].desc[j]));
                            }
                            ui.same_line_with_spacing(0.0, t_gap);
                            render_expandable_block(
                                ui,
                                &format!("##p{}_col", i*2+1),
                                t_col_w,
                                perk2_h,
                                &mut state.perks_expanded[i*2+1],
                                &character.perks[i*2+1].name,
                                Some(&p2_desc.join("\n")),
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
                ui.set_cursor_pos([inv_cursor[0] + 300.0 + t_padding, curr_h]);
                ui.child_window(format!("##p_block_{}", i))
                    .size([t_block_w, perk_h])
                    .border(false)
                    .build(|| {
                        let mut p1_desc: Vec<String> = vec![];
                        for j in 0..character.perks[i*2].ranks as usize {
                            p1_desc.push(format!("{}: {}", j+1, character.perks[i*2].desc[j]));
                        }
                        render_expandable_block(
                            ui,
                            &format!("##p{}_col", i*2),
                            t_col_w,
                            perk1_h,
                            &mut state.perks_expanded[i*2],
                            &character.perks[i*2].name,
                            Some(&p1_desc.join("\n")),
                        );
                        if character.perks.len() >=  i*3 {
                            let mut p2_desc: Vec<String> = vec![];
                            for j in 0..character.perks[i*2+1].ranks as usize {
                                p2_desc.push(format!("{}: {}", j+1, character.perks[i*2+1].desc[j]));
                            }
                            ui.same_line_with_spacing(0.0, t_gap);
                            render_expandable_block(
                                ui,
                                &format!("##p{}_col", i*2+1),
                                t_col_w,
                                perk2_h,
                                &mut state.perks_expanded[i*2+1],
                                &character.perks[i*2+1].name,
                                Some(&p2_desc.join("\n")),
                            );
                        }
                });
            }
        }
    }

    h
}
