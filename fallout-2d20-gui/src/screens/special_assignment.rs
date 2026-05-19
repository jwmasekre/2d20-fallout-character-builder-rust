use std::collections::HashMap;

use fallout_2d20_core::{
    character::Character,
    constants::SPECIAL_LABELS,
    db::Db,
    states::{
        BackgroundState,
        EquipmentState,
        OriginState,
        PerkState,
        SkillState,
        SpecialArray,
        SpecialState,
    },
};
use imgui::Ui;
use sdl2::video::Window;
use crate::AppScreen;
use crate::theme::{render_text_wrapped, render_window};
//use crate::log_on_change;

pub fn render_special_assignment(
    ui: &Ui,
    window: &Window,
    state: &mut SpecialState,
    _db: &Db,
    character: &mut Character,
    screen: &mut AppScreen,
    origin_state: &mut OriginState,
    skill_state: &mut SkillState,
    perk_state: &mut PerkState,
    background_state: &mut BackgroundState,
    equipment_state: &mut EquipmentState,
) -> f32 {
    let Some((w, h, _token)) = render_window(ui, window, "##special_assignment", "Special Assignment", screen, origin_state, state, skill_state, perk_state, background_state, equipment_state, character)
        else { return 0.0 };

    ui.text("SPECIAL");
    ui.separator();
    ui.spacing();

    ui.text("Select Array:");
    ui.same_line();
    ui.set_next_item_width(260.0);

    if let Some(_cb) = ui.begin_combo("##array_select", state.selected_array.label()) {
        for array in [
            SpecialArray::Balanced,
            SpecialArray::Focused,
            SpecialArray::Specialized,
            SpecialArray::Custom,
        ] {
            let selected = state.selected_array == array;
            if ui.selectable_config(array.label()).selected(selected).build() {
                state.selected_array = array;
                //clear assignments if we changed arrays
                state.assignments = [None; 7];
                //state.values = [5; 7];
            }
        }
    }
    ui.spacing();

    if state.selected_array == SpecialArray::None {
        ui.text_disabled("Select an array to continue");
        return h
    }

    if state.selected_array == SpecialArray::Custom {
        let remaining = state.remaining_points(character);
        if remaining < 0 {
            render_text_wrapped(true, false, ui, &format!("Remaining Points: {}", remaining), 0.0, w);
        } else if remaining == 0 {
            render_text_wrapped(false,true, ui, &format!("Remaining Points: {}", remaining), 0.0, w);
        } else {
            ui.text_wrapped(&format!("Remaining Points: {}", remaining));
        }
        ui.spacing();
    }
    ui.separator();
    ui.spacing();

    let label_w = 110.0_f32;
    let val_w = 60.0_f32;

    //we want different experiences if the player selects a preset array vs custom
    match state.selected_array {
        SpecialArray::Custom => render_custom(ui, state, label_w, val_w, w, character),
        SpecialArray::Balanced | SpecialArray::Focused | SpecialArray::Specialized => render_preset(ui, state, label_w, val_w, w, character),
        SpecialArray::None => {},
    }
    return h
}

//custom array
fn render_custom(
    ui: &Ui,
    state: &mut SpecialState,
    label_w: f32,
    val_w: f32,
    _w: f32,
    character: &mut Character,
) {

    for (i, special) in SPECIAL_LABELS.iter().enumerate() {
        let mutant_stat = i == 0 || i == 2;
        ui.text(special);
        ui.same_line_with_pos(label_w);
        {
            let _dec_guard = (!state.can_dec[i]).then(|| ui.begin_disabled(true));
            //if ui.button(format!("-##dec_{}", stringify!(special))) {
            if ui.button(format!("-##dec_{}", i)) {
                character.special.mut_special_block()[i].value -= 1;
                state.values[i] -= 1;
                state.update(character)
            }
        }
        ui.same_line();

        ui.set_next_item_width(val_w);
        ui.text(format!("{:2}", state.values[i]));
        ui.same_line();

        {
            let _inc_guard = (!state.can_inc[i]).then(|| ui.begin_disabled(true));
            //if ui.button(format!("+##inc_{}", stringify!(special))) {
            if ui.button(format!("+##inc_{}", i)) {
                character.special.mut_special_block()[i].value += 1;
                state.values[i] += 1;
                state.update(character)
            }
        }
        ui.same_line();

        if state.gifted {
            let spec = character.special.special_block()[i].clone();
            let disabled = state.gifted_count >= 2 || spec.value >= spec.max;
            let _gifted_guard = disabled.then(|| ui.begin_disabled(true));
            let mut checked = spec.gifted;
            if ui.checkbox(format!("G##gifted_{}", i), &mut checked) {
                character.special.mut_special_block()[i].gifted = checked;
                character.special.mut_special_block()[i].value += if checked { 1 } else { -1 };
                state.update(character);
            }
            ui.same_line();
        } else {
            //clear gifted state on the character
            if character.special.mut_special_block()[i].gifted {
                character.special.mut_special_block()[i].gifted = false;
                character.special.mut_special_block()[i].value -= 1;
            }
        }
        let spec = character.special.special_block()[i].clone();
        let display = spec.value;
        let max = spec.max;
        let mutant = if mutant_stat && character.is_mutant() { 2 } else { 0 };
        let mod_val = if spec.gifted { 1 } else { 0 } + mutant + spec.trained;
        let mod_state = mod_val > 0;
        render_text_wrapped(!mod_state, mod_state, ui, &format!(" -> {} (+{})", display, mod_val), label_w, label_w + 900.0);

        if display >= max {
            ui.same_line();
            ui.text_disabled(&format!("[cap: {}]", max));
        }
        ui.spacing();
    }
}

fn render_preset(
    ui: &Ui,
    state: &mut SpecialState,
    label_w: f32,
    _w: f32,
    _val_w: f32,
    character: &mut Character,
) {
    let preset_values = match state.selected_array.values() {
        Some(v) => v,
        None => return,
    };
    let assigned_values: Vec<i32> = state.assignments
        .iter()
        .filter_map(|&v| v)
        .collect();
    ui.text_disabled("Assign each to a stat:");
    ui.spacing();

    ui.text("Available:");
    ui.same_line();
    let mut leftover_map: HashMap<i32,usize> = HashMap::new();
    for &v in preset_values.iter() {
        let used_count = assigned_values.iter().filter(|&&x| x == v).count();
        let total_count = preset_values.iter().filter(|&&x| x == v).count();
        let remaining = total_count - used_count;
        if remaining > 0 {
            leftover_map.insert(v, remaining);
        }
    }
    //sorting the hashmap since hashmaps are unsorted
    let mut leftover: Vec<_> = leftover_map.iter().collect();
    leftover.sort_by(|a, b| b.0.cmp(a.0));

    let mut instance = 0;
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
    draw_list.add_line(ui.cursor_screen_pos(), [ui.cursor_screen_pos()[0], ui.cursor_screen_pos()[1] + 16.0], debug_color[instance]).build();
    */
    for (val, num) in leftover {
        for _ in 0..*num {
            ui.same_line();
            instance += 1;
            let wrap = 28.0 * (instance as f32);
            let label_wrap = label_w + wrap;
            //draw_list.add_line(ui.cursor_screen_pos(), [ui.cursor_screen_pos()[0], ui.cursor_screen_pos()[1] + 16.0], debug_color[instance]).build();
            render_text_wrapped(false, true, ui, &format!("[{}]", val), label_w, label_wrap);
        }
    }

    ui.spacing();
    ui.separator();
    ui.spacing();

    for (i, special) in SPECIAL_LABELS.iter().enumerate() {
        let mutant_stat = i == 0 || i == 2;
        ui.text(special);
        ui.same_line_with_pos(label_w);
        let preset_values = match state.selected_array.values() {
            Some(v) => v,
            None => return,
        };
        let assigned = state.assignments[i];
        let max = character.special.special_block()[i].max.clone();

        ui.set_next_item_width(80.0);
        let combo_label = match assigned {
            Some(v) => format!("{}", v),
            None => "--".to_string(),
        };

        if let Some(_cb) = ui.begin_combo(format!("##assign_{}", i), &combo_label) {
            if ui.selectable_config("--").selected(assigned.is_none()).build() {
                state.assignments[i] = None;
                state.update(character);
                //log_on_change!(state.assignments);
            }
            let mut offered: Vec<i32> = preset_values.to_vec();
            offered.sort_unstable_by(|a,b| b.cmp(a));
            offered.dedup();
            for &v in &offered {
                let used_elsewhere = state.assignments
                    .iter()
                    .enumerate()
                    .filter(|&(e, &av)| e != i && av == Some(v))
                    .count();
                let total = preset_values
                    .iter()
                    .filter(|&&x| x == v)
                    .count();
                let available = total > used_elsewhere;
                let over_cap = v > max;
                let disabled = !available || over_cap;
                {
                    let _opt_guard = disabled.then(|| ui.begin_disabled(true));
                    let is_selected = assigned == Some(v);
                    let label = if over_cap {
                        format!("{} (exceeds cap: {}) ", v, max)
                    } else {
                        format!("{}", v)
                    };
                    let mutant = if mutant_stat && character.is_mutant() { 2 } else { 0 };
                    if ui.selectable_config(&label).selected(is_selected).build() {
                        state.assignments[i] = Some(v);
                        //might have issues with special training here idk
                        character.special.mut_special_block()[i].value = v + mutant;
                        state.update(character);
                    }
                }
            }
        }
        ui.same_line();

        if state.gifted {
            let disabled = state.gifted_count >= 2 || assigned.map(|v| v >= max).unwrap_or(false) || assigned.is_none();
            {
                let _gift_guard = disabled.then(|| ui.begin_disabled(true));
                let mut checked = character.special.special_block()[i].gifted.clone();
                if ui.checkbox(format!("G##gifted_{}", i), &mut checked) {
                    character.special.mut_special_block()[i].gifted = checked;
                    character.special.mut_special_block()[i].value += if checked { 1 } else { -1 };
                    state.update(character);
                }
            }
            ui.same_line();
        } else {
            //clear gifted state on the character
            if character.special.mut_special_block()[i].gifted {
                character.special.mut_special_block()[i].gifted = false;
                character.special.mut_special_block()[i].value -= 1;
            }
        }
        let display = character.special.special_block()[i].value;
        let mod_val = display - assigned.unwrap_or(0);

        if assigned.is_some() {
            let mod_state = mod_val > 0;
            render_text_wrapped(!mod_state, mod_state, ui, &format!(" -> {} (+{})", display, mod_val), label_w, label_w + 900.0);
        } else {
            ui.text_disabled(" -> ?");
        }
        ui.spacing();
    }
}