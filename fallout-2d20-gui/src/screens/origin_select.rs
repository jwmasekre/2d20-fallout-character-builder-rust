use imgui::Ui;
use sdl2::video::Window;
use crate::db::Db;
use crate::character::{Character, MutantType, RobotType, Origin, Trait, Special};
use crate::theme::{render_text_wrapped, render_window};
use crate::log_on_change;

#[derive(Debug)]
pub struct OriginState {
    selected: bool,
    trait_count: i32,
    origin_trait_count: i32,
    origin_index: usize,
    origin_labels: Vec<String>,
    origin_label_to_index: Vec<Option<usize>>,
    origins: Vec<OriginRow>,
    traits: Vec<TraitRow>,
    ghoul_trait: Option<TraitRow>,
}
impl OriginState {
    pub fn new(db: &Db) -> Self {
        let origins = load_origins(db);
        let (labels, label_map) = build_origin_labels(&origins);
        Self {
            selected: false,
            trait_count: 0,
            origin_trait_count: 0,
            origin_index: 0,
            origin_labels: labels,
            origin_label_to_index: label_map,
            origins,
            traits: vec![],
            ghoul_trait: None,
        }
    }
    pub fn is_complete(&self) -> bool {//do we need to be checking the character to make sure that's good to go too?
        self.selected && (self.trait_count == self.origin_trait_count)
    }
    fn update_origin(&mut self, character: &mut Character) {
        //if no origins are returned by the db, return None
        if self.origins.is_empty() { character.origin = None; return }
        //mark that the player has selected an origin
        self.selected = true;
        //capture which origin they selected
        let selected_origin = self.origins.get(self.origin_index).unwrap();
        //build the origin struct for assignment to character
        let active_origin = Origin {
            id: selected_origin.id,
            name: selected_origin.name.clone(),
            desc: selected_origin.description.clone(),
            can_ghoul: selected_origin.can_ghoul,
        };
        //grab the old origin; checking if they go from mutant -> non or vice-versa
        let old_origin = if character.origin.is_some() {character.origin.clone().unwrap().id} else {i32::MAX};
        //if they go to mutant from non, add two to str and end
        if [3,16].contains(&selected_origin.id) && ![3,16].contains(&old_origin) {
            character.special.strength.value += 2;
            character.special.endurance.value += 2;
        //if they go from mutant to non, remove two from str and end
        } else if ![3,16].contains(&selected_origin.id) && [3,16].contains(&old_origin) {
            character.special.strength.value -= 2;
            character.special.endurance.value -= 2;
        }
        //update the character origin
        character.origin = Some(active_origin);
        //set mutant status
        character.mutant = match selected_origin.id {
            3 => MutantType::SuperMutant,
            16 => MutantType::Nightkin,
            _ => MutantType::None,
        };
        //set robot status
        character.robot = match selected_origin.id {
            4 => RobotType::Handy,
            9 => RobotType::Protectron,
            10 => RobotType::Robobrain,
            11 => RobotType::Securitron,
            12 => RobotType::Synth,
            14 => RobotType::Assaultron,
            _ => RobotType::None,
        };
        //update max special
        Special::apply_max(character);
    }
    //retrieve the traits based on the current origin
    fn reload_traits(&mut self, db: &Db, character: &mut Character) {
        //clear traits
        self.traits = vec![];
        character.traits = vec![];
        //return if the db doesn't return any origins
        if self.origins.is_empty() {
            return;
        }
        //grab the currently selected origin's id
        let origin_id = self.origins[self.origin_index].id;
        //retrieve traits from the db
        let traits = load_traits(db, origin_id, self);
        //retrieve the ghoul trait
        let ghoul_trait = &load_ghoul_traits(db, self)[0];
        //if the player selected the ghoul origin, mark the character as a ghoul
        if traits[0].is_ghoul_trait {
            character.ghoul = true;
        //if the player selected an origin that can't ghoul, mark them as not a ghoul
        } else if !self.origins[self.origin_index].can_ghoul {
            character.ghoul = false;
        }
        //if the character is a ghoul, set their trait to the ghoul trait and don't run any other trait logic
        if character.ghoul {
            //building the trait struct for assignment
            let active_trait= Trait {
                id: ghoul_trait.id,
                name: ghoul_trait.name.clone(),
                desc: ghoul_trait.description.clone(),
            };
            //assign the trait
            character.traits = vec![active_trait];
            //mark the number of valid traits as 1 and assigned as 1 (satisfies one of the complete conditions)
            self.trait_count = 1;
            self.origin_trait_count = 1;
        //if the character's origin only has one trait, just grab that and assign it (like the ghoul block above)
        } else if traits.len() == 1 {
            let active_trait = Trait {
                id: traits[0].id,
                name: traits[0].name.clone(),
                desc: traits[0].description.clone(),
            };
            character.traits = vec![active_trait];
            self.trait_count = 1;
            self.origin_trait_count = 1;
        //the character's origin has options, so clear out the character's traits and indicate that they've selected 0 out of a max of 2
        } else {
            character.traits = vec![];
            self.trait_count = 0;
            self.origin_trait_count = 2;
            self.traits = traits;
        }
    }
    fn update_trait(&self, character: &mut Character) {
        let skill_max = character.level.clamp(3,6);
        if character.is_mutant() {
            character.skills.athletics.max = 4.min(skill_max);
            character.skills.barter.max = 4.min(skill_max);
            character.skills.big_guns.max = 4.min(skill_max);
            character.skills.energy_weapons.max = 4.min(skill_max);
            character.skills.explosives.max = 4.min(skill_max);
            character.skills.lockpick.max = 4.min(skill_max);
            character.skills.medicine.max = 4.min(skill_max);
            character.skills.melee_weapons.max = 4.min(skill_max);
            character.skills.pilot.max = 4.min(skill_max);
            character.skills.repair.max = 4.min(skill_max);
            character.skills.science.max = 4.min(skill_max);
            character.skills.small_guns.max = 4.min(skill_max);
            character.skills.sneak.max = 4.min(skill_max);
            character.skills.speech.max = 4.min(skill_max);
            character.skills.survival.max = 4.min(skill_max);
            character.skills.throwing.max = 4.min(skill_max);
            character.skills.unarmed.max = 4.min(skill_max);
        } else {
            character.skills.athletics.max = skill_max;
            character.skills.barter.max = skill_max;
            character.skills.big_guns.max = skill_max;
            character.skills.energy_weapons.max = skill_max;
            character.skills.explosives.max = skill_max;
            character.skills.lockpick.max = skill_max;
            character.skills.medicine.max = skill_max;
            character.skills.melee_weapons.max = skill_max;
            character.skills.pilot.max = skill_max;
            character.skills.repair.max = skill_max;
            character.skills.science.max = skill_max;
            character.skills.small_guns.max = skill_max;
            character.skills.sneak.max = skill_max;
            character.skills.speech.max = skill_max;
            character.skills.survival.max = skill_max;
            character.skills.throwing.max = skill_max;
            character.skills.unarmed.max = skill_max;
        }
        if character.has_trait(13) {
            character.skills.athletics.max = 4.min(skill_max);
            character.skills.big_guns.max = 4.min(skill_max);
            character.skills.energy_weapons.max = 4.min(skill_max);
            character.skills.explosives.max = 4.min(skill_max);
            character.skills.lockpick.max = 4.min(skill_max);
            character.skills.melee_weapons.max = 4.min(skill_max);
            character.skills.pilot.max = 4.min(skill_max);
            character.skills.small_guns.max = 4.min(skill_max);
            character.skills.sneak.max = 4.min(skill_max);
            character.skills.survival.max = 4.min(skill_max);
            character.skills.throwing.max = 4.min(skill_max);
            character.skills.unarmed.max = 4.min(skill_max);
        }
    }
}

#[derive(Debug, Clone)]
pub struct OriginRow {
    pub id: i32,
    pub name: String,
    pub sourcebook: String,
    pub description: String,
    pub can_ghoul: bool,
}

#[derive(Debug, Clone)]
pub struct TraitRow {
    pub id: i32,
    pub origin_id: i32,
    pub name: String,
    pub description: String,
    pub is_ghoul_trait: bool,
}

fn load_origins(db: &Db) -> Vec<OriginRow> {
    let result = db.block_on(async {
        sqlx::query!(
            r#"
            SELECT o.id, o.name, o.description, o.can_ghoul,
                s.name AS sourcebook
            FROM origins o
            JOIN sourcebooks s ON s.id = o.sourcebook_id
            ORDER BY s.id, o.name
            "#
        ).fetch_all(&db.pool).await
    });

    match result {
        Ok(rows) => rows.into_iter().map(|r| OriginRow {
            id: r.id as i32,
            name: r.name.unwrap_or_default(),
            sourcebook: r.sourcebook.unwrap_or_default(),
            description: r.description.unwrap_or_default(),
            can_ghoul: r.can_ghoul.unwrap_or(0) != 0,
        }).collect(),
        Err(e) => { eprintln!("Failed to load origins: {e}"); vec![] }
    }
}

fn build_origin_labels(origins: &[OriginRow]) -> (Vec<String>, Vec<Option<usize>>) {
    let mut labels: Vec<String> = vec![];
    let mut label_map: Vec<Option<usize>> = vec![];
    let mut current_book = String::new();

    for (i, origin) in origins.iter().enumerate() {
        if origin.sourcebook != current_book {
            current_book = origin.sourcebook.clone();
            labels.push(format!("-- {} --", current_book));
            label_map.push(None); // header — not selectable
        }
        labels.push(format!("  {}", origin.name));
        label_map.push(Some(i));
    }

    (labels, label_map)
}

fn load_traits(db: &Db, origin_id: i32, state: &mut OriginState) -> Vec<TraitRow> {
    //let origin_id = origin.id as i64;
    let result =
        db.block_on(async {
            sqlx::query!(
                r#"
                SELECT t.id, ot.origin_id, t.name, t.description,
                    ot.is_ghoul_trait
                FROM origin_traits ot
                JOIN traits t ON t.id = ot.trait_id
                WHERE ot.origin_id = ?
                ORDER BY ot.is_ghoul_trait, t.name
                "#,
                origin_id
            ).fetch_all(&db.pool).await
        });
    
    state.origin_trait_count = result.iter().count().min(2) as i32;
    match result {
        Ok(rows) => rows.into_iter().map(|r| TraitRow {
            id: r.id as i32,
            origin_id: r.origin_id.unwrap_or_default() as i32,
            name: r.name.unwrap_or_default(),
            description: r.description.unwrap_or_default(),
            is_ghoul_trait: r.is_ghoul_trait.unwrap_or(0) != 0,
        }).collect(),
        Err(e) => { eprintln!("Failed to load traits: {e}"); vec![] }
    }
}

fn load_ghoul_traits(db: &Db, state: &mut OriginState) -> Vec<TraitRow> {
    let result =
        db.block_on(async {
            sqlx::query!(
                r#"
                SELECT t.id, ot.origin_id, t.name, t.description,
                    ot.is_ghoul_trait
                FROM origin_traits ot
                JOIN traits t ON t.id = ot.trait_id
                WHERE ot.origin_id = 2
                ORDER BY ot.is_ghoul_trait, t.name
                "#,
            ).fetch_all(&db.pool).await
        });
    
    state.origin_trait_count = 1;
    match result {
        Ok(rows) => rows.into_iter().map(|r| TraitRow {
            id: r.id as i32,
            origin_id: r.origin_id.unwrap_or_default() as i32,
            name: r.name.unwrap_or_default(),
            description: r.description.unwrap_or_default(),
            is_ghoul_trait: r.is_ghoul_trait.unwrap_or(0) != 0,
        }).collect(),
        Err(e) => { eprintln!("Failed to load traits: {e}"); vec![] }
    }
}

pub fn render_origin_select(
    ui: &Ui,
    window: &Window,
    state: &mut OriginState,
    db: &Db,
    character: &mut Character,
) -> f32 {
    let Some((w, h, _token)) = render_window(ui, window, "##origin_select", "Origin Select")
        else { return 0.0 };
    ui.text("ORIGIN");
    ui.separator();
    ui.spacing();

    let label_w = 140.0_f32;
    let field_w = w - label_w - 32.0;

    ui.text("Character Name");
    ui.same_line_with_pos(label_w);
    ui.set_next_item_width(field_w);
    ui.input_text("##char_name", &mut character.name).build();

    ui.spacing();

    //dec/increment buttons for level
    ui.text("Character Level");
    ui.same_line_with_pos(label_w);
    if ui.button("-##level_dec") {
        if character.level > 1 { character.level -= 1; }
    }
    ui.same_line();
    ui.text(format!("{}", character.level));
    ui.same_line();
    if ui.button("+##level_inc") {
        character.level += 1;
    }
    //safety net, won't let character level go below 1
    if character.level < 1 { character.level = 1; }

    ui.spacing();
    ui.separator();
    ui.spacing();

    ui.text("Origin");
    ui.same_line_with_pos(label_w);
    ui.set_next_item_width(field_w);

    let current_index = state.origin_label_to_index
        .iter()
        .position(|m| *m == Some(state.origin_index))
        .unwrap_or(0);

    let current_label = state.origin_labels
        .get(current_index)
        .map(|s| s.trim())
        .unwrap_or("-")
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
        state.update_origin(character);
        state.reload_traits(db, character)
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

        if ghoul_changed { state.reload_traits(db, character); }

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

            log_on_change!(state.traits);

            for (ti, t) in state.traits.iter().enumerate() {
                //log_on_change!(ti);
                //log_on_change!(t);
                let mut checked = character.has_trait(t.id);
                let at_limit = !checked && selected_count >= 2;
                let y = ui.cursor_pos()[1];
                ui.set_cursor_pos([label_w, y]);

                if at_limit {
                    let _lim_guard = at_limit.then(|| ui.begin_disabled(true));
                    ui.checkbox(&format!("##trait_{}", ti), &mut checked);
                } else {
                    if ui.checkbox(&format!("##trait_{}", ti), &mut checked) {
                        //this may not work properly, it's behaving really weird with the .iter().any() vs the old way
                        let test = &mut character.has_trait(t.id);
                        if *test {
                            *test = checked;
                        }
                        state.update_trait(character);
                    }
                }
                ui.same_line_with_pos(label_w + 24.0);
                if at_limit {
                    ui.text_disabled(&t.name);
                } else {
                    ui.text(&t.name);
                }
                let y = ui.cursor_pos()[1];
                ui.set_cursor_pos([label_w + 24.0, y]);
                render_text_wrapped(at_limit, !at_limit, ui, &t.description, label_w + 24.0, label_w + field_w);

                ui.spacing();
            }
        }
    }

    return h
}