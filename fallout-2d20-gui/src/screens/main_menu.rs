use imgui::Ui;
use sdl2::video::Window;
use crate::AppScreen;
use crate::NewCharacterSetupState;
use crate::screens::load_character::LoadCharacterState;
use crate::screens::import_character::ImportState;

pub fn render_main_menu(
    ui: &Ui,
    window: &Window,
    screen: &mut AppScreen,
    selected: &mut i32,
    items: &[&str],
    nc_setup: &mut NewCharacterSetupState,
    load_state: &mut LoadCharacterState,
    import_state: &mut ImportState,
) {
    //get window dimmensions
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
            //title
            let title = "fallout 2d20 companion";
            let title_w = ui.calc_text_size(title)[0];
            //center the text on the top of the menu
            ui.set_cursor_pos([(menu_w - title_w) * 0.5, 24.0]);
            ui.text(title);

            ui.separator();
            ui.spacing();
            ui.spacing();

            //navigate the menu with the arrows and enter/space
            if ui.is_window_focused() {
                if ui.is_key_pressed_no_repeat(imgui::Key::DownArrow) {
                    *selected = (*selected + 1).min(items.len() as i32 - 1);
                }
                if ui.is_key_pressed_no_repeat(imgui::Key::UpArrow) {
                    *selected = (*selected - 1).max(0);
                }
                if ui.is_key_pressed_no_repeat(imgui::Key::Enter) || ui.is_key_pressed_no_repeat(imgui::Key::Space) {
                    handle_selection(*selected, screen, nc_setup, load_state, import_state);
                }
            }

            for (i, &label) in items.iter().enumerate() {
                let is_selected = *selected == i as i32;
                //add an arrow to the label of the active button
                let display = if is_selected {
                    format!("  > {}  ", label)
                } else {
                    format!("    {}  ", label)
                };

                //align the buttons centered
                let item_w = menu_w - 40.0;
                let cursor_x = (menu_w - item_w) * 0.5;
                let y = ui.cursor_pos()[1];
                ui.set_cursor_pos([cursor_x, y]);

                if ui.selectable_config(&display)
                    .selected(is_selected)
                    .size([item_w, 36.0])
                    .build() {
                    *selected = i as i32;
                    handle_selection(i as i32, screen, nc_setup, load_state, import_state);
                }

                //also select the button on hover (so that space/enter/click navigates)
                if ui.is_item_hovered() {
                    *selected = i as i32;
                }
                ui.spacing();
            }

            ui.spacing();
            ui.spacing();
            ui.separator();

            let hint = "arrow/hover select | enter/space confirm";
            let hint_w = ui.calc_text_size(hint)[0];
            ui.set_cursor_pos([(menu_w - hint_w) * 0.5, menu_h - 28.0]);
            ui.text_disabled(hint);
        });
}

fn handle_selection(selected: i32, screen: &mut AppScreen, nc_setup: &mut NewCharacterSetupState, load_state: &mut LoadCharacterState, import_state: &mut ImportState) {
    match selected {
        0 => {
            nc_setup.reset();
            *screen = AppScreen::NewCharSetup
        },
        1 => {
            load_state.reset();
            *screen = AppScreen::LoadCharacter
        },
        2 => {
            import_state.reset();
            *screen = AppScreen::ImportCharacter
        },
        3 => *screen = AppScreen::Settings,
        4 => std::process::exit(0),
        _ => {}
    }
}