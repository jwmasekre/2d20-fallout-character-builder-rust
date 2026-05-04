use imgui::Ui;
use sdl2::video::Window;
use crate::{character::{Character, Skill, DamageType},
    db::Db,
    screens::{background_select::{BackgroundState, EquipmentState},
    skill_assignment::SKILLS,
    special_assignment::SPECIAL_LABELS,
    stat_calculation::get_melee_str},
    theme::render_window
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

pub fn render_character_review(
    ui: &Ui,
    window: &Window,
    state: &mut ReviewState,
    background: &BackgroundState,
    equipment: &EquipmentState,
    _db: &Db,
    character: &Character,
) -> f32 {
    let Some((w, h, _token)) = render_window(ui, window, "##character_review", "Character Review")
        else { return 0.0 };

    ui.text("REVIEW");
    ui.separator();
    ui.spacing();

    if !state.loaded {
        //trigger all the clothing 
    }
    let Some(_scroll) = ui.child_window("##review_scroll")
        .size([w - 16.0, h - 32.0 - 44.0])
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
    ui.text_disabled(format!("{}{}{}{}{}Luck Points:",spacer,spacer,spacer,spacer,spacer,));
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
    let quan_w = 70.0_f32;
    let eq_w = 70.0_f32;

    let mut eq_wgt = 0;

    if !equipment.ammo.is_empty() {
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
        for item in equipment.ammo.clone() {
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
    if !equipment.misc.is_empty() {
        ui.columns(1,"#eq_misc", false);
        ui.text_disabled("Misc");
        for item in equipment.misc.clone() {
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
    ui.next_column();
    ui.text_disabled("Max Weight");
    ui.next_column();
    ui.text(format!("{}", character.carry_wgt));
    ui.next_column();

    ui.columns(1,"##end_eq", false);

    drop(_scroll);
    h
}