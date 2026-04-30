use imgui::Ui;
use sdl2::video::Window;
use crate::{character::Character, db::Db, screens::background_select::{BackgroundState, EquipmentState}, theme::render_window};

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
    background_state: &BackgroundState,
    equipment_state: &EquipmentState,
    _db: &Db,
    _character: &Character,
) -> f32 {
    let Some((_w, h, _token)) = render_window(ui, window, "##character_review", "Character Review")
        else { return 0.0 };

    ui.text("REVIEW");
    ui.separator();
    ui.spacing();

    if !state.loaded {
        //trigger all the clothing 
    }

    h
}