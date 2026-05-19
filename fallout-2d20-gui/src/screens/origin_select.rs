use imgui::Ui;
use sdl2::video::Window;
use crate::theme::{render_text_wrapped, render_window};

use fallout_2d20_core::{
    character::{
        Character,
        Trait,
    }, db::Db, states::{
        BackgroundState, EquipmentState, OriginState, PerkState, SkillState, SpecialState
    }, structs::{AppConfig, AppScreen}
};

pub fn render_origin_select(
    ui: &Ui,
    window: &Window,
    state: &mut OriginState,
    db: &Db,
    character: &mut Character,
    special_state: &mut SpecialState,
    skill_state: &mut SkillState,
    perk_state: &mut PerkState,
    background_state: &mut BackgroundState,
    equipment_state: &mut EquipmentState,
    screen: &mut AppScreen,
    cfg: &AppConfig,
) -> f32 {
    let Some((w, h, _token)) = render_window(ui, window, "##origin_select", "Origin Select", screen, state, special_state, skill_state, perk_state, background_state, equipment_state, character, cfg)
        else { return 0.0 };
    ui.text("ORIGIN");
    ui.separator();
    ui.spacing();

    let label_w = 140.0 * cfg.ui_scale;
    let field_w = w - label_w - 32.0 * cfg.ui_scale;

    ui.text("Character Name");
    ui.same_line_with_pos(label_w);
    ui.set_next_item_width(field_w);
    ui.input_text("##char_name", &mut character.name).build();

    ui.spacing();

    //dec/increment buttons for level
    ui.text("Character Level");
    ui.same_line_with_pos(label_w);
    if ui.button("-##level_dec") {
        if character.level > 1 {
            character.level -= 1;
            state.update_trait(character);
        }
    }
    ui.same_line();
    ui.text(format!("{}", character.level));
    ui.same_line();
    if ui.button("+##level_inc") {
        character.level += 1;
        state.update_trait(character);
    }
    //safety net, won't let character level go below 1
    if character.level < 1 {
        character.level = 1;
        state.update_trait(character);
    }

    ui.spacing();
    ui.separator();
    ui.spacing();

    ui.text("Origin");
    ui.same_line_with_pos(label_w);
    ui.set_next_item_width(field_w);

    let current_index = state.origin_label_to_index
        .iter()
        .position(|m| *m == Some(state.origin_index))
        .unwrap_or(usize::MAX);

    let current_label = state.origin_labels
        .get(current_index)
        .map(|s| s.trim())
        .unwrap_or("Select an Origin")
        .to_string();

    //origin
    let mut origin_changed = false;
    if let Some(_cb) = ui.begin_combo("##origin", &current_label) {
        for (combo_idx, label) in state.origin_labels.iter().enumerate() {
            match state.origin_label_to_index[combo_idx] {
                None => {
                    //if the label isn't an origin, print disabled (sourcebooks)
                    ui.text_disabled(label);
                }
                Some(origin_index) => {
                    //check if the index changed
                    let selected = origin_index == state.origin_index;
                    if ui.selectable_config(label.trim()).selected(selected).build() {
                        if origin_index != state.origin_index {
                            state.origin_index = origin_index;
                            origin_changed = true;
                        }
                    }
                    if selected {
                        ui.set_item_default_focus();
                    }
                }
            }
        }
    }

    //when the player selects an origin, update the origin and reload the traits
    if origin_changed {
        state.update_origin(character, background_state);
        state.reload_traits(db, character);
        skill_state.reset(character);
    }

    ui.spacing();

    //render the origin description if an origin is selected
    if let Some(origin) = &character.origin {
        ui.text("Description");
        ui.same_line_with_pos(label_w);
        render_text_wrapped(false, true, ui, &origin.desc.clone(), label_w, label_w + field_w);

        ui.spacing();

        //setting up to check if the ghoul checkbox changes
        let mut ghoul_changed = false;
        if character.origin.as_ref().unwrap().can_ghoul {
            ui.text("Ghoul?");
            ui.same_line_with_pos(label_w);
            let mut ghoul = character.ghoul;
            //if the checkbox doesn't match the character, set ghoul_changed
            if ui.checkbox("##is_ghoul", &mut ghoul) {
                if ghoul != character.ghoul {
                    ghoul_changed = true;
                }
                //set the character to whatever the checkbox says
                character.ghoul = ghoul;
            }
            ui.spacing();
        }
        
        //traits
        ui.separator();
        ui.spacing();
        ui.text("Trait");

        if ghoul_changed {
            state.reload_traits(db, character);
            skill_state.reset(character);
        }

        //check if we have any traits
        if state.origin_trait_count == 0 {
            ui.same_line_with_pos(label_w);
            ui.text_disabled("(no traits found)");
        } else if state.origin_trait_count == 1 {
            //just set the only trait available
            ui.same_line_with_pos(label_w);
            ui.text(&character.traits[0].name);
            ui.new_line();
            let y = ui.cursor_pos()[1];
            ui.set_cursor_pos([label_w, y]);
            render_text_wrapped(false, true, ui, &character.traits[0].desc, label_w, label_w + field_w);
            ui.spacing();
            state.update_trait(character);
        } else {
            //list all the traits with checkboxes, maximum of two
            let selected_count = character.traits.len();
            //let y = ui.cursor_pos()[1];
            //ui.set_cursor_pos([label_w, y]);
            ui.same_line_with_pos(label_w);
            ui.text_disabled("Choose up to 2:");
            ui.spacing();

            for (ti, t) in state.traits.iter().enumerate() {
                let mut checked = character.has_trait(t.id);
                let at_limit = !checked && selected_count >= 2;
                let y = ui.cursor_pos()[1];
                ui.set_cursor_pos([label_w, y]);

                if at_limit {
                    let _lim_guard = at_limit.then(|| ui.begin_disabled(true));
                    ui.checkbox(&format!("##trait_{}", ti), &mut checked);
                } else {
                    let mut checked = character.has_trait(t.id);
                    if ui.checkbox(&format!("##trait_{}", ti), &mut checked) {
                        //this may not work properly, it's behaving really weird with the .iter().any() vs the old way
                        let test = &mut character.has_trait(t.id);
                        if checked != *test {
                            if checked {
                                let ct = Trait {
                                    id: t.id,
                                    name: t.name.clone(),
                                    desc: t.description.clone(),
                                };
                                character.traits.push(ct);
                            } else {
                                for i in 0..character.traits.len() {
                                    if character.traits[i].id == t.id {
                                        character.traits.remove(i);
                                        break
                                    }
                                };
                            }
                        }
                        state.trait_count = character.traits.len() as i32;
                        state.update_trait(character);
                    }
                }
                ui.same_line_with_pos(label_w + 24.0 * cfg.ui_scale);
                if at_limit {
                    ui.text_disabled(&t.name);
                } else {
                    ui.text(&t.name);
                }
                let y = ui.cursor_pos()[1];
                ui.set_cursor_pos([label_w + 24.0 * cfg.ui_scale, y]);
                render_text_wrapped(at_limit, !at_limit, ui, &t.description, label_w + 24.0 * cfg.ui_scale, label_w + field_w);

                ui.spacing();
            }
        }
    }

    return h
}