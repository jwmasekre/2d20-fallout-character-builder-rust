mod config;
mod screens;
mod theme;
mod crt;
#[macro_use]
mod debug;

use std::os::raw::c_void;
use glow::HasContext;
use sdl2::video::{ GLProfile,Window };
use imgui_sdl2::ImguiSdl2;
use imgui_opengl_renderer::Renderer;
use imgui::Ui;
use anyhow::Result;

use fallout_2d20_core::constants::{
    VERSION,
    DATE,
    BUILD_SCREENS
};

use fallout_2d20_core::{
    sync_derived_weapons,
    character::{
        Character,
        Party,
        Player,
    },
    db::Db,
    states::{
        BackgroundState,
        EquipmentState,
        ImportState,
        LoadCharacterState,
        NewCharacterSetupState,
        OriginState,
        PerkState,
        ReviewState,
        SheetState,
        SkillState,
        SpecialState
    },
    structs::{
        AppConfig,
        AppScreen,
        PerkResolutionPopup
    },
};
use crate::screens::background_select::render_background_select;
use crate::screens::character_review::render_character_review;
use crate::screens::character_sheet::render_character_sheet;
use crate::screens::import_character::render_import_character;
use crate::screens::load_character::render_load_character;
use crate::screens::main_menu::render_main_menu;
use crate::screens::new_char_setup::render_new_character_setup;
use crate::screens::origin_select::render_origin_select;
use crate::screens::perk_select::{render_perk_resolution, render_perk_select};
use crate::screens::settings::render_settings;
use crate::screens::skill_assignment::render_skill_assignment;
use crate::screens::special_assignment::render_special_assignment;
use crate::screens::stat_calculation::render_stat_calculation;
use crate::theme::Theme;
use crate::{
    config::{
        db_path,
        load_config,
        save_config
    },
    crt::CrtEffect,
    theme::{
        BAR_HEIGHT,
        THEMES,
        apply_theme,
        render_text_wrapped
    }
};

pub fn screen_unlocked(
    screen: &AppScreen,
    origin: &OriginState,
    special: &SpecialState,
    skill: &SkillState,
    perk: &PerkState,
    background: &mut BackgroundState,
    equipment: &mut EquipmentState,
    review: &mut ReviewState,
    db: &Db,
    character: &Character,
) -> bool {
    match screen {
        AppScreen::OriginSelect => true,
        AppScreen::SpecialAssignment => origin.is_complete(),
        AppScreen::SkillAssignment => special.is_complete(character),
        AppScreen::PerkSelect => skill.is_complete(character),
        AppScreen::StatCalculation => perk.is_complete(),
        AppScreen::BackgroundSelect => special.is_complete(character) && skill.is_complete(character) && perk.is_complete(),
        AppScreen::CharacterReview => background.is_complete(equipment, db, character, review),
        _ => false,
    }
}

//build a placeholder window
fn _render_placeholder(ui: &Ui, window: &Window, title: &str, screen: &mut AppScreen, cfg: &AppConfig) {
    let (win_w, win_h) = window.size();
    let w = 500.0 * cfg.ui_scale;
    let h = 200.0 * cfg.ui_scale;

    ui.window(&format!("##{title}_placeholder"))
        .title_bar(false)
        .resizable(false)
        .movable(false)
        .size([w, h], imgui::Condition::Always)
        .position(
            [(win_w as f32 - w) * 0.5, (win_h as f32 - h) * 0.5],
            imgui::Condition::Always,
        )
        .build(|| {
            ui.text(format!("{} -- coming soon", title));
            ui.spacing();
            ui.separator();
            ui.spacing();
            if ui.button("< Back to Main Menu") {
                *screen = AppScreen::MainMenu;
            }
        });
}

fn main() -> Result<()> {
    //build the window
    let sdl_context = sdl2::init().map_err(|e| anyhow::anyhow!(e))?;
    let video_subsystem = sdl_context.video().map_err(|e| anyhow::anyhow!(e))?;

    video_subsystem.gl_attr().set_context_profile(GLProfile::Core);
    video_subsystem.gl_attr().set_context_version(3,2);

    let window = video_subsystem
        .window(&format!("Fallout 2d20 Character Manager v{}",VERSION.as_string()), 1280, 960)
        .position_centered()
        .opengl()
        .resizable()
        .build()
        .map_err(anyhow::Error::msg)?;

    let _gl_context = window.gl_create_context().map_err(|e| anyhow::anyhow!(e))?;

    let gl = unsafe {
        glow::Context::from_loader_function(|s| {
            video_subsystem.gl_get_proc_address(s) as *const c_void
        })
    };

    //load the user config
    let mut cfg = load_config();
    //load the theme from the user config
    let mut current_theme = cfg.theme_index.min(THEMES.len() - 1);

    //creates the context for imgui functions
    let mut imgui = imgui::Context::create();
    //create the crt effect
    let (init_w, init_h) = window.size();
    let mut crt = CrtEffect::new(&gl, init_w as i32, init_h as i32);
    crt.distortion = cfg.crt_distortion;
    crt.scanline_strength = cfg.crt_scanline_strength;
    crt.vignette_multiplier = cfg.crt_vignette_multiplier;
    crt.vignette_exponent = cfg.crt_vignette_exponent;
    crt.roll_speed = cfg.crt_roll_speed;
    crt.tint_strength = cfg.crt_tint_strength;
    crt.chromatic_aberration = cfg.crt_chromatic_aberration;
    //applies the theme from the user config
    apply_theme(&mut imgui, THEMES[current_theme], &mut crt);
    //load custom font (might move to theme.rs)
    imgui.fonts().clear();
    let font_path = if cfg.font_path.is_none() {
        std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join("fonts/Monofonto.ttf")
    } else { cfg.font_path.clone().unwrap() };
    imgui.fonts().add_font(&[imgui::FontSource::TtfData {
        data: &std::fs::read(&font_path).expect("Failed to load Monofonto.ttf"),
        size_pixels: cfg.font_size,
        config: Some(imgui::FontConfig {
            oversample_h: 2,
            oversample_v: 2,
            pixel_snap_h: false,
            ..imgui::FontConfig::default()
        }),
    }]);
    imgui.fonts().tex_id;
    //sets the path for the db
    let db_path = &db_path(cfg.db_path.clone());
    //sets the file path for imgui ini file
    let ini_path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("imgui.ini");
    imgui.set_ini_filename(ini_path);

    //create the renderer
    let mut imgui_sdl2 = ImguiSdl2::new(&mut imgui, &window);
    let renderer = Renderer::new(&mut imgui, |s| {
        video_subsystem.gl_get_proc_address(s) as *const c_void
    });

    //create the event pump
    let mut event_pump = sdl_context.event_pump().map_err(|e| anyhow::anyhow!(e))?;

    //create the main menu
    let mut screen = AppScreen::MainMenu;
    let mut selected_menu_item: i32 = 0;
    let menu_items = ["New Character", "Load Character", "Import Character", "Settings", "Quit"];

    //since we just set our theme, pending theme can be set to the same thing
    //we'll check this in the loop every frame to determine if the theme needs
    //  to be updated
    let mut pending_theme: Option<usize> = Some(current_theme);

    //create the db if it doesn't exist, which avoids errors
    //if it's a new db, it's going to result in blank entries for origin select
    std::fs::create_dir_all(db_path.parent().unwrap())?;
    let db = Db::connect(&format!("sqlite:{}", db_path.display()))?;

    //initializing all the states on first load as None
    let mut show_about = false;
    //let mut player = Player::new();
    let player = Player::new();
    //let mut party = Party::new();
    let party = Party::new();
    let mut character = Character::new(player, party, VERSION);
    //let mut party: Option<Party> = None;
    let _party: Option<Party> = None;
    let mut origin = OriginState::new(&db);
    let mut special = SpecialState::new();
    let mut skill = SkillState::new(character.clone());
    let mut perk = PerkState::new(&db, &character);
    let mut perk_resolution: Option<PerkResolutionPopup> = None;
    let mut background = BackgroundState::new(&db);
    let mut equipment = EquipmentState::new();
    let mut review = ReviewState::new();
    let mut nc_setup = NewCharacterSetupState::new();
    let mut pending_player_name: Option<String> = None;
    let mut pending_party_name: Option<String> = None;
    let mut load_character_state = LoadCharacterState::new();
    let mut import_state = ImportState::new();
    let mut sheet_state = SheetState::new();

    //we hide the os cursor so that we can apply the crt shader to a custom one
    sdl_context.mouse().show_cursor(false);

    //start the render loop
    'main: loop {
        //function for handling the tabbed windows for the builder:
        pub fn render_tab_bar(
            ui: &Ui,
            screen: &mut AppScreen,
            origin: &OriginState,
            special: &SpecialState,
            skill: &SkillState,
            perk: &PerkState,
            background: &mut BackgroundState,
            equipment: &mut EquipmentState,
            review: &mut ReviewState,
            db: &Db,
            character: &Character,
        ) {
            //evenly space the tabs
            let tab_w = ui.content_region_avail()[0] / BUILD_SCREENS.len() as f32;

            //establish which tab is the current tab and which are unlocked
            let current = screen.clone();
            for (target, label) in BUILD_SCREENS {
                let is_current  = target == &current;
                let is_unlocked = screen_unlocked(target, origin, special, skill, perk, background, equipment, review, &db, &character);

                //highlight the current tab
                if is_current {
                    let color = ui.push_style_color(
                        imgui::StyleColor::Button,
                        ui.style_color(imgui::StyleColor::ButtonActive),
                    );
                    ui.set_next_item_width(tab_w);
                    ui.button(label);
                    drop(color);
                    //render the tab normally
                } else if is_unlocked {
                    ui.set_next_item_width(tab_w);
                    if ui.button(label) {
                        *screen = target.clone();
                    }
                } else {
                    //disable the tab
                    let color = ui.push_style_color(
                        imgui::StyleColor::Text,
                        ui.style_color(imgui::StyleColor::TextDisabled),
                    );
                    let color2 = ui.push_style_color(
                        imgui::StyleColor::Button,
                        ui.style_color(imgui::StyleColor::FrameBg),
                    );
                    ui.set_next_item_width(tab_w);
                    ui.button(label);
                    drop(color);
                    drop(color2);
                }

                ui.same_line_with_spacing(0.0, 0.0);
            }

            ui.new_line();
            ui.separator();
            ui.spacing();
        }

        //function for rendering forward/back buttons
        pub fn render_nav_footer(
            ui: &Ui,
            w: f32,
            h: f32,
            screen: &mut AppScreen,
            origin: &OriginState,
            special: &SpecialState,
            skill: &SkillState,
            perk: &PerkState,
            background: &mut BackgroundState,
            equipment: &mut EquipmentState,
            review: &mut ReviewState,
            db: &Db,
            character: &mut Character,
            sheet: &mut SheetState,
            cfg: &AppConfig,
        ) {
            let current = screen.clone();
            //figure out which tab/screen we're on
            let idx = BUILD_SCREENS.iter().position(|(s, _)| s == &current).unwrap_or(0);

            //define the previous and next screens
            let prev = idx.checked_sub(1).map(|i| &BUILD_SCREENS[i].0);
            let next = BUILD_SCREENS.get(idx + 1).map(|(s, _)| s);

            ui.separator();
            ui.spacing();

            //if there's a previous screen, create a back button and point to it
            if let Some(prev_screen) = prev {
                ui.set_cursor_pos([16.0, h - 36.0 * cfg.ui_scale]);
                if ui.button("< Back") {
                    *screen = prev_screen.clone();
                }
            }

            if let Some(next_screen) = next {
                let next_w = ui.calc_text_size("Next >")[0];
                ui.set_cursor_pos([w - 24.0 - next_w, h - 36.0 * cfg.ui_scale]);
                let unlocked = screen_unlocked(next_screen, origin, special, skill, perk, background, equipment, review, db, &character);
                //disable the next button if it's not unlocked
                if !unlocked {
                    let c = ui.push_style_color(imgui::StyleColor::Text, ui.style_color(imgui::StyleColor::TextDisabled));
                    let c2 = ui.push_style_color(imgui::StyleColor::Button, ui.style_color(imgui::StyleColor::FrameBg));
                    ui.button("Next >");
                    drop(c); drop(c2);
                } else if ui.button("Next >") {
                    *screen = next_screen.clone();
                }
            } else if current == AppScreen::CharacterReview {
                let final_w = ui.calc_text_size("Finalize Character >")[0];
                ui.set_cursor_pos([w - 24.0 - final_w, h - 36.0 * cfg.ui_scale]);
                if ui.button("Finalize Character >") {
                    character.weapons = equipment.weapons.clone();
                    character.ammo = equipment.ammo.clone();
                    character.apparel = equipment.apparel.clone();
                    character.robot_modules = equipment.robot_modules.clone();
                    character.consumables = equipment.consumables.clone();
                    character.gear = equipment.gear.clone();
                    character.junk = equipment.junk.clone();
                    character.misc = equipment.misc.clone();
                    sheet.new_character(character);
                    sync_derived_weapons(character, db);
                    let mut success = false;
                    match db.save_character(character) {
                        Ok(_) => {success = true;},
                        Err(e) => eprintln!("Failed to save character: {e}"),
                    }
                    if success {
                        match db.load_character(&character.id.to_string()) {
                            Ok(loaded) => {
                                *character = loaded;
                                sheet.new_character(character);
                                *screen = AppScreen::CharacterSheet;
                            }
                            Err(e) => {
                                eprintln!("Failed to load: {e}");
                            }
                        }
                    }
                }
            }
        }

        //listen for events and handle them
        for event in event_pump.poll_iter() {
            imgui_sdl2.handle_event(&mut imgui, &event);
            match event {
                sdl2::event::Event::Quit { .. } => break 'main,
                sdl2::event::Event::Window { win_event: sdl2::event::WindowEvent::Resized(w, h), .. } => crt.resize(&gl, w, h),
                _ => {},
            }
        }
        
        //if pending theme is not None, apply it and make it None
        if let Some(t) = pending_theme.take() {
            apply_theme(&mut imgui, THEMES[t], &mut crt);
        }

        //we tell imgui not to manipulate the cursor. without this it would just redraw the os cursor
        imgui.io_mut()
            .config_flags
            .insert(imgui::ConfigFlags::NO_MOUSE_CURSOR_CHANGE);
        //create the frame for rendering stuff
        imgui_sdl2.prepare_frame(imgui.io_mut(), &window, &event_pump.mouse_state());
        let ui = imgui.frame();
        //get the window size
        let (win_w, _win_h) = window.size();

        //create the theme bar at the very top
        ui.window("##theme_bar")
            .title_bar(false)
            .resizable(false)
            .movable(false)
            .collapsible(false)
            .no_decoration()
            .size([win_w as f32, BAR_HEIGHT * cfg.ui_scale], imgui::Condition::Always)
            .position([0.0, 0.0], imgui::Condition::Always)
            .build(|| {
                ui.set_cursor_pos([8.0, 7.0]);
                ui.text_colored(THEMES[current_theme].text_dim, "Theme:");
                ui.same_line();
                //iterating through the themes to draw radio buttons
                for (i, theme) in THEMES.iter().enumerate() {
                    //checks if the radio button is clicked for the theme
                    if ui.radio_button_bool(theme.name, current_theme == i) {
                        //sets the theme, then writes it to the config
                        current_theme = i;
                        pending_theme = Some(i);
                        let ui_scale = {cfg.set_ui_scale();cfg.ui_scale};
                        
                        save_config(&AppConfig {
                            theme_index: i,
                            db_path: db_path.to_path_buf(),
                            font_path: Some(font_path.clone()),
                            font_size: cfg.font_size,
                            ui_scale,
                            crt_distortion: cfg.crt_distortion,
                            crt_scanline_strength: cfg.crt_scanline_strength,
                            crt_vignette_multiplier: cfg.crt_vignette_multiplier,
                            crt_vignette_exponent: cfg.crt_vignette_exponent,
                            crt_roll_speed: cfg.crt_roll_speed,
                            crt_tint_strength: cfg.crt_tint_strength,
                            crt_chromatic_aberration: cfg.crt_chromatic_aberration,
                        });
                    }
                    if i < THEMES.len() - 1 {
                        //doesn't move to the next line unless we're at the end of the themes
                        ui.same_line();
                    }
                }
                ui.same_line();
                ui.text_disabled("|");
                ui.same_line();
                let robco = if win_w as f32 / cfg.ui_scale >= 1040.0 {"ROBCO Industries (TM) Termlink##crt_toggle"} else if win_w as f32 / cfg.ui_scale >= 960.0 {"ROBCO Industries(TM)##crt_toggle"} else {"ROBCO##crt_toggle"};
                ui.checkbox(robco, &mut crt.enabled);
                // About button, right-aligned
                let button_w = 60.0 * cfg.ui_scale;
                let button_x = win_w as f32 - button_w - 8.0;
                ui.set_cursor_pos([button_x, 4.0 * cfg.ui_scale]);
                //set the show_about flag if clicked
                if ui.button("About") {
                    show_about = true; 
                }
            });

        //render about window if the flag is set
        if show_about {
            let (win_w, win_h) = window.size();
            let aw = 400.0 * cfg.ui_scale;
            let ah = 220.0 * cfg.ui_scale;
            let center = [(win_w as f32 - aw) * 0.5, (win_h as f32 - ah) * 0.5];

            //center the about window when it's opened or the button is clicked again (not every frame)
            let _condition = if ui.is_mouse_released(imgui::MouseButton::Left) {
                imgui::Condition::Appearing
            } else {
                imgui::Condition::Appearing
            };

            //rendering the about window
            ui.window("##about")
                .title_bar(false)
                .resizable(false)
                .movable(true)
                .collapsible(false)
                .size([aw, ah], imgui::Condition::Always)
                .position(center, imgui::Condition::Once)
                .bring_to_front_on_focus(true)
                .build(|| {
                    //title with X
                    let close_x = aw - 28.0 * cfg.ui_scale;
                    ui.text("About");
                    ui.same_line_with_pos(close_x);
                    if ui.button("X##about_close") {
                        show_about = false;
                    }
                    ui.separator();
                    ui.spacing();

                    ui.text("fallout 2d20 character manager");
                    ui.spacing();
                    render_text_wrapped(true, false, ui, &format!("v{}, {}", VERSION.as_string(), DATE), 16.0 * cfg.ui_scale, aw - 32.0 * cfg.ui_scale);
                    ui.spacing();
                    ui.text_wrapped("a character creation and management tool for the 2d20 ttrpg system.");
                    ui.text_colored([0.90, 0.10, 0.50, 1.00], "by josh");
                    ui.spacing();
                    ui.separator();
                    ui.spacing();
                    render_text_wrapped(true, false, ui, "built with rust//imgui//sdl2", 16.0 * cfg.ui_scale, aw - 32.0 * cfg.ui_scale);
                });
        }

        let is_builder_screen = BUILD_SCREENS.iter().any(|(s, _)| s == &screen);
        if is_builder_screen {
            //tabs across the top
            let tab_bar_h: f32 = 44.0 * cfg.ui_scale;
            ui.window("##tab_bar")
                .title_bar(false)
                .resizable(false)
                .movable(false)
                .collapsible(false)
                .no_decoration()
                .size([win_w as f32, tab_bar_h], imgui::Condition::Always)
                .position([0.0, BAR_HEIGHT], imgui::Condition::Always)
                .build(|| {
                    render_tab_bar(ui, &mut screen, &origin, &special, &skill, &perk, &mut background, &mut equipment, &mut review, &db, &character);
                });
        }

        let _content_h: f32 = match screen {
/*--------*/AppScreen::MainMenu => {
                render_main_menu(&ui, &window, &mut screen, &mut selected_menu_item, &menu_items, &mut nc_setup, &mut load_character_state, &mut import_state, &cfg);
                0.0
            }
            AppScreen::NewCharSetup => {
                render_new_character_setup(&ui, &window,&mut nc_setup, &mut screen, &db, &mut pending_player_name, &mut pending_party_name, &mut character, &cfg
                );
                0.0
            }
/*--------*/AppScreen::OriginSelect => {
                let state = &mut origin;
                render_origin_select(&ui, &window, state, &db, &mut character, &mut special, &mut skill, &mut perk, &mut background, &mut equipment, &mut screen, &cfg)
            }
/*--------*/AppScreen::SpecialAssignment => {
                //let state = &mut special.update(&character);
                let state = &mut special;
                render_special_assignment(&ui, &window, state, &db, &mut character, &mut screen, &mut origin, &mut skill, &mut perk, &mut background, &mut equipment, &cfg)
            }
/*--------*/AppScreen::SkillAssignment => {
                skill.update(&character);
                let state = &mut skill;
                render_skill_assignment(&ui, &window, state, &db, &mut character, &mut screen, &mut origin, &mut special, &mut perk, &mut background, &mut equipment, &cfg)
            }
/*--------*/AppScreen::PerkSelect => {
                let state = &mut perk;
                let h = render_perk_select(&ui, &window, state, &mut screen, &db, &mut character, perk_resolution.is_some(), &mut origin, &mut skill, &mut special, &mut background, &mut equipment, &cfg);
                //resolution popup
                if let Some((p_id, add, name)) = state.pending_resolution.take() {
                    let perk = state.perks.iter().find(|p| p.id == p_id).unwrap();
                    if add {
                        if let Some(popup) = state.begin_resolve(perk, add, "".to_string()) {
                            perk_resolution = Some(popup);
                        }
                    } else {
                        if let Some(popup) = state.begin_resolve(perk, add, name) {
                            perk_resolution = Some(popup);
                        }
                    }
                }
                //render popup
                if let Some(popup) = &mut perk_resolution {
                    let result = render_perk_resolution(ui, &window, popup, state, &mut character, &cfg);
                    match result {
                        Some(false) => {
                            if popup.perk_add {
                                if let Some(i) = character.perks.iter().position(|p| p.id == popup.perk_id) {
                                    if character.perks[i].ranks > 1 {
                                        character.perks[i].ranks -= 1;
                                    } else {
                                        character.perks.remove(i);
                                    }
                                    perk.update(&mut character);
                                }
                            }
                            perk_resolution = None;
                        }
                        Some(true) => {
                            perk_resolution = None;
                        }
                        None => {} //it's still open so don't do anything
                    }
                }
                h
            }
/*--------*/AppScreen::StatCalculation => {
                render_stat_calculation(&ui, &window, &mut special, &mut skill, &mut character, &mut screen, &mut origin, &mut perk, &mut background, &mut equipment, &cfg)
            }
/*--------*/AppScreen::BackgroundSelect => {
                let state = &mut background;
                render_background_select(&ui, &window, state, &mut equipment, &db, &mut character, &mut review, &mut screen, &mut origin, &mut special, &mut skill, &mut perk, &cfg)
            }
/*--------*/AppScreen::CharacterReview => {
                let state = &mut review;
                render_character_review(&ui, &window, state, &mut background, &mut equipment, &db, &mut character, &mut screen, &mut origin, &mut special, &mut skill, &mut perk, &cfg)
            }
/*--------*/AppScreen::CharacterSheet => {
                render_character_sheet(&ui, &window, &db, &mut character, &mut screen, &mut sheet_state, &mut origin, &mut special, &mut skill, &mut perk, &mut background, &mut equipment, &cfg);
                0.0
            }
/*--------*/AppScreen::Settings => {
                render_settings(&ui, &window, &mut screen, &mut cfg, &mut crt, &mut current_theme);
                0.0
            }
/*--------*/AppScreen::LoadCharacter => {
                render_load_character(&ui, &window,&mut load_character_state, &mut screen, &db, &mut character, &mut sheet_state, &cfg);
                0.0
            }
/*--------*/AppScreen::ImportCharacter => {
                render_import_character(&ui, &window, &mut import_state, &mut screen, &db, &cfg);
                0.0
            }
        };

        if is_builder_screen {
            //footer on the bottom
            let (_, win_h) = window.size();
            let footer_h: f32 = 48.0 * cfg.ui_scale;
            ui.window("##nav_footer")
                .title_bar(false)
                .resizable(false)
                .movable(false)
                .no_decoration()
                .size([win_w as f32, footer_h], imgui::Condition::Always)
                .position([0.0, win_h as f32 - footer_h], imgui::Condition::Always)
                .build(|| {
                    //render_nav_footer(ui, content_h, &mut screen, &origin, &special, &skill, &perk, &background, &character);
                    render_nav_footer(ui, win_w as f32, footer_h, &mut screen, &origin, &special, &skill, &perk, &mut background, &mut equipment, &mut review, &db, &mut character, &mut sheet_state, &cfg);
                });
        }
        //draw our custom cursor
        render_cursor(&ui, THEMES[current_theme]);

        if crt.enabled {
            crt.begin_capture(&gl);
        } else {
            unsafe {
                gl.clear_color(0.05, 0.05, 0.05, 1.0);
                gl.clear(glow::COLOR_BUFFER_BIT);
        }

        }

        imgui_sdl2.prepare_render(&ui, &window);
        renderer.render(&mut imgui);

        if crt.enabled {
            crt.end_capture_and_draw(&gl);
        }

        window.gl_swap_window();
    }
    Ok(())
}

pub fn render_cursor(ui: &imgui::Ui, theme: &Theme) {
    let [mx, my] = ui.io().mouse_pos;
    if mx < 0.0 || my < 0.0 { return; }

    let draw = ui.get_foreground_draw_list();

    // arrow pointer — tip at (mx, my)
    //let fill    = imgui::ImColor32::from_rgba(210, 210, 185, 255);
    let mut fill = theme.text_dim;
    fill[3] = 128.0;
    //let outline = imgui::ImColor32::from_rgba(20,  20,  20,  200);
    let outline = theme.text;

    // main triangle
    draw.add_triangle(
        [mx,        my       ],
        [mx + 10.0, my + 14.0],
        [mx,        my + 18.0],
        fill,
    ).build();
    // outline
    draw.add_triangle(
        [mx,        my       ],
        [mx + 10.0, my + 14.0],
        [mx,        my + 18.0],
        outline,
    ).thickness(1.0).build();
}