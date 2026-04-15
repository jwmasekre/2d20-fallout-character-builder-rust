use imgui::Ui;
use sdl2::video::Window;
use crate::db::Db;
use crate::character::Character;
use crate::theme::{render_text_wrapped, render_window};

//list of our array options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpecialArray {
    None,
    Balanced,   // 6,6,6,6,6,5,5
    Focused,    // 8,7,6,6,5,4,4
    Specialized,// 9,8,5,5,5,4,4
    Custom,
}

impl SpecialArray {
    //functions that look up the drop-down label...
    fn label(&self) -> &'static str {
        match self {
            Self::None        => "Select SPECIAL array...",
            Self::Balanced    => "Balanced    (6,6,6,6,6,5,5)",
            Self::Focused     => "Focused     (8,7,6,6,5,4,4)",
            Self::Specialized => "Specialized (9,8,5,5,5,4,4)",
            Self::Custom      => "Custom",
        }
    }
    //...and actual values of each array
    fn values(&self) -> Option<[i32; 7]> {
        match self {
            Self::Balanced    => Some([6,6,6,6,6,5,5]),
            Self::Focused     => Some([8,7,6,6,5,4,4]),
            Self::Specialized => Some([9,8,5,5,5,4,4]),
            _ => None,
        }
    }
}
//have a reference for stats to refer to (there's an enum in character.rs?)
pub const SPECIAL_LABELS: [&str; 7] = ["Strength", "Perception", "Endurance", "Charisma", "Intelligence", "Agility", "Luck"];

//track validity states (no array, )
pub struct SpecialState {
    selected_array: SpecialArray,
    assignments: [Option<i32>; 7],
    values: [i32; 7],
    gifted: bool,
    gifted_count: i32,
    trained: i32,
    trained_count: i32,
    total: i32,
}

impl SpecialState {
    pub fn new() -> Self {
        Self {
            selected_array: SpecialArray::None,
            assignments: [None; 7],
            values: [5; 7],
            gifted: false,
            gifted_count: 0,
            trained: 0,
            trained_count: 0,
            total: 35,
        }
    }
    pub fn update(&self, character: &Character) -> Self {
        let selected_array = self.selected_array;
        let assignments = self.assignments;
        let values = self.values;
        //check if the character is gifted
        let gifted = character.is_gifted();
        //count number of gifted selections
        let gifted_count = character.special.special_block().iter().map(|s| s.gifted).filter(|&b| b).count() as i32;
        //check how much intense training the character has
        let trained = character.perks.iter().find(|p| p.id == 45).unwrap().ranks;
        //check how many times intense training has been applied
        let trained_count: i32 = character.special.special_block().iter().map(|s| s.trained).sum();
        let total: i32 = character.special.special_block().iter().map(|s| s.value).sum();
        Self {
            selected_array,
            assignments,
            values,
            gifted,
            gifted_count,
            trained,
            trained_count,
            total,
        }
    }
    pub fn is_complete(&self, character: &Character) -> bool {
        (if self.gifted { self.gifted_count == 2 } else { self.gifted_count == 0 }) &&
            self.trained == self.trained_count && self.total == 40 + self.trained_count + self.gifted_count + if character.is_mutant() { 4 } else { 0 }
    }
    pub fn remaining_points(&self, character: &Character) -> i32 {
        40 + if self.gifted { 2 } else { 0 } + self.trained + if character.is_mutant() { 4 } else { 0 } - self.total
    }
}

pub fn render_special_assignment(
    ui: &Ui,
    window: &Window,
    state: &mut SpecialState,
    db: &Db,
    character: &mut Character,
) -> f32 {
    let (w, h) = render_window(ui, window, "##special_assignment", "Special Assignment");

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
                state.values = [5; 7];
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
        _ => render_preset(ui, state, label_w, val_w, w, character)
    }
    return h
}

//custom array
fn render_custom(
    ui: &Ui,
    state: &mut SpecialState,
    label_w: f32,
    val_w: f32,
    w: f32,
    character: &mut Character,
) {
    //this macro may not be necessary if this works as i intend

/*    
    //either i duplicate code 7 times for the custom special rendering, or i create this once and call the macro with each special stat
    //this is because character.special is 7 structs, rather than a Vec<specialstatblock; 7>
    macro_rules! build_special_custom {
        (
            $ui:expr,
            $state:expr,
            $character:expr,
            $label_w:expr,
            $val_w:expr,
            $w:expr,
            $index:expr,
            $field:ident,
            $mutant_stat:expr
        ) => {{
            $ui.text(SPECIAL_LABELS[$index]);
            $ui.same_line_with_pos($label_w);

            let _dec_guard = (!$character.special.$field.can_decrease($character))
                .then(|| $ui.begin_disabled(true));
            if $ui.button(format!("-##dec_{}", stringify!($field))) {
                $character.special.$field.value -= 1;
                $state.update($character);
                $state.values[$index] -= 1;
            }
            $ui.same_line();

            $ui.set_next_item_width($val_w);
            $ui.text(format!("{:2}", $state.values[$index]));
            $ui.same_line();

            let _inc_guard = (!$character.special.$field.can_increase($state, $character))
                .then(|| $ui.begin_disabled(true));
            if $ui.button(format!("+##inc_{}", stringify!($field))) {
                $character.special.$field.value += 1;
                $state.update($character);
                $state.values[$index] += 1;
            }
            $ui.same_line();

            if $state.gifted {
                let disabled = $state.gifted_count >= 2
                    || $character.special.$field.value >= $character.special.$field.max;
                let _gifted_guard = disabled.then(|| $ui.begin_disabled(true));
                let mut checked = $character.special.$field.gifted;
                if $ui.checkbox(format!("G##gifted_{}", stringify!($field)), &mut checked) {
                    $character.special.$field.gifted = checked;
                    $character.special.$field.value += if checked { 1 } else { -1 };
                    $state.update($character);
                }
                $ui.same_line();
            }

            let display = $character.special.$field.value;
            let max     = $character.special.$field.max;
            let mutant  = if $mutant_stat && $character.is_mutant() { 2 } else { 0 };
            let mod_val = if $character.special.$field.gifted { 1 } else { 0 }
                + mutant + $character.special.$field.trained;
            let mod_state = mod_val > 0;
            render_text_wrapped(
                !mod_state, mod_state, $ui,
                &format!("-> {} (+{})", display, mod_val),
                $label_w, $label_w + $w,
            );

            if display >= max {
                $ui.same_line();
                $ui.text_disabled(&format!("[cap: {}]", max));
            }
            $ui.spacing();
        }};
    }

    build_special_custom!(ui, state, character, label_w, val_w, w, 0, strength, true);
    build_special_custom!(ui, state, character, label_w, val_w, w, 1, perception, false);
    build_special_custom!(ui, state, character, label_w, val_w, w, 2, endurance, true);
    build_special_custom!(ui, state, character, label_w, val_w, w, 3, charisma, false);
    build_special_custom!(ui, state, character, label_w, val_w, w, 4, intelligence, false);
    build_special_custom!(ui, state, character, label_w, val_w, w, 5, agility, false);
    build_special_custom!(ui, state, character, label_w, val_w, w, 6, luck, false);
*/

    let char_clone = character.clone();
    for (i, special) in character.special.mut_special_block().iter_mut().enumerate() {

        let mutant_stat = [0,2].contains(&i.into());
        ui.text(SPECIAL_LABELS[i]);
        ui.same_line_with_pos(label_w);

        let _dec_guard = (!special.can_decrease(&char_clone))
            .then(|| ui.begin_disabled(true));
        if ui.button(format!("-##dec_{}", stringify!(SPECIAL_LABELS[i]))) {
            special.value -= 1;
            state.values[i] -= 1;
        }
        ui.same_line();

        ui.set_next_item_width(val_w);
        ui.text(format!("{:2}", state.values[i]));
        ui.same_line();

        let _inc_guard = (!special.can_increase(state, &char_clone))
            .then(|| ui.begin_disabled(true));
        if ui.button(format!("+##inc_{}", stringify!(SPECIAL_LABELS[i]))) {
            special.value += 1;
            state.values[i] += 1;
        }
        ui.same_line();

        if state.gifted {
            let disabled = state.gifted_count >= 2
                || special.value >= special.max;
            let _gifted_guard = disabled.then(|| ui.begin_disabled(true));
            let mut checked = special.gifted;
            if ui.checkbox(format!("G##gifted_{}", stringify!(SPECIAL_LABELS[i])), &mut checked) {
                special.gifted = checked;
                special.value += if checked { 1 } else { -1 };
            }
            ui.same_line();
        }

        let display = special.value;
        let max     = special.max;
        let mutant  = if mutant_stat && char_clone.is_mutant() { 2 } else { 0 };
        let mod_val = if special.gifted { 1 } else { 0 }
            + mutant + special.trained;
        let mod_state = mod_val > 0;
        render_text_wrapped(
            !mod_state, mod_state, ui,
            &format!("-> {} (+{})", display, mod_val),
            label_w, label_w + w,
        );

        if display >= max {
            ui.same_line();
            ui.text_disabled(&format!("[cap: {}]", max));
        }
        ui.spacing();
    }
    state.update(character);
}

fn render_preset(
    ui: &Ui,
    state: &mut SpecialState,
    label_w: f32,
    w: f32,
    val_w: f32,
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
    let mut sorted = preset_values;
    sorted.sort_unstable_by(|a,b| b.cmp(a));
    for &v in sorted.iter() {
        let used_count = assigned_values.iter().filter(|&&x| x == v).count();
        let total_count = preset_values.iter().filter(|&&x| x == v).count();
        let remaining = total_count - used_count;
        if remaining > 0 {
            for i in 0..remaining {
                ui.same_line();
                render_text_wrapped(false, true, ui, &format!("[{}]", v), label_w, w);
            }
        }
    }
    ui.spacing();
    ui.separator();
    ui.spacing();
/*
    macro_rules! build_special_preset {
        (
            $ui:expr,
            $state:expr,
            $character:expr,
            $label_w:expr,
            $val_w:expr,
            $w:expr,
            $index:expr,
            $field:ident,
            $mutant_stat:expr
        ) => {{
            let preset_values = match $state.selected_array.values() {
                Some(v) => v,
                None => return,
            };
            let assigned = $state.assignments[$index];
            let max = $character.special.$field.max;
            $ui.text(SPECIAL_LABELS[$index]);
            $ui.same_line_with_pos($label_w);

            $ui.set_next_item_width(80.0);
            let combo_label = match assigned {
                Some(v) => format!("{}", v),
                None => "--".to_string(),
            };

            if let Some(_cb) = $ui.begin_combo(format!("##assign_{}", stringify!($field)), &combo_label) {
                if $ui.selectable_config("--").selected(assigned.is_none()).build() {
                    $state.assignments[$index] = None;
                }
                let mut offered: Vec<i32> = preset_values.to_vec();
                offered.sort_unstable_by(|a,b| b.cmp(a));
                offered.dedup();
                for &v in &offered {
                    let used_elsewhere = $state.assignments
                        .iter()
                        .enumerate()
                        .filter(|&(i,&av)| i != $index && av == Some(v))
                        .count();
                    let total = preset_values
                        .iter()
                        .filter(|&&x| x == v)
                        .count();
                    let available = total > used_elsewhere;
                    let over_cap = v > max;
                    let disabled = !available || over_cap;

                    let _opt_guard = disabled.then(|| $ui.begin_disabled(true));
                    let is_selected = assigned == Some(v);
                    let label = if over_cap {
                        format!("{} (exceeds cap {}", v, max)
                    } else {
                        format!("{}", v)
                    };
                    let mutant  = if $mutant_stat && $character.is_mutant() { 2 } else { 0 };
                    if $ui.selectable_config(&label).selected(is_selected).build() {
                        $state.assignments[$index] = Some(v);
                        $character.special.$field.value = v + mutant;
                        $state.update($character);
                    }
                }
            }
            $ui.same_line();

            if $state.gifted {
                let disabled = $state.gifted_count >= 2 || assigned.map(|v| v >= max).unwrap_or(false) || assigned.is_none();
                let _gift_guard = disabled.then(|| $ui.begin_disabled(true));
                let mut checked = $character.special.$field.gifted;
                if $ui.checkbox(format!("G##gifted_{}", stringify!($field)), &mut checked) {
                    $character.special.$field.gifted = checked;
                    $character.special.$field.value += if checked { 1 } else { -1 };
                    $state.update($character);
                }
                $ui.same_line();
            }

            let display = $character.special.$field.value;
            let mod_val = display - assigned.unwrap_or(0);

            if assigned.is_some() {
                let mod_state = mod_val > 0;
                render_text_wrapped(!mod_state, mod_state, $ui, &format!("-> {} (+{})", display, mod_val), $label_w, $label_w + $w);
            } else {
                $ui.text_disabled("-> ?")
            }
            $ui.spacing();
        }};
    }
    build_special_preset!(ui, state, character, label_w, val_w, w, 0, strength, true);
    build_special_preset!(ui, state, character, label_w, val_w, w, 1, perception, false);
    build_special_preset!(ui, state, character, label_w, val_w, w, 2, endurance, true);
    build_special_preset!(ui, state, character, label_w, val_w, w, 3, charisma, false);
    build_special_preset!(ui, state, character, label_w, val_w, w, 4, intelligence, false);
    build_special_preset!(ui, state, character, label_w, val_w, w, 5, agility, false);
    build_special_preset!(ui, state, character, label_w, val_w, w, 6, luck, false);
*/

    let char_clone = character.clone();
    for (i, special) in character.special.mut_special_block().iter_mut().enumerate() {

        let mutant_stat = [0,2].contains(&i.into());
        ui.text(SPECIAL_LABELS[i]);
        ui.same_line_with_pos(label_w);
        let preset_values = match state.selected_array.values() {
            Some(v) => v,
            None => return,
        };
        let assigned = state.assignments[i];
        let max = special.max;
        ui.text(SPECIAL_LABELS[i]);
        ui.same_line_with_pos(label_w);

        ui.set_next_item_width(80.0);
        let combo_label = match assigned {
            Some(v) => format!("{}", v),
            None => "--".to_string(),
        };

        if let Some(_cb) = ui.begin_combo(format!("##assign_{}", stringify!(SPECIAL_LABELS[i])), &combo_label) {
            if ui.selectable_config("--").selected(assigned.is_none()).build() {
                state.assignments[i] = None;
            }
            let mut offered: Vec<i32> = preset_values.to_vec();
            offered.sort_unstable_by(|a,b| b.cmp(a));
            offered.dedup();
            for &v in &offered {
                let used_elsewhere = state.assignments
                    .iter()
                    .enumerate()
                    .filter(|&(i,&av)| i != i && av == Some(v))
                    .count();
                let total = preset_values
                    .iter()
                    .filter(|&&x| x == v)
                    .count();
                let available = total > used_elsewhere;
                let over_cap = v > max;
                let disabled = !available || over_cap;

                let _opt_guard = disabled.then(|| ui.begin_disabled(true));
                let is_selected = assigned == Some(v);
                let label = if over_cap {
                    format!("{} (exceeds cap {}", v, max)
                } else {
                    format!("{}", v)
                };
                let mutant  = if mutant_stat && char_clone.is_mutant() { 2 } else { 0 };
                if ui.selectable_config(&label).selected(is_selected).build() {
                    state.assignments[i] = Some(v);
                    special.value = v + mutant;
                }
            }
        }
        ui.same_line();

        if state.gifted {
            let disabled = state.gifted_count >= 2 || assigned.map(|v| v >= max).unwrap_or(false) || assigned.is_none();
            let _gift_guard = disabled.then(|| ui.begin_disabled(true));
            let mut checked = special.gifted;
            if ui.checkbox(format!("G##gifted_{}", stringify!(SPECIAL_LABELS[i])), &mut checked) {
                special.gifted = checked;
                special.value += if checked { 1 } else { -1 };
            }
            ui.same_line();
        }

        let display = special.value;
        let mod_val = display - assigned.unwrap_or(0);

        if assigned.is_some() {
            let mod_state = mod_val > 0;
            render_text_wrapped(!mod_state, mod_state, ui, &format!("-> {} (+{})", display, mod_val), label_w, label_w + w);
        } else {
            ui.text_disabled("-> ?")
        }
        ui.spacing();
    }
    state.update(character);
}