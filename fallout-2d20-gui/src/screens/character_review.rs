use imgui::Ui;
use sdl2::video::Window;
use crate::{character::Character, db::Db, theme::render_window};



pub fn render_character_review(
    ui: &Ui,
    window: &Window,
    db: &Db,
    character: &Character,
) -> f32 {
    let Some((w, h, _token)) = render_window(ui, window, "##character_review", "Character Review")
        else { return 0.0 };

    ui.text("REVIEW");
    ui.separator();
    ui.spacing();

    h


}