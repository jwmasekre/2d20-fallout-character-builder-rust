
use imgui::Ui;
use sdl2::video::Window;
use crate::{AppScreen, character::Character, db::Db, theme::render_window};



pub fn render_character_sheet(
    ui: &Ui,
    window: &Window,
    db: &Db,
    character: &Character,
    screen: &mut AppScreen,
) -> f32 {
    let Some((w, h, _token)) = render_window(ui, window, "##character_sheet", "Character Sheet", screen)
        else { return 0.0 };

    ui.text(format!("{} --- {}", character.name, character.player.name));
    ui.separator();
    ui.spacing();

    h
}
