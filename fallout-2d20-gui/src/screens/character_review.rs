use imgui::Ui;
use sdl2::video::Window;
use std::cmp::Ordering;
use crate::{AppScreen, character::{AmmoInv, Apparel, ApparelType, BodyLocation, Character, DamageType, Skill}, db::Db, log_on_change, screens::{background_select::{BackgroundState, EquipmentState},
    skill_assignment::SKILLS,
    special_assignment::SPECIAL_LABELS,
    stat_calculation::{BaseDR, get_melee_str}}, theme::render_window
};

//for this i think we want to build the state to be something we can apply directly to the character struct upon acceptance; applying to the character directly here would likely lead to weird issues with clearing stuff when changing backgrounds/origins
pub struct ReviewState {
    pub loaded: bool,
}
impl ReviewState {
    pub fn new() -> Self {
        Self {
            loaded: false,
        }
    }
}

pub fn equip_apparel(
    character: &mut Character,
    equipment: &mut EquipmentState,
    background: &mut BackgroundState,
) {
    if background.selected_index.is_none() || background.apparel_selections.is_empty() { return }

    let apparel = equipment.apparel.clone();
    let mut _armor: Vec<(usize,&Apparel)> = vec![];
    let mut outfit_dr = BaseDR {
        ph_dr: 0,
        en_dr: 0,
        rd_dr: 0,
    };
    let mut outfit_pos = usize::MAX;
    let mut clothing_dr = BaseDR {
        ph_dr: 0,
        en_dr: 0,
        rd_dr: 0,
    };
    let mut clothing_pos = usize::MAX;
    let headgear: Vec<(usize,&Apparel)> = apparel.iter().enumerate().filter(|(_,a)| a.apparel_type == ApparelType::Headgear).collect();
    let mut armored_limbs: Vec<BodyLocation> = vec![];

    if character.is_robot() {
        _armor = apparel.iter().enumerate().filter(|(_,a)| a.apparel_type == ApparelType::RobotArmor).collect();
        let _modules = equipment.robot_modules.clone();

        if !headgear.is_empty() {
            let (_, hat) = headgear[0];
            character.robot_hat = Some(hat.clone());
        }
        //handle robot modules at some point
    } else {
        let outfits: Vec<(usize,&Apparel)> = apparel.iter().enumerate().filter(|(_,a)| a.apparel_type == ApparelType::Outfit).collect();
        let clothing: Vec<(usize,&Apparel)> = apparel.iter().enumerate().filter(|(_,a)| a.apparel_type == ApparelType::Clothing).collect();
        _armor = apparel.iter().enumerate().filter(|(_,a)| a.apparel_type == ApparelType::Armor).collect();
        (outfit_dr, outfit_pos) = match outfits.len() {
            0 => (outfit_dr, outfit_pos),
            1 => (
                BaseDR {
                    ph_dr: outfits[0].1.ph_dr,
                    en_dr: outfits[0].1.en_dr,
                    rd_dr: outfits[0].1.rd_dr
                }, outfits[0].0),
            _ => {
                let best = outfits
                    .iter()
                    .max_by(|a, b| {
                        match a.1.ph_dr.cmp(&b.1.ph_dr) {
                            Ordering::Equal => a.1.en_dr.cmp(&b.1.en_dr),
                            other => other,
                        }
                    });
                let (pos, best_dr) = (best.unwrap().0.clone(), BaseDR {
                    ph_dr: best.unwrap().1.ph_dr,
                    en_dr: best.unwrap().1.en_dr,
                    rd_dr: best.unwrap().1.rd_dr,
                });
                (best_dr, pos)
            },
        };
        (clothing_dr, clothing_pos) = match clothing.len() {
            0 => (clothing_dr, clothing_pos),
            1 => (
                BaseDR {
                    ph_dr: clothing[0].1.ph_dr,
                    en_dr: clothing[0].1.en_dr,
                    rd_dr: clothing[0].1.rd_dr
                }, clothing[0].0),
            _ => {
                let best = clothing
                    .iter()
                    .max_by(|a, b| {
                        match a.1.ph_dr.cmp(&b.1.ph_dr) {
                            Ordering::Equal => a.1.en_dr.cmp(&b.1.en_dr),
                            other => other,
                        }
                    });
                let (pos, best_dr) = (best.unwrap().0.clone(), BaseDR {
                    ph_dr: best.unwrap().1.ph_dr,
                    en_dr: best.unwrap().1.en_dr,
                    rd_dr: best.unwrap().1.rd_dr,
                });
                (best_dr, pos)
            },
        };
    }
    for item in _armor.clone() {
        let covers = item.1.covers.clone();
        for loc in covers {
            if !armored_limbs.contains(&loc) { armored_limbs.push(loc) }
        }
    }
    let mut top_each: Vec<(usize, &Apparel)> = vec![];
    for (i, loc) in armored_limbs.iter().enumerate() {
        let mut loc_armor: Vec<(usize, &Apparel)> = vec![];
        for item in _armor.clone() {
            if item.1.covers.contains(&loc) { loc_armor.push(item)}
        }
        for item in loc_armor {
            if top_each.len() <= i {
                top_each.push(item.clone());
            } else if top_each[i].1.ph_dr < item.1.ph_dr {
                top_each[i] = item.clone();
            } else if top_each[i].1.ph_dr == item.1.ph_dr && top_each[i].1.en_dr < item.1.en_dr {
                top_each[i] = item.clone();
            } else if top_each[i].1.ph_dr == item.1.ph_dr && top_each[i].1.en_dr == item.1.en_dr && top_each[i].1.rd_dr == item.1.rd_dr {
                top_each[i] = item.clone();
            }
        }
    }
    let mut outfit = false;
    if outfit_pos != usize::MAX {
        outfit = true;
        for loc in apparel[outfit_pos].covers.clone() {
            let limb_pos = armored_limbs.iter().position(|l| *l == loc);
            //can't guarantee there's a limb covered by armor here, handle the unwrap correctly
            if limb_pos.is_none() { break; }
            if outfit_dr.ph_dr < top_each[limb_pos.unwrap()].1.ph_dr + clothing_dr.ph_dr {
                outfit = false;
                break;
            } else if outfit_dr.ph_dr == top_each[limb_pos.unwrap()].1.ph_dr + clothing_dr.ph_dr && outfit_dr.en_dr < top_each[limb_pos.unwrap()].1.en_dr + clothing_dr.en_dr {
                outfit = false;
                break;
            } else if outfit_dr.ph_dr == top_each[limb_pos.unwrap()].1.ph_dr + clothing_dr.ph_dr && outfit_dr.en_dr == top_each[limb_pos.unwrap()].1.en_dr + clothing_dr.en_dr && outfit_dr.rd_dr < top_each[limb_pos.unwrap()].1.rd_dr + clothing_dr.rd_dr {
                outfit = false;
                break;
            }
        }
    }
    if outfit {
        equipment.apparel[outfit_pos].equipped = true;
        for loc in equipment.apparel[outfit_pos].covers.clone() {
            match loc {
                BodyLocation::Head => {
                    character.limb_dr.head.equipped = vec![equipment.apparel[outfit_pos].clone()];
                },
                BodyLocation::ArmLeft => {
                    character.limb_dr.arm_left.equipped = vec![equipment.apparel[outfit_pos].clone()];
                },
                BodyLocation::ArmRight => {
                    character.limb_dr.arm_right.equipped = vec![equipment.apparel[outfit_pos].clone()];
                },
                BodyLocation::Torso => {
                    character.limb_dr.torso.equipped = vec![equipment.apparel[outfit_pos].clone()];
                },
                BodyLocation::LegLeft => {
                    character.limb_dr.leg_left.equipped = vec![equipment.apparel[outfit_pos].clone()];
                },
                BodyLocation::LegRight => {
                    character.limb_dr.leg_right.equipped = vec![equipment.apparel[outfit_pos].clone()];
                },
                _ => {},
            }
        }
    } else {
        if clothing_pos != usize::MAX {
            equipment.apparel[clothing_pos].equipped = true;
            for loc in equipment.apparel[clothing_pos].covers.clone() {
                match loc {
                    BodyLocation::Head => {
                        character.limb_dr.head.equipped = vec![equipment.apparel[clothing_pos].clone()];
                        },
                    BodyLocation::ArmLeft => {
                        character.limb_dr.arm_left.equipped = vec![equipment.apparel[clothing_pos].clone()];
                    },
                    BodyLocation::ArmRight => {
                        character.limb_dr.arm_right.equipped = vec![equipment.apparel[clothing_pos].clone()];
                    },
                    BodyLocation::Torso => {
                        character.limb_dr.torso.equipped = vec![equipment.apparel[clothing_pos].clone()];
                    },
                    BodyLocation::LegLeft => {
                        character.limb_dr.leg_left.equipped = vec![equipment.apparel[clothing_pos].clone()];
                    },
                    BodyLocation::LegRight => {
                        character.limb_dr.leg_right.equipped = vec![equipment.apparel[clothing_pos].clone()];
                    },
                    _ => {},
                }
            }
        }
        if !character.is_robot() && !headgear.is_empty() {
            //just put on the first hat idgaf rn
            character.limb_dr.head.equipped.push(equipment.apparel[headgear[0].0].clone());
        }
        for (i,loc) in armored_limbs.iter().enumerate() {
            let item = equipment.apparel[top_each[i].0].clone();
            match loc {
                BodyLocation::None => {},
                BodyLocation::Head => character.limb_dr.head.equipped.push(item),
                BodyLocation::ArmLeft => character.limb_dr.arm_left.equipped.push(item),
                BodyLocation::ArmRight => character.limb_dr.arm_right.equipped.push(item),
                BodyLocation::Torso => character.limb_dr.torso.equipped.push(item),
                BodyLocation::LegLeft => character.limb_dr.leg_left.equipped.push(item),
                BodyLocation::LegRight => character.limb_dr.leg_right.equipped.push(item),
                BodyLocation::Optics => character.limb_dr.optics.equipped.push(item),
                BodyLocation::Arm1 => character.limb_dr.arm_1.equipped.push(item),
                BodyLocation::Arm2 => character.limb_dr.arm_2.equipped.push(item),
                BodyLocation::Arm3 => character.limb_dr.arm_3.equipped.push(item),
                BodyLocation::Body => character.limb_dr.body.equipped.push(item),
                BodyLocation::Thruster => character.limb_dr.thruster.equipped.push(item),
                BodyLocation::Wheel => character.limb_dr.wheel.equipped.push(item),
            };
        }
    }
    //background.equipment_changed = false;
}

pub fn render_character_review(
    ui: &Ui,
    window: &Window,
    state: &mut ReviewState,
    background: &mut BackgroundState,
    equipment: &mut EquipmentState,
    _db: &Db,
    character: &mut Character,
    screen: &mut AppScreen,
) -> f32 {
    let Some((w, h, _token)) = render_window(ui, window, "##character_review", "Character Review", screen)
        else { return 0.0 };

    ui.text("REVIEW");
    ui.separator();
    ui.spacing();

    if !state.loaded {
        equip_apparel(character, equipment, background);
        state.loaded = true;
    }
    let Some(_scroll) = ui.child_window("##review_scroll")
        .size([w - 16.0, h - 32.0 - 48.0])
        .begin()
    else { return h };

    let col_w = (w - 48.0) / 2.0;

    ui.text_disabled("IDENTITY");
    ui.separator();
    ui.spacing();

    ui.columns(2, "##id_cols", false);
    ui.set_column_width(0, col_w);
    ui.set_column_width(1, col_w);

    let origin_name = character.origin.clone().unwrap().name;
    
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
    ui.text(format!("{}",melee_str));

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
    ui.text_disabled(format!("{}{}{}{}{}   Luck Points:",spacer,spacer,spacer,spacer,spacer,));
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
        ui.text(format!("  {:20} {} {}  ", active_skills[i].0, active_skills[i].1, active_skills[i].2));
        ui.same_line();
        ui.text(format!("  {:20} {} {}  ", active_skills[i+rows].0, active_skills[i+rows].1, active_skills[i+rows].2));
        if active_skills.len() > i + rows * 2 {
            ui.same_line();
            ui.text(format!("  {:20} {} {}", active_skills[i+rows*2].0, active_skills[i+rows*2].1, active_skills[i+rows*2].2));
        }
    }

    ui.text_disabled("DR");
    ui.separator();
    ui.spacing();
    ui.text_disabled("temporary display:");

    let rad_dr = if character.is_mutant() || character.is_robot() {
        99
    } else {
        let atom = if character.origin.clone().unwrap().id == 13 { 1 } else { 0 };
        let rad_res = character.perk_ranks(73);
        atom + rad_res
    };

    if character.limb_dr.head.active {
        let mut ph: i32 = character.limb_dr.head.equipped.iter().map(|a| a.ph_dr).sum();
        ph += character.limb_dr.head.ph_dr;
        let mut en: i32 = character.limb_dr.head.equipped.iter().map(|a| a.en_dr).sum();
        en += character.limb_dr.head.en_dr;
        let mut rd: i32 = character.limb_dr.head.equipped.iter().map(|a| a.rd_dr).sum();
        rd += character.limb_dr.head.rd_dr + rad_dr;
        let worn: Vec<String> = character.limb_dr.head.equipped.iter().map(|a| a.name.clone()).collect();
        ui.text(format!("{:10} - P:{} E:{} R:{} - {}", "Head", ph, en, rd, worn.join(", ")));
    };
    if character.limb_dr.arm_left.active {
        let mut ph: i32 = character.limb_dr.arm_left.equipped.iter().map(|a| a.ph_dr).sum();
        ph += character.limb_dr.arm_left.ph_dr;
        let mut en: i32 = character.limb_dr.arm_left.equipped.iter().map(|a| a.en_dr).sum();
        en += character.limb_dr.arm_left.en_dr;
        let mut rd: i32 = character.limb_dr.arm_left.equipped.iter().map(|a| a.rd_dr).sum();
        rd += character.limb_dr.arm_left.rd_dr + rad_dr;
        let worn: Vec<String> = character.limb_dr.arm_left.equipped.iter().map(|a| a.name.clone()).collect();
        ui.text(format!("{:10} - P:{} E:{} R:{} - {}", "arm_left", ph, en, rd, worn.join(", ")))
    };
    if character.limb_dr.arm_right.active {
        let mut ph: i32 = character.limb_dr.arm_right.equipped.iter().map(|a| a.ph_dr).sum();
        ph += character.limb_dr.arm_right.ph_dr;
        let mut en: i32 = character.limb_dr.arm_right.equipped.iter().map(|a| a.en_dr).sum();
        en += character.limb_dr.arm_right.en_dr;
        let mut rd: i32 = character.limb_dr.arm_right.equipped.iter().map(|a| a.rd_dr).sum();
        rd += character.limb_dr.arm_right.rd_dr + rad_dr;
        let worn: Vec<String> = character.limb_dr.arm_right.equipped.iter().map(|a| a.name.clone()).collect();
        ui.text(format!("{:10} - P:{} E:{} R:{} - {}", "arm_right", ph, en, rd, worn.join(", ")))
    };
    if character.limb_dr.torso.active {
        let mut ph: i32 = character.limb_dr.torso.equipped.iter().map(|a| a.ph_dr).sum();
        ph += character.limb_dr.torso.ph_dr;
        let mut en: i32 = character.limb_dr.torso.equipped.iter().map(|a| a.en_dr).sum();
        en += character.limb_dr.torso.en_dr;
        let mut rd: i32 = character.limb_dr.torso.equipped.iter().map(|a| a.rd_dr).sum();
        rd += character.limb_dr.torso.rd_dr + rad_dr;
        let worn: Vec<String> = character.limb_dr.torso.equipped.iter().map(|a| a.name.clone()).collect();
        ui.text(format!("{:10} - P:{} E:{} R:{} - {}", "torso", ph, en, rd, worn.join(", ")))
    };
    if character.limb_dr.leg_left.active {
        let mut ph: i32 = character.limb_dr.leg_left.equipped.iter().map(|a| a.ph_dr).sum();
        ph += character.limb_dr.leg_left.ph_dr;
        let mut en: i32 = character.limb_dr.leg_left.equipped.iter().map(|a| a.en_dr).sum();
        en += character.limb_dr.leg_left.en_dr;
        let mut rd: i32 = character.limb_dr.leg_left.equipped.iter().map(|a| a.rd_dr).sum();
        rd += character.limb_dr.leg_left.rd_dr + rad_dr;
        let worn: Vec<String> = character.limb_dr.leg_left.equipped.iter().map(|a| a.name.clone()).collect();
        ui.text(format!("{:10} - P:{} E:{} R:{} - {}", "leg_left", ph, en, rd, worn.join(", ")))
    };
    if character.limb_dr.leg_right.active {
        let mut ph: i32 = character.limb_dr.leg_right.equipped.iter().map(|a| a.ph_dr).sum();
        ph += character.limb_dr.leg_right.ph_dr;
        let mut en: i32 = character.limb_dr.leg_right.equipped.iter().map(|a| a.en_dr).sum();
        en += character.limb_dr.leg_right.en_dr;
        let mut rd: i32 = character.limb_dr.leg_right.equipped.iter().map(|a| a.rd_dr).sum();
        rd += character.limb_dr.leg_right.rd_dr + rad_dr;
        let worn: Vec<String> = character.limb_dr.leg_right.equipped.iter().map(|a| a.name.clone()).collect();
        ui.text(format!("{:10} - P:{} E:{} R:{} - {}", "leg_right", ph, en, rd, worn.join(", ")))
    };
    if character.limb_dr.optics.active {
        let mut ph: i32 = character.limb_dr.optics.equipped.iter().map(|a| a.ph_dr).sum();
        ph += character.limb_dr.optics.ph_dr;
        let mut en: i32 = character.limb_dr.optics.equipped.iter().map(|a| a.en_dr).sum();
        en += character.limb_dr.optics.en_dr;
        let mut rd: i32 = character.limb_dr.optics.equipped.iter().map(|a| a.rd_dr).sum();
        rd += character.limb_dr.optics.rd_dr + rad_dr;
        let worn: Vec<String> = character.limb_dr.optics.equipped.iter().map(|a| a.name.clone()).collect();
        ui.text(format!("{:10} - P:{} E:{} R:{} - {}", "optics", ph, en, rd, worn.join(", ")))
    };
    if character.limb_dr.arm_1.active {
        let mut ph: i32 = character.limb_dr.arm_1.equipped.iter().map(|a| a.ph_dr).sum();
        ph += character.limb_dr.arm_1.ph_dr;
        let mut en: i32 = character.limb_dr.arm_1.equipped.iter().map(|a| a.en_dr).sum();
        en += character.limb_dr.arm_1.en_dr;
        let mut rd: i32 = character.limb_dr.arm_1.equipped.iter().map(|a| a.rd_dr).sum();
        rd += character.limb_dr.arm_1.rd_dr + rad_dr;
        let worn: Vec<String> = character.limb_dr.arm_1.equipped.iter().map(|a| a.name.clone()).collect();
        ui.text(format!("{:10} - P:{} E:{} R:{} - {}", "arm_1", ph, en, rd, worn.join(", ")))
    };
    if character.limb_dr.arm_2.active {
        let mut ph: i32 = character.limb_dr.arm_2.equipped.iter().map(|a| a.ph_dr).sum();
        ph += character.limb_dr.arm_2.ph_dr;
        let mut en: i32 = character.limb_dr.arm_2.equipped.iter().map(|a| a.en_dr).sum();
        en += character.limb_dr.arm_2.en_dr;
        let mut rd: i32 = character.limb_dr.arm_2.equipped.iter().map(|a| a.rd_dr).sum();
        rd += character.limb_dr.arm_2.rd_dr + rad_dr;
        let worn: Vec<String> = character.limb_dr.arm_2.equipped.iter().map(|a| a.name.clone()).collect();
        ui.text(format!("{:10} - P:{} E:{} R:{} - {}", "arm_2", ph, en, rd, worn.join(", ")))
    };
    if character.limb_dr.arm_3.active {
        let mut ph: i32 = character.limb_dr.arm_3.equipped.iter().map(|a| a.ph_dr).sum();
        ph += character.limb_dr.arm_3.ph_dr;
        let mut en: i32 = character.limb_dr.arm_3.equipped.iter().map(|a| a.en_dr).sum();
        en += character.limb_dr.arm_3.en_dr;
        let mut rd: i32 = character.limb_dr.arm_3.equipped.iter().map(|a| a.rd_dr).sum();
        rd += character.limb_dr.arm_3.rd_dr + rad_dr;
        let worn: Vec<String> = character.limb_dr.arm_3.equipped.iter().map(|a| a.name.clone()).collect();
        ui.text(format!("{:10} - P:{} E:{} R:{} - {}", "arm_3", ph, en, rd, worn.join(", ")))
    };
    if character.limb_dr.body.active {
        let mut ph: i32 = character.limb_dr.body.equipped.iter().map(|a| a.ph_dr).sum();
        ph += character.limb_dr.body.ph_dr;
        let mut en: i32 = character.limb_dr.body.equipped.iter().map(|a| a.en_dr).sum();
        en += character.limb_dr.body.en_dr;
        let mut rd: i32 = character.limb_dr.body.equipped.iter().map(|a| a.rd_dr).sum();
        rd += character.limb_dr.body.rd_dr + rad_dr;
        let worn: Vec<String> = character.limb_dr.body.equipped.iter().map(|a| a.name.clone()).collect();
        ui.text(format!("{:10} - P:{} E:{} R:{} - {}", "body", ph, en, rd, worn.join(", ")))
    };
    if character.limb_dr.thruster.active {
        let mut ph: i32 = character.limb_dr.thruster.equipped.iter().map(|a| a.ph_dr).sum();
        ph += character.limb_dr.thruster.ph_dr;
        let mut en: i32 = character.limb_dr.thruster.equipped.iter().map(|a| a.en_dr).sum();
        en += character.limb_dr.thruster.en_dr;
        let mut rd: i32 = character.limb_dr.thruster.equipped.iter().map(|a| a.rd_dr).sum();
        rd += character.limb_dr.thruster.rd_dr + rad_dr;
        let worn: Vec<String> = character.limb_dr.thruster.equipped.iter().map(|a| a.name.clone()).collect();
        ui.text(format!("{:10} - P:{} E:{} R:{} - {}", "thruster", ph, en, rd, worn.join(", ")))
    };
    if character.limb_dr.wheel.active {
        let mut ph: i32 = character.limb_dr.wheel.equipped.iter().map(|a| a.ph_dr).sum();
        ph += character.limb_dr.wheel.ph_dr;
        let mut en: i32 = character.limb_dr.wheel.equipped.iter().map(|a| a.en_dr).sum();
        en += character.limb_dr.wheel.en_dr;
        let mut rd: i32 = character.limb_dr.wheel.equipped.iter().map(|a| a.rd_dr).sum();
        rd += character.limb_dr.wheel.rd_dr + rad_dr;
        let worn: Vec<String> = character.limb_dr.wheel.equipped.iter().map(|a| a.name.clone()).collect();
        ui.text(format!("{:10} - P:{} E:{} R:{} - {}", "wheel", ph, en, rd, worn.join(", ")))
    };

    /*
    DR BLOCK
    */

    ui.text_disabled("WEAPONS");
    ui.separator();
    ui.spacing();

    if equipment.weapons.is_empty() {
        ui.text_disabled("  No weapons.");
    } else {
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
            ui.text_disabled( hdr);
            ui.next_column();
        }
        ui.separator();

        for weapon in &equipment.weapons {
            let weapon_name = format!("{} {}",weapon.prefix, weapon.name);
            let eff_str = weapon.effects.join(", ");
            let qual_str = weapon.qualities.join(",");
            let tag_str = if weapon.tag { "*" } else { "" };
            let skill_str = match weapon.skill {
                Skill::Athletics => "At",
                Skill::BigGuns => "BG",
                Skill::EnergyWeapons => "EW",
                Skill::Explosives => "Ex",
                Skill::MeleeWeapons => "MW",
                Skill::SmallGuns => "SG",
                Skill::Throwing => "Th",
                Skill::Unarmed => "Un",
                _ => "",
            };
            let dam_type = match weapon.dam_type {
                DamageType::All => "All",
                DamageType::En => "En",
                DamageType::EnRad => "En/Rd",
                DamageType::Ph => "Ph",
                DamageType::PhEn => "Ph/En",
                DamageType::Poi => "Poi",
                DamageType::Rad => "Rd",
                DamageType::None => "N/A",
            };
            let mut damage = weapon.damage;
            if skill_str == "MW" {
                damage += character.melee_mod.melee;
            } else if skill_str == "Un" {
                damage += character.melee_mod.melee + character.melee_mod.unarmed;
            }

            let cells: &[&str] = &[
                &weapon_name,
                skill_str,
                &weapon.target.to_string(),
                tag_str,
                &damage.to_string(),
                &eff_str,
                dam_type,
                &weapon.rate.to_string(),
                &weapon.range,
                &qual_str,
                &weapon.ammo,
                &weapon.wgt.to_string(),
            ];
            for cell in cells {
                ui.text_wrapped(cell);
                ui.next_column();
            }
        }
        ui.columns(1, "##weapon_end", false);
    }
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

    let name_w = 150.0_f32;
    let wgt_w = 55.0_f32;
    let quan_w = 75.0_f32;
    let eq_w = 75.0_f32;

    let mut eq_wgt = 0;

    let ammo_actual: Vec<&AmmoInv> = equipment.ammo.iter().filter(|a| a.quantity > 0).collect();

    if !ammo_actual.is_empty() {
        ui.columns(3,"##eq_ammo",false);
        ui.set_column_width(0, name_w);
        ui.text_disabled("Ammo");
        ui.next_column();
        ui.set_column_width(1, wgt_w);
        ui.text_disabled("Weight");
        ui.next_column();
        ui.set_column_width(2, quan_w);
        ui.text_disabled("Quantity");
        ui.next_column();
        ui.separator();
        for item in ammo_actual.clone() {
            ui.text(format!("{}", item.ammo.name));
            ui.next_column();
            ui.text(format!("{}", item.ammo.wgt));
            ui.next_column();
            ui.text(format!("{}", item.quantity));
            ui.next_column();
            eq_wgt += item.ammo.wgt * item.quantity;
        }
        ui.spacing();
    }
    if !equipment.apparel.is_empty() {
        ui.columns(3,"##eq_apparel",false);
        ui.set_column_width(0, name_w);
        ui.text_disabled("Apparel");
        ui.next_column();
        ui.set_column_width(1, wgt_w);
        ui.text_disabled("Weight");
        ui.next_column();
        ui.set_column_width(2, eq_w);
        ui.text_disabled("Equipped");
        ui.next_column();
        ui.separator();
        for item in equipment.apparel.clone() {
            ui.text(format!("{}", item.name));
            ui.next_column();
            ui.text(format!("{}", item.wgt));
            ui.next_column();
            ui.text(format!("{}", if item.equipped {"*"} else {""}));
            ui.next_column();
            eq_wgt += item.wgt;
        }
        ui.spacing();
    }
    if !equipment.consumables.is_empty() {
        ui.columns(3,"##eq_consumable",false);
        ui.set_column_width(0, name_w);
        ui.text_disabled("Consumable");
        ui.next_column();
        ui.set_column_width(1, wgt_w);
        ui.text_disabled("Weight");
        ui.next_column();
        ui.set_column_width(2, quan_w);
        ui.text_disabled("Quantity");
        ui.next_column();
        ui.separator();
        for item in equipment.consumables.clone() {
            ui.text(format!("{}", item.name));
            ui.next_column();
            ui.text(format!("{}", item.wgt));
            ui.next_column();
            ui.text(format!("{}", item.quantity));
            ui.next_column();
            eq_wgt += item.wgt * item.quantity;
        }
        ui.spacing();
    }
    if !equipment.robot_modules.is_empty() {
        ui.columns(3,"##eq_robomods",false);
        ui.set_column_width(0, name_w);
        ui.text_disabled("Module");
        ui.next_column();
        ui.set_column_width(1, wgt_w);
        ui.text_disabled("Weight");
        ui.next_column();
        ui.set_column_width(2, eq_w);
        ui.text_disabled("Equipped");
        ui.next_column();
        ui.separator();
        for item in equipment.apparel.clone() {
            ui.text(format!("{}", item.name));
            ui.next_column();
            ui.text(format!("{}", item.wgt));
            ui.next_column();
            ui.text(format!("{}", if item.equipped {"*"} else {""}));
            ui.next_column();
            eq_wgt += item.wgt;
        }
        ui.spacing();
    }
    if !equipment.gear.is_empty() {
        ui.columns(3,"##eq_gear",false);
        ui.set_column_width(0, name_w);
        ui.text_disabled("Gear");
        ui.next_column();
        ui.set_column_width(1, wgt_w);
        ui.text_disabled("Weight");
        ui.next_column();
        ui.set_column_width(2, quan_w);
        ui.text_disabled("Quantity");
        ui.next_column();
        ui.separator();
        for item in equipment.consumables.clone() {
            ui.text(format!("{}", item.name));
            ui.next_column();
            ui.text(format!("{}", item.wgt));
            ui.next_column();
            ui.text(format!("{}", item.quantity));
            ui.next_column();
            eq_wgt += item.wgt * item.quantity;
        }
        ui.spacing();
    }
    if equipment.junk.common > 0 {
        ui.columns(1,"#eq_junk", false);
        ui.text_disabled(format!("{:14}", "Junk (Common)"));
        ui.same_line();
        ui.text(format!("{}",equipment.junk.common));
        ui.spacing();
            eq_wgt += equipment.junk.common * 2;
    }
    let misc_actual: Vec<&String> = equipment.misc.iter().filter(|s| *s != "").collect();
    if !misc_actual.is_empty() {
        ui.columns(1,"#eq_misc", false);
        ui.text_disabled("Misc");
        for item in misc_actual.clone() {
            ui.text(format!("  {}", item));
        }
        ui.spacing();
    }
    ui.separator();

    ui.columns(2, "##eq_weight", false);
    ui.set_column_width(0,name_w);
    ui.text_disabled("Current Weight");
    ui.next_column();
    ui.set_column_width(1, wgt_w);
    ui.text(format!("{}", eq_wgt));
    character.carry_wgt = eq_wgt;
    ui.next_column();
    ui.text_disabled("Max Weight");
    ui.next_column();
    ui.text(format!("{}", character.carry_wgt_max));
    ui.next_column();

    ui.columns(1,"##end_eq", false);

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

    log_on_change!(format!("char: {:?}", character));

    drop(_scroll);
    h
}