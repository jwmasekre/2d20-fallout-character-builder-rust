use imgui::Ui;
use sdl2::video::Window;
use crate::AppScreen;
use crate::screens::new_character::render_text_wrapped;

pub fn render_main_menu(
    ui: &Ui,
    window: &Window,
    screen: &mut AppScreen,
    selected: &mut i32,
    items: &[&str],
) {
    let (win_w, win_h) = window.size();
    let menu_w = 340.0_f32;
    let menu_h = 320.0_f32;

    ui.window("##main_menu")
        .title_bar(false)
        .resizable(false)
        .movable(false)
        .collapsible(false)
        .size([menu_w, menu_h], imgui::Condition::Always)
        .position(
            [(win_w as f32 - menu_w) * 0.5, (win_h as f32 - menu_h) * 0.5],
            imgui::Condition::Always,
        )
        .build(|| {
            // Title
            let title = "FALLOUT 2d20 COMPANION";
            let title_w = ui.calc_text_size(title)[0];
            ui.set_cursor_pos([(menu_w - title_w) * 0.5, 24.0]);
            ui.text(title);

            ui.separator();
            ui.spacing();
            ui.spacing();

            // Keyboard nav: arrow keys move selection
            if ui.is_window_focused() {
                if ui.is_key_pressed(imgui::Key::DownArrow) {
                    *selected = (*selected + 1).min(items.len() as i32 - 1);
                }
                if ui.is_key_pressed(imgui::Key::UpArrow) {
                    *selected = (*selected - 1).max(0);
                }
                if ui.is_key_pressed(imgui::Key::Enter) || ui.is_key_pressed(imgui::Key::Space) {
                    handle_selection(*selected, screen);
                }
            }

            // Render each item as a selectable
            for (i, &label) in items.iter().enumerate() {
                let is_selected = *selected == i as i32;
                let display = if is_selected {
                    format!("  > {}  ", label)
                } else {
                    format!("    {}  ", label)
                };

                let item_w = menu_w - 40.0;
                let cursor_x = (menu_w - item_w) * 0.5;
                let y = ui.cursor_pos()[1];
                ui.set_cursor_pos([cursor_x, y]);

                if ui.selectable_config(&display)
                    .selected(is_selected)
                    .size([item_w, 36.0])
                    .build()
                {
                    *selected = i as i32;
                    handle_selection(i as i32, screen);
                }

                // Hover sets selection too
                if ui.is_item_hovered() {
                    *selected = i as i32;
                }

                ui.spacing();
            }

            ui.spacing();
            ui.spacing();
            ui.separator();

            // Footer hint
            let hint = "arrow keys / click to select  |  enter to confirm";
            //let hint_w = ui.calc_text_size(hint)[0];
            //ui.set_cursor_pos([(menu_w - hint_w) * 0.5, menu_h - 28.0]);
            render_text_wrapped(true, false, ui, hint, 16.0, menu_w - 16.0);
            //ui.text_disabled(hint);
        });
}

fn handle_selection(selected: i32, screen: &mut AppScreen) {
    match selected {
        0 => *screen = AppScreen::NewCharacter,
        1 => *screen = AppScreen::LoadCharacter,
        2 => *screen = AppScreen::ImportCharacter,
        3 => std::process::exit(0),
        _ => {}
    }
}