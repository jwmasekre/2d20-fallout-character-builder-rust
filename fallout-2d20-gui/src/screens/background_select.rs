use crate::{AppScreen, theme::render_window};
use fallout_2d20_core::{
    background_slots::{
        render_apparel_slot,
        render_consumable_slot,
        render_robot_module_slot,
        render_weapon_slot
    }, character::{
        Background,
        Character
    }, db::Db, roll_d20, roll_trinket, states::{
        BackgroundState,
        EquipmentState,
        OriginState,
        PerkState,
        ReviewState,
        SkillState,
        SpecialState
    }, structs::AppConfig
};
use imgui::Ui;
use sdl2::video::Window;
//use rand::rng;

pub fn render_background_select(
    ui: &Ui,
    window: &Window,
    state: &mut BackgroundState,
    equipment: &mut EquipmentState,
    db: &Db,
    character: &mut Character,
    _review: &mut ReviewState,
    screen: &mut AppScreen,
    origin: &mut OriginState,
    special: &mut SpecialState,
    skill: &mut SkillState,
    perk: &mut PerkState,
    cfg: &AppConfig,
) -> f32 {
    let Some((w, h, _token)) = render_window(ui, window, "##background_select", "Background Select", screen, origin, special, skill, perk, state, equipment, character, cfg)
        else { return 0.0 };

    ui.text("BACKGROUND");
    ui.separator();
    ui.spacing();

    ui.text("Background:");
    ui.same_line();
    ui.set_next_item_width(280.0 * cfg.ui_scale);
    //grab all the available backgrounds for the selected origin
    let bg_names: Vec<(usize, String)> = state.origin_backgrounds(character.clone())
        .into_iter()
        .map(|(i, bg)| (i, bg.name.clone()))
        .collect();
    let preview = state.selected_index
        .and_then(|i| state.all_backgrounds.get(i))
        .map(|bg| bg.name.as_str())
        .unwrap_or("Select background...");
    if let Some(_cb) = ui.begin_combo("##bg_select", preview) {
        for (i, name) in &bg_names {
            let sel = state.selected_index == Some(*i);
            if ui.selectable_config(name.as_str()).selected(sel).build() {
                if state.selected_index != Some(*i) {
                    state.load_background(db, *i);
                    character.background = Some(Background {
                        id: (*i + 1) as i32,
                        name: state.current_background.clone().unwrap().name,
                        desc: state.current_background.clone().unwrap().desc,
                    });
                }
            }
        }
    }
    ui.spacing();
    ui.separator();
    ui.spacing();
    //if there isn't a selected background, inform the player and stop rendering
    let Some(bg) = &state.current_background else {
        ui.text_disabled("Select a background to view starting equipment options...");
        return h;
    };
    //create a clone of the background we can reference
    //avoids borrowing issues
    let bg = bg.clone();
    //creates a scrolling child window for the selection (not sure if we ever need this much space but who knows)
    let list_h = h - 140.0 * cfg.ui_scale;
    let Some(_child) = ui.child_window("##equip_scroll")
        .size([w - 16.0 * cfg.ui_scale, list_h])
        .begin()
    else { return h };

    ui.separator();
    //weapons
    if !bg.weapon_slots.is_empty() {
        ui.text("WEAPONS");
        ui.separator();
        ui.spacing();
        for (i, slot) in bg.weapon_slots.iter().enumerate() {
            state.equipment_changed = render_weapon_slot(ui, i, slot, &mut state.weapon_selections[i], &bg.ammo, cfg);
            ui.spacing();
        }
        ui.spacing();
    }
    //apparel
    if !bg.apparel_slots.is_empty() {
        ui.text("APPAREL");
        ui.separator();
        ui.spacing();
        for (i, slot) in bg.apparel_slots.iter().enumerate() {
            state.equipment_changed = render_apparel_slot(ui, i, slot, &mut state.apparel_selections[i], cfg);
            ui.spacing();
        }
        ui.spacing();
    }
    //consumables
    if !bg.consumable_slots.is_empty() {
        ui.text("CONSUMABLES");
        ui.separator();
        ui.spacing();
        for (i, slot) in bg.consumable_slots.iter().enumerate() {
            state.equipment_changed = render_consumable_slot(ui, i, slot, &mut state.consumable_selections[i], cfg);
            ui.spacing();
        }
        ui.spacing();
    }
    //robot modules
    if !bg.robot_module_slots.is_empty() {
        ui.text("ROBOT MODULES");
        ui.separator();
        ui.spacing();
        for (i, slot) in bg.robot_module_slots.iter().enumerate() {
            state.equipment_changed = render_robot_module_slot(ui, i, slot, &mut state.robot_module_selections[i], cfg);
            ui.spacing();
        }
        ui.spacing();
    }
    //gear
    if !bg.gear.is_empty() {
        ui.text("GEAR");
        ui.separator();
        ui.spacing();
        for g in &bg.gear {
            ui.text(format!("  {}", g.gear_name));
        }
        ui.spacing();
    }
    //misc
    ui.text("MISC");
    ui.separator();
    ui.spacing();
    ui.text(format!("  Caps: {}", bg.caps));
    if !bg.misc.is_empty()   { ui.text(format!("  Misc: {}", bg.misc)); }
    if bg.trinket + bg.food + bg.forage + bg.bev + bg.chem + bg.ammo_count + bg.aid + bg.odd + bg.outcast + bg.junk > 0 {
        ui.text("Table Rolls:");
/*====*/if bg.trinket > 0 {
            if state.roll_trinket.len() < bg.trinket as usize {
                for _ in 0..bg.trinket {
                    state.roll_trinket.push(String::new());
                }
            }
            ui.text(format!("  Trinket x{}", bg.trinket));
            for i in 0..bg.trinket as usize {
                let label = if state.roll_trinket[i] == String::new() {
                    "Roll"
                } else {
                    "Reroll"
                };
                ui.same_line();
                if ui.button(format!("{}##trinket{}",label,i)) {
                    state.roll_trinket[i] = roll_trinket();
                }
            }
            let mut trinket_list: Vec<String> = vec![];
            for trinket in state.roll_trinket.clone() {
                if trinket != String::new() { trinket_list.push(trinket); }
            }
            ui.text_disabled(format!("    {}", trinket_list.join(", ")));
        }
/*====*/if bg.food > 0 {
            if state.roll_food.len() < bg.food as usize {
                for _ in 0..bg.food {
                    state.roll_food.push(None);
                }
            }
            ui.text(format!("  Food x{}", bg.food));
            for i in 0..bg.food as usize {
                let label = if state.roll_food[i].is_none() {
                    "Roll"
                } else {
                    "Reroll"
                };
                ui.same_line();
                if ui.button(format!("{}##food{}",label,i)) {
                    state.roll_food[i] = Some(db.roll_food_core()[0].clone());
                }
            }
            let mut food_list: Vec<String> = vec![];
            for food in state.roll_food.clone() {
                if food.is_some() { food_list.push(food.unwrap().name); }
            }
            ui.text_disabled(format!("    {}", food_list.join(", ")));
        }
/*====*/if bg.forage > 0 {
            if state.roll_forage.len() < bg.forage as usize {
                for _ in 0..bg.forage {
                    state.roll_forage.push(None);
                }
            }
            ui.text(format!("  Forage x{}", bg.forage));
            for i in 0..bg.forage as usize {
                let label = if state.roll_forage[i].is_none() {
                    "Roll"
                } else {
                    "Reroll"
                };
                ui.same_line();
                if ui.button(format!("{}##forage{}",label,i)) {
                    state.roll_forage[i] = Some(db.roll_forage_core()[0].clone());
                }
            }
            let mut forage_list: Vec<String> = vec![];
            for forage in state.roll_forage.clone() {
                if forage.is_some() { forage_list.push(forage.unwrap().name); }
            }
            ui.text_disabled(format!("    {}", forage_list.join(", ")));
        }
/*====*/if bg.bev > 0 {
            if state.roll_bev.len() < bg.bev as usize {
                for _ in 0..bg.bev {
                    state.roll_bev.push(None);
                }
            }
            ui.text(format!("  Bev x{}", bg.bev));
            for i in 0..bg.bev as usize {
                let label = if state.roll_bev[i].is_none() {
                    "Roll"
                } else {
                    "Reroll"
                };
                ui.same_line();
                if ui.button(format!("{}##bev{}",label,i)) {
                    state.roll_bev[i] = Some(db.roll_bevs_core()[0].clone());
                }
            }
            let mut bev_list: Vec<String> = vec![];
            for bev in state.roll_bev.clone() {
                if bev.is_some() { bev_list.push(bev.unwrap().name); }
            }
            ui.text_disabled(format!("    {}", bev_list.join(", ")));
        }
/*====*/if bg.chem > 0 {
            if state.roll_chem.len() < bg.chem as usize {
                for _ in 0..bg.chem {
                    state.roll_chem.push(None);
                }
            }
            ui.text(format!("  Chem x{}", bg.chem));
            for i in 0..bg.chem as usize {
                let label = if state.roll_chem[i].is_none() {
                    "Roll"
                } else {
                    "Reroll"
                };
                ui.same_line();
                if ui.button(format!("{}##chem{}",label,i)) {
                    state.roll_chem[i] = Some(db.roll_chem_core()[0].clone());
                }
            }
            let mut chem_list: Vec<String> = vec![];
            for chem in state.roll_chem.clone() {
                if chem.is_some() { chem_list.push(chem.unwrap().name); }
            }
            ui.text_disabled(format!("    {}", chem_list.join(", ")));
        }
/*====*/if bg.ammo_count > 0 {
            if state.roll_ammo_count.len() < bg.ammo_count as usize {
                for _ in 0..bg.ammo_count {
                    state.roll_ammo_count.push(None);
                }
            }
            ui.text(format!("  Ammo x{}", bg.ammo_count));
            for i in 0..bg.ammo_count as usize {
                let label = if state.roll_ammo_count[i].is_none() {
                    "Roll"
                } else {
                    "Reroll"
                };
                ui.same_line();
                if ui.button(format!("{}##ammo_count{}",label,i)) {
                    state.roll_ammo_count[i] = Some(db.roll_ammo_core()[0].clone());
                }
            }
            let mut ammo_count_list: Vec<String> = vec![];
            for ammo_count in state.roll_ammo_count.clone() {
                if ammo_count.is_some() { ammo_count_list.push(format!("{} x{}",ammo_count.clone().unwrap().ammo.name, ammo_count.clone().unwrap().quantity)); }
            }
            ui.text_disabled(format!("    {}", ammo_count_list.join(", ")));
        }
/*====*/if bg.aid > 0 {
            if state.roll_aid.len() < bg.aid as usize {
                for _ in 0..bg.aid {
                    state.roll_aid.push(None);
                }
            }
            ui.text(format!("  Aid x{}", bg.aid));
            for i in 0..bg.aid as usize {
                let label = if state.roll_aid[i].is_none() {
                    "Roll"
                } else {
                    "Reroll"
                };
                ui.same_line();
                if ui.button(format!("{}##aid{}",label,i)) {
                    state.roll_aid[i] = Some(db.roll_chem_core()[0].clone());
                }
            }
            let mut aid_list: Vec<String> = vec![];
            for aid in state.roll_aid.clone() {
                if aid.is_some() { aid_list.push(aid.unwrap().name); }
            }
            ui.text_disabled(format!("    {}", aid_list.join(", ")));
        }
/*====*/if bg.odd > 0 {
            if state.roll_odd.len() < bg.odd as usize {
                for _ in 0..bg.odd {
                    state.roll_odd.push(None);
                }
            }
            ui.text(format!("  odd x{}", bg.odd));
            for i in 0..bg.odd as usize {
                let label = if state.roll_odd[i].is_none() {
                    "Roll"
                } else {
                    "Reroll"
                };
                ui.same_line();
                if ui.button(format!("{}##odd{}",label,i)) {
                    state.roll_odd[i] = Some(db.roll_random_core());
                }
            }
            let mut odd_list: Vec<String> = vec![];
            for odd in state.roll_odd.clone() {
                if odd.is_some() { 
                    if !odd.clone().unwrap().0.is_empty() {
                        odd_list.push(odd.clone().unwrap().0[0].name.clone());
                    } else if !odd.clone().unwrap().1.is_empty() {
                        odd_list.push(odd.clone().unwrap().1[0].name.clone());
                    } else if !odd.clone().unwrap().2.is_empty() {
                        odd_list.push(odd.clone().unwrap().2[0].clone());
                    } else if !odd.clone().unwrap().3.is_empty() {
                        odd_list.push(format!("{} {}",odd.clone().unwrap().3[0].1, if odd.clone().unwrap().3[0].0 {"pre-war dollars"} else {"caps"}));
                    } else if !odd.clone().unwrap().4.is_empty() {
                        odd_list.push(odd.clone().unwrap().4[0].name.clone());
                    }
                 }
            }
            ui.text_disabled(format!("    {}", odd_list.join(", ")));
        }
/*====*/if bg.outcast > 0 {
            if state.roll_outcast.len() < bg.outcast as usize {
                for _ in 0..bg.outcast {
                    state.roll_outcast.push(None);
                }
            }
            ui.text(format!("  outcast x{}", bg.outcast));
            for i in 0..bg.outcast as usize {
                let label = if state.roll_outcast[i].is_none() {
                    "Roll"
                } else {
                    "Reroll"
                };
                ui.same_line();
                if ui.button(format!("{}##outcast{}",label,i)) {
                    state.roll_outcast[i] = Some(db.roll_random_outcast(character));
                }
            }
            let mut outcast_list: Vec<String> = vec![];
            for outcast in state.roll_outcast.clone() {
                if outcast.is_some() { 
                    if !outcast.clone().unwrap().0.is_empty() {
                        outcast_list.push(outcast.unwrap().0[0].name.clone());
                    } else if !outcast.clone().unwrap().1.is_empty() {
                        outcast_list.push(outcast.unwrap().1[0].name.clone());
                    } else if !outcast.clone().unwrap().2.is_empty() {
                        outcast_list.push(outcast.unwrap().2[0].name.clone());
                    } else if !outcast.clone().unwrap().3.is_empty() {
                        outcast_list.push(outcast.unwrap().3[0].name.clone());
                    } else if !outcast.clone().unwrap().4.is_empty() {
                        outcast_list.push(outcast.unwrap().4);
                    } else if !outcast.clone().unwrap().5.is_empty() {
                        outcast_list.push(outcast.unwrap().5[0].name.clone());
                    }
                 }
            }
            ui.text_disabled(format!("    {}", outcast_list.join(", ")));
        }
/*====*/if bg.junk > 0 {
            if state.roll_junk.len() < bg.junk as usize {
                for _ in 0..bg.junk {
                    state.roll_junk.push(0);
                }
            }
            ui.text(format!("  Junk x{}", bg.junk));
            for i in 0..bg.junk as usize {
                let label = if state.roll_junk[i] <= 1 {
                    "Roll"
                } else {
                    "Reroll"
                };
                ui.same_line();
                if ui.button(format!("{}##junk{}",label,i)) {
                    state.roll_junk[i] = roll_d20(2);
                }
            }
            let mut junk_list: Vec<String> = vec![];
            for junk in state.roll_junk.clone() {
                if junk > 0 { junk_list.push(junk.to_string()); }
            }
            ui.text_disabled(format!("    {}", junk_list.join(", ")));
        }
    }
    ui.spacing();
    ui.separator();
    ui.spacing();
    ui.separator();
    ui.spacing();

    ui.text("debug output");
    ui.spacing();
    ui.separator();
    ui.spacing();

    /*
    ui.text_disabled("background:");
    ui.same_line();
    if state.current_background.is_none() {
        ui.text("none");
    } else {
        let bg = state.current_background.clone().unwrap();
        ui.text_wrapped(format!("  id: {}   name: {}   caps: {}   misc: {}", bg.id, bg.name, bg.caps, bg.misc));
        ui.text_wrapped(format!("              trinket: {}  food: {}  forage: {}  bev: {}  chem: {}  ammo: {}  aid: {}  odd: {}  outcast: {}  junk: {}", bg.trinket, bg.food, bg.forage, bg.bev, bg.chem, bg.ammo_count, bg.aid, bg.odd, bg.outcast, bg.junk));
        ui.text_disabled("  weapon_slots:      ");
        ui.same_line();
        ui.text_wrapped(format!("{:?}", bg.weapon_slots));
        ui.text_disabled("  ammo:              ");
        ui.same_line();
        ui.text_wrapped(format!("{:?}", bg.ammo));
        ui.text_disabled("  apparel_slots:     ");
        ui.same_line();
        ui.text_wrapped(format!("{:?}", bg.apparel_slots));
        ui.text_disabled("  consumable_slots:  ");
        ui.same_line();
        ui.text_wrapped(format!("{:?}", bg.consumable_slots));
        ui.text_disabled("  robotmod_slots:    ");
        ui.same_line();
        ui.text_wrapped(format!("{:?}", bg.robot_module_slots));
        ui.text_disabled("  gear_slots:        ");
        ui.same_line();
        ui.text_wrapped(format!("{:?}", bg.gear));
    }
    ui.text_disabled("weapons:");
    ui.same_line();
    ui.text_wrapped(format!("  {:?}", state.weapon_selections));
    ui.text_disabled("apparel:");
    ui.same_line();
    ui.text_wrapped(format!("  {:?}", state.apparel_selections));
    ui.text_disabled("consumables:");
    ui.same_line();
    ui.text_wrapped(format!("  {:?}", state.consumable_selections));
    ui.text_disabled("robot modules:");
    ui.same_line();
    ui.text_wrapped(format!("  {:?}", state.robot_module_selections));
    */

    ui.separator();
    ui.separator();
    ui.text_disabled("weapons   ");
    ui.same_line();
    ui.text_wrapped(format!("{:?}", equipment.weapons));
    ui.text_disabled("ammo   ");
    ui.same_line();
    ui.text_wrapped(format!("{:?}", equipment.ammo));
    ui.text_disabled("apparel   ");
    ui.same_line();
    ui.text_wrapped(format!("{:?}", equipment.apparel));
    ui.text_disabled("consumables   ");
    ui.same_line();
    ui.text_wrapped(format!("{:?}", equipment.consumables));
    ui.text_disabled("robot modules   ");
    ui.same_line();
    ui.text_wrapped(format!("{:?}", equipment.robot_modules));
    ui.text_disabled("gear   ");
    ui.same_line();
    ui.text_wrapped(format!("{:?}", equipment.gear));
    ui.text_disabled("junk   ");
    ui.same_line();
    ui.text_wrapped(format!("{:?}", equipment.junk));
    ui.text_disabled("misc   ");
    ui.same_line();
    ui.text_wrapped(format!("{:?}", equipment.misc));

    //ends the scroll window
    drop(_child);
    h
}