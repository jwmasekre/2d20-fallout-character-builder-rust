use crate::{AppScreen, theme::render_window};
use fallout_2d20_core::{
    background_slots::{
        render_apparel_slot,
        render_consumable_slot,
        render_robot_module_slot,
        render_weapon_slot
    },
    character::{
        Background,
        Character
    },
    db::Db,
    states::{
        BackgroundState,
        EquipmentState,
        OriginState,
        PerkState,
        ReviewState,
        SkillState,
        SpecialState
    }
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
) -> f32 {
    let Some((w, h, _token)) = render_window(ui, window, "##background_select", "Background Select", screen, origin, special, skill, perk, state, equipment, character)
        else { return 0.0 };

    ui.text("BACKGROUND");
    ui.separator();
    ui.spacing();

    ui.text("Background:");
    ui.same_line();
    ui.set_next_item_width(280.0);
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
    let list_h = h - 140.0;
    let Some(_child) = ui.child_window("##equip_scroll")
        .size([w - 16.0, list_h])
        .begin()
    else { return h };

    ui.separator();
    //weapons
    if !bg.weapon_slots.is_empty() {
        ui.text("WEAPONS");
        ui.separator();
        ui.spacing();
        for (i, slot) in bg.weapon_slots.iter().enumerate() {
            state.equipment_changed = render_weapon_slot(ui, i, slot, &mut state.weapon_selections[i], &bg.ammo);
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
            state.equipment_changed = render_apparel_slot(ui, i, slot, &mut state.apparel_selections[i]);
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
            state.equipment_changed = render_consumable_slot(ui, i, slot, &mut state.consumable_selections[i]);
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
            state.equipment_changed = render_robot_module_slot(ui, i, slot, &mut state.robot_module_selections[i]);
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
    let mut table_vec: Vec<String> = vec![];
    if bg.trinket > 0  { table_vec.push(format!("  Trinket x{}", bg.trinket)); }
    if bg.food > 0     { table_vec.push(format!("  Food x{}", bg.food)); }
    if bg.forage > 0   { table_vec.push(format!("  Forage x{}", bg.forage)); }
    if bg.bev > 0      { table_vec.push(format!("  Beverages x{}", bg.bev)); }
    if bg.chem > 0     { table_vec.push(format!("  Chems x{}", bg.chem)); }
    if bg.ammo_count > 0  { table_vec.push(format!("  Ammo x{}", bg.ammo_count)); }
    if bg.aid > 0      { table_vec.push(format!("  Aid x{}", bg.aid)); }
    if bg.odd > 0      { table_vec.push(format!("  Oddities x{}", bg.odd)); }
    if bg.outcast > 0  { table_vec.push(format!("  Outcast Equipment x{}", bg.outcast)); }
    if bg.junk > 0     { table_vec.push(format!("  Junk x{}", bg.junk)); }
    if table_vec.len() > 0 {
        ui.text("Table Rolls:");
        for i in 0..table_vec.len() {
            ui.text(table_vec[i].clone());
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