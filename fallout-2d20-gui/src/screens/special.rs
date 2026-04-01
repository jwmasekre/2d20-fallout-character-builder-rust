use imgui::Ui;
use sdl2::video::Window;
use crate::{AppScreen, BAR_HEIGHT};
use crate::screens::new_character::{ render_text_wrapped };
//use crate::theme::Theme;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpecialArray {
    None,
    Balanced,   // 6,6,6,6,6,5,5
    Focused,    // 8,7,6,6,5,4,4
    Specialized,// 9,8,5,5,5,4,4
    Custom,
}

impl SpecialArray {
    fn label(&self) -> &'static str {
        match self {
            Self::None        => "Select SPECIAL array...",
            Self::Balanced    => "Balanced    (6,6,6,6,6,5,5)",
            Self::Focused     => "Focused     (8,7,6,6,5,4,4)",
            Self::Specialized => "Specialized (9,8,5,5,5,4,4)",
            Self::Custom      => "Custom",
        }
    }

    fn values(&self) -> Option<[i32; 7]> {
        match self {
            Self::Balanced    => Some([6,6,6,6,6,5,5]),
            Self::Focused     => Some([8,7,6,6,5,4,4]),
            Self::Specialized => Some([9,8,5,5,5,4,4]),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MutantType {
    None,
    StandardSuperMutant,
    Nightkin,
}

// ── SPECIAL stat indices
const S: usize = 0; // Strength
const P: usize = 1; // Perception
const E: usize = 2; // Endurance
const C: usize = 3; // Charisma
const I: usize = 4; // Intelligence
const A: usize = 5; // Agility
const L: usize = 6; // Luck

const STAT_LABELS: [&str; 7] = [
    "Strength", "Perception", "Endurance",
    "Charisma", "Intelligence", "Agility", "Luck",
];

const SPECIAL_POINTS: i32 = 40;
const STAT_MIN: i32 = 4;
const STAT_MAX: i32 = 10;

// ── State ─────────────────────────────────────────────────────────────────────

pub struct SpecialState {
    pub selected_array: SpecialArray,

    /// The assigned values for preset arrays: which slot each preset value goes to.
    /// For preset arrays, `assignments[value_index] = Some(stat_index)`.
    /// We store the stat assignments as a mapping from stat → assigned value.
    pub preset_assignments: [Option<i32>; 7], // stat_index → assigned value (None = unassigned)

    /// Custom mode raw values (before modifiers)
    pub custom_values: [i32; 7],

    /// Gifted trait — which two stats get +1
    pub is_gifted: bool,
    pub gifted_selected: [bool; 7], // max 2 true

    /// Mutant type
    pub mutant_type: MutantType,
}

impl SpecialState {
    pub fn new(is_gifted: bool, mutant_type: MutantType) -> Self {
        Self {
            selected_array: SpecialArray::None,
            preset_assignments: [None; 7],
            custom_values: [5; 7],
            is_gifted,
            gifted_selected: [false; 7],
            mutant_type,
        }
    }

    /// Base values before modifiers (what the player set)
    pub fn base_values(&self) -> [i32; 7] {
        match self.selected_array {
            SpecialArray::Custom => self.custom_values,
            _ => {
                let mut out = [0i32; 7];
                for (i, &v) in self.preset_assignments.iter().enumerate() {
                    out[i] = v.unwrap_or(0);
                }
                out
            }
        }
    }

    /// Stat cap for a given stat considering mutant type
    fn stat_max(&self, stat: usize) -> i32 {
        match self.mutant_type {
            MutantType::StandardSuperMutant => match stat {
                I | C => 6,
                S | E => 12,
                _ => STAT_MAX,
            },
            MutantType::Nightkin => match stat {
                I | C => 8,
                S | E => 12,
                _ => STAT_MAX,
            },
            MutantType::None => STAT_MAX,
        }
    }

    /// Modifier for a given stat (Gifted +1, mutant STR/END +2)
    pub fn modifier(&self, stat: usize) -> i32 {
        let mut m = 0;
        if self.is_gifted && self.gifted_selected[stat] { m += 1; }
        if matches!(self.mutant_type, MutantType::StandardSuperMutant | MutantType::Nightkin) {
            if stat == S || stat == E { m += 2; }
        }
        m
    }

    /// Final displayed value
    pub fn display_value(&self, stat: usize) -> i32 {
        self.base_values()[stat] + self.modifier(stat)
    }

    pub fn total_base(&self) -> i32 {
        self.base_values().iter().sum()
    }

    pub fn remaining_points(&self) -> i32 {
        SPECIAL_POINTS - self.total_base()
    }

    pub fn gifted_count(&self) -> usize {
        self.gifted_selected.iter().filter(|&&v| v).count()
    }
}

// ── Render ────────────────────────────────────────────────────────────────────

pub fn render_special(
    ui: &Ui,
    window: &Window,
    state: &mut SpecialState,
    screen: &mut AppScreen,
    //theme: &Theme,
) {
    let (win_w, win_h) = window.size();
    let content_h = win_h as f32 - BAR_HEIGHT;
    let w = (win_w as f32 * 0.65).min(860.0);
    let h = win_h as f32 * 0.85;

    let Some(_window_token) = ui.window("##special")
        .title_bar(false)
        .resizable(false)
        .movable(false)
        .size([w, h], imgui::Condition::Always)
        .position(
            [(win_w as f32 - w) * 0.5, BAR_HEIGHT + (content_h - h) * 0.5], imgui::Condition::Always,
        )
        .begin()
    else {
        return;
    };

    ui.text("SPECIAL");
    ui.separator();
    ui.spacing();

    // ── Array selector ────────────────────────────────────────────
    ui.text("Array:");
    ui.same_line();
    ui.set_next_item_width(260.0);
    if let Some(_cb) = ui.begin_combo("##array_select", state.selected_array.label()) {
        for variant in [
            SpecialArray::Balanced,
            SpecialArray::Focused,
            SpecialArray::Specialized,
            SpecialArray::Custom,
        ] {
            let selected = state.selected_array == variant;

            if ui.selectable_config(variant.label()).selected(selected).build() {
                state.selected_array = variant;
                // Reset assignments when switching arrays
                state.preset_assignments = [None; 7];
                if variant == SpecialArray::Custom {
                    state.custom_values = [5; 7];
                }
            }
        }
    }
    //ui.text(format!("DBG: {:?}", state.selected_array));

    ui.spacing();

    if state.selected_array == SpecialArray::None {
        ui.text_disabled("Select an array to continue.");
        render_footer(ui, h, screen);
        return;
    }

    // ── Remaining points (custom only) ────────────────────────────
    if state.selected_array == SpecialArray::Custom {
        let remaining = state.remaining_points();
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

    // ── Stat rows ─────────────────────────────────────────────────
    let label_w = 110.0_f32;
    let val_w = 60.0_f32;

    match state.selected_array {
        SpecialArray::Custom => render_custom_stats(ui, state, label_w, val_w, w),
        _ => render_preset_stats(ui, state, label_w, val_w, w),
    }

    render_footer(ui, h, screen);

    /*
    ui.window("##special")
        .title_bar(false)
        .resizable(false)
        .movable(false)
        .size([w, h], imgui::Condition::Always)
        .position(
            [(win_w as f32 - w) * 0.5, BAR_HEIGHT + (content_h - h) * 0.5],
            imgui::Condition::Always,
        )
        .build(|| {
            ui.text("SPECIAL");
            ui.separator();
            ui.spacing();

            // ── Array selector ────────────────────────────────────────────
            ui.text("Array:");
            ui.same_line();
            ui.set_next_item_width(260.0);
            let current_label = state.selected_array.label();
            if let Some(_cb) = ui.begin_combo("##array_select", current_label) {
                for variant in [
                    SpecialArray::Balanced,
                    SpecialArray::Focused,
                    SpecialArray::Specialized,
                    SpecialArray::Custom,
                ] {
                    let selected = state.selected_array == variant;
                    let item_id = format!("{}##arr_{:?}", variant.label(), variant);
                    if ui.selectable_config(&item_id).selected(selected).build() {
                        state.selected_array = variant;
                        // Reset assignments when switching arrays
                        state.preset_assignments = [None; 7];
                        if variant == SpecialArray::Custom {
                            state.custom_values = [5; 7];
                        }
                    }
                }
            }
            ui.text(format!("DBG: {:?}", state.selected_array));

            ui.spacing();

            if state.selected_array == SpecialArray::None {
                ui.text_disabled("Select an array to continue.");
                render_footer(ui, h, screen);
                return;
            }

            // ── Remaining points (custom only) ────────────────────────────
            if state.selected_array == SpecialArray::Custom {
                let remaining = state.remaining_points();
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

            // ── Stat rows ─────────────────────────────────────────────────
            let label_w = 110.0_f32;
            let val_w = 60.0_f32;

            match state.selected_array {
                SpecialArray::Custom => render_custom_stats(ui, state, label_w, val_w, w),
                _ => render_preset_stats(ui, state, label_w, val_w, w),
            }

            render_footer(ui, h, screen);
        });
        */

}

// ── Custom array rendering ────────────────────────────────────────────────────

fn render_custom_stats(
    ui: &Ui,
    state: &mut SpecialState,
    //theme: &Theme,
    label_w: f32,
    val_w: f32,
    win_w: f32,
) {
    let remaining = state.remaining_points();
    let gifted_count = state.gifted_count();

    for si in 0..7 {
        let base = state.custom_values[si];
        let max = state.stat_max(si);
        let can_increase = base < max && (remaining > 0 || false);
        let can_decrease = base > STAT_MIN;

        // Label
        ui.text(STAT_LABELS[si]);
        ui.same_line_with_pos(label_w);

        // Decrement button
        let _dec_guard = (!can_decrease).then(|| ui.begin_disabled(true));
        if ui.button(format!("-##dec_{}", si)) {
            state.custom_values[si] -= 1;
        }
        ui.same_line();

        // Value display
        ui.set_next_item_width(val_w);
        ui.text(format!("{:2}", base));
        ui.same_line();

        // Increment button
        let blocked = base >= max || remaining <= 0;
        let _inc_guard = blocked.then(|| ui.begin_disabled(true));
        if ui.button(format!("+##inc_{}", si)) {
            state.custom_values[si] += 1;
        }
        ui.same_line();

        // Gifted checkbox
        if state.is_gifted {
            let at_limit = !state.gifted_selected[si] && gifted_count >= 2;
            let at_cap = base >= max; // can't gift a stat already at cap
            let disabled = at_limit || at_cap;
            let _gift_guard = (at_limit || at_cap).then(|| ui.begin_disabled(true));
            let mut checked = state.gifted_selected[si];
            if ui.checkbox(format!("G##gifted_{}", si), &mut checked) {
                state.gifted_selected[si] = checked;
            }
            ui.same_line();
        }

        // Display value (with modifiers)
        let display = state.display_value(si);
        let mod_val = state.modifier(si);
        let mod_state = mod_val > 0;
        
        render_text_wrapped(!mod_state, mod_state, ui, &format!("→ {} (+{})", display, mod_val), label_w, label_w + val_w);

        // Cap warning
        if display > max {
            ui.same_line();
            render_text_wrapped(true, false, ui, &format!("[cap: {}]", max), label_w, label_w + val_w);
        }

        ui.spacing();
    }
}

// ── Preset array rendering ────────────────────────────────────────────────────

fn render_preset_stats(
    ui: &Ui,
    state: &mut SpecialState,
    //theme: &Theme,
    label_w: f32,
    _val_w: f32,
    _win_w: f32,
) {
    let preset_values = match state.selected_array.values() {
        Some(v) => v,
        None => return,
    };
    let gifted_count = state.gifted_count();

    // Show which values are available to assign
    // Build "already used" set
    let assigned_values: Vec<i32> = state.preset_assignments.iter()
        .filter_map(|&v| v)
        .collect();

    // Instruction
    render_text_wrapped(true, false, ui, "Assign each value to a SPECIAL stat:", label_w, label_w + _val_w);
    ui.spacing();

    // Available values pool (sorted descending, show counts)
    ui.text("Available:");
    ui.same_line();
    let mut sorted = preset_values;
    sorted.sort_unstable_by(|a, b| b.cmp(a));
    for &v in sorted.iter() {
        let used_count = assigned_values.iter().filter(|&&x| x == v).count();
        let total_count = preset_values.iter().filter(|&&x| x == v).count();
        let remaining = total_count - used_count;
        if remaining > 0 {
            ui.same_line();
            render_text_wrapped(false, true, ui, &format!("[{}]", v), label_w, label_w + _val_w);
        }
    }
    ui.spacing();
    ui.separator();
    ui.spacing();

    for si in 0..7 {
        let assigned = state.preset_assignments[si];
        let max = state.stat_max(si);

        ui.text(STAT_LABELS[si]);
        ui.same_line_with_pos(label_w);

        // Combo to pick which value to assign
        ui.set_next_item_width(80.0);
        let combo_label = match assigned {
            Some(v) => format!("{}", v),
            None => "--".to_string(),
        };

        if let Some(_cb) = ui.begin_combo(format!("##assign_{}", si), &combo_label) {
            // Unassign option
            if ui.selectable_config("--").selected(assigned.is_none()).build() {
                state.preset_assignments[si] = None;
            }
            // Each unique value in preset
            let mut offered: Vec<i32> = preset_values.to_vec();
            offered.sort_unstable_by(|a, b| b.cmp(a));
            offered.dedup();
            for &v in &offered {
                // How many of this value are available (not assigned to OTHER stats)?
                let used_elsewhere = state.preset_assignments.iter().enumerate()
                    .filter(|&(i, &av)| i != si && av == Some(v))
                    .count();
                let total = preset_values.iter().filter(|&&x| x == v).count();
                let available = total > used_elsewhere;

                let over_cap = v > max;
                let disabled = !available || over_cap;

                let _opt_guard = disabled.then(|| ui.begin_disabled(true));
                let is_selected = assigned == Some(v);
                let label = if over_cap {
                    format!("{} (exceeds cap {})", v, max)
                } else {
                    format!("{}", v)
                };
                if ui.selectable_config(&label).selected(is_selected).build() {
                    state.preset_assignments[si] = Some(v);
                }
            }
        }

        ui.same_line();

        // Gifted checkbox
        if state.is_gifted {
            let at_limit = !state.gifted_selected[si] && gifted_count >= 2;
            let at_cap = assigned.map(|v| v >= max).unwrap_or(false);
            let disabled = at_limit || at_cap || assigned.is_none();
            let _gift_guard = disabled.then(|| ui.begin_disabled(true));
            let mut checked = state.gifted_selected[si];
            if ui.checkbox(format!("G##gifted_{}", si), &mut checked) {
                state.gifted_selected[si] = checked;
            }
            ui.same_line();
        }

        // Display value
        let base = assigned.unwrap_or(0);
        let mod_val = state.modifier(si);
        let display = base + mod_val;

        if assigned.is_some() {
            let mod_state = mod_val > 0;
            render_text_wrapped(!mod_state, mod_state, ui, &format!("→ {} (+{})", display, mod_val), label_w, label_w + _val_w);
        } else {
            ui.text_disabled("→ ?");
        }

        ui.spacing();
    }

    // Show if all assigned
    let all_assigned = state.preset_assignments.iter().all(|v| v.is_some());
    if !all_assigned {
        ui.spacing();
        render_text_wrapped(true, false, ui, "Assign all 7 stats to continue.", label_w, label_w + _val_w);
    }
}

// ── Footer ────────────────────────────────────────────────────────────────────

fn render_footer(ui: &Ui, win_h: f32, screen: &mut AppScreen) {
    let footer_y = win_h - 48.0;
    ui.set_cursor_pos([16.0, footer_y]);
    if ui.button("< Back") {
        *screen = AppScreen::NewCharacter;
    }
    ui.same_line();
    if ui.button("Next >") {
        // TODO: advance to next step
    }
}