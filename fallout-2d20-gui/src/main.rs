mod db;
mod config;
mod screens;
mod character;
mod theme;

use db::Db;
use sdl2::video::{ GLProfile,Window };
use imgui_sdl2::ImguiSdl2;
use imgui_opengl_renderer::Renderer;
use imgui::{ Ui };
use std::os::raw::c_void;
use anyhow::Result;
use glow::HasContext;
use config::{ load_config, save_config, AppConfig };
use theme::{ Theme, BAR_HEIGHT, THEME_CAPITAL, THEME_COMMONWEALTH, THEME_MOJAVE, THEME_NUCLEAR_SHADOW, THEME_NUCLEAR_WINTER, apply_theme};
use screens::main_menu::render_main_menu;
use screens::new_character::{ NewCharacterState, render_new_character, render_text_wrapped };
use screens::special::{ render_special, SpecialState, MutantType };
use screens::skills::{ render_skills, SkillsState, sync_trait_effects };
use screens::perks::{ render_perks, PerksState, load_perks, render_perk_resolution, PerkResolutionPopup };
use screens::stats::{ render_stats, ComputedStats, compute_stats };
use screens::equipment::{ render_equipment, EquipmentState };
use screens::review::render_review;

#[derive(Debug, Clone, PartialEq)]
enum AppScreen {
    MainMenu,
    NewCharacter,
    LoadCharacter,
    ImportCharacter,
    Special,
    Skills,
    Perks,
    Stats,
    Equipment,
    Review
}

fn render_placeholder(ui: &Ui, window: &Window, title: &str, screen: &mut AppScreen) {
    let (win_w, win_h) = window.size();
    let w = 500.0_f32;
    let h = 200.0_f32;

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
    let sdl_context = sdl2::init().map_err(|e| anyhow::anyhow!(e))?;
    let video_subsystem = sdl_context.video().map_err(|e| anyhow::anyhow!(e))?;

    video_subsystem.gl_attr().set_context_profile(GLProfile::Core);
    video_subsystem.gl_attr().set_context_version(3, 2);

    let window = video_subsystem
        .window("Fallout 2d20 Character Manager", 1900, 950)
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

    let themes: [&Theme; 5] = [&THEME_CAPITAL, &THEME_MOJAVE, &THEME_COMMONWEALTH, &THEME_NUCLEAR_WINTER, &THEME_NUCLEAR_SHADOW];

    let cfg = load_config();
    let mut current_theme = cfg.theme_index.min(themes.len() - 1);

    let mut imgui = imgui::Context::create();
    apply_theme(&mut imgui, themes[current_theme]);
    imgui.set_ini_filename(None);

    let mut imgui_sdl2 = ImguiSdl2::new(&mut imgui, &window);
    let renderer = Renderer::new(&mut imgui, |s| {
        video_subsystem.gl_get_proc_address(s) as *const c_void
    });

    let mut event_pump = sdl_context.event_pump().map_err(|e| anyhow::anyhow!(e))?;

    let mut screen = AppScreen::MainMenu;
    let mut selected_menu_item: i32 = 0;
    let menu_items = ["New Character", "Load Character", "Import Character", "Quit"];
    
    let mut pending_theme: Option<usize> = Some(current_theme);

    let mut show_about = false;
    let mut new_char_state: Option<NewCharacterState> = None;
    let mut special_state: Option<SpecialState> = None;
    let mut skills_state: Option<SkillsState> = None;
    let mut perks_state: Option<PerksState> = None;
    let mut perk_resolution: Option<PerkResolutionPopup> = None;
    let mut equipment_state: Option<EquipmentState> = None;
    let mut stats_state: Option<ComputedStats> = None;
    
    let db_path = config::db_path();
    /*
    let db_path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("fallout_2d20.db");
    */
    std::fs::create_dir_all(db_path.parent().unwrap())?;
    let db = Db::connect(&format!("sqlite:{}", db_path.display()))?;

    'main: loop {
        for event in event_pump.poll_iter() {
            imgui_sdl2.handle_event(&mut imgui, &event);
            if let sdl2::event::Event::Quit { .. } = event {
                break 'main;
            }
        }
        
        if let Some(t) = pending_theme.take() {
            apply_theme(&mut imgui, themes[t]);
        }

        imgui_sdl2.prepare_frame(imgui.io_mut(), &window, &event_pump.mouse_state());
        let ui = imgui.frame();

        // ── Always-visible theme bar ──────────────────────────────────────────────────
        let (win_w, _win_h) = window.size();

        ui.window("##theme_bar")
            .title_bar(false)
            .resizable(false)
            .movable(false)
            .collapsible(false)
            .no_decoration()
            .size([win_w as f32, BAR_HEIGHT], imgui::Condition::Always)
            .position([0.0, 0.0], imgui::Condition::Always)
            .build(|| {
                ui.set_cursor_pos([8.0, 7.0]);
                ui.text_colored(themes[current_theme].text_dim, "Theme:");
                ui.same_line();
                for (i, theme) in themes.iter().enumerate() {
                    if ui.radio_button_bool(theme.name, current_theme == i) {
                        current_theme = i;
                        pending_theme = Some(i);
                        save_config(&AppConfig { theme_index: i });
                    }
                    if i < themes.len() - 1 {
                        ui.same_line();
                    }
                }
                // About button, right-aligned
                let button_w = 60.0_f32;
                let button_x = win_w as f32 - button_w - 8.0;
                ui.set_cursor_pos([button_x, 4.0]);
                if ui.button("About") {
                    show_about = true; // open or re-center
                }
            });

        if show_about {
            let (win_w, win_h) = window.size();
            let aw = 400.0_f32;
            let ah = 220.0_f32;
            let center = [(win_w as f32 - aw) * 0.5, (win_h as f32 - ah) * 0.5];

            // Only force position when first opened or re-centered (not every frame)
            let condition = if ui.is_mouse_released(imgui::MouseButton::Left) {
                imgui::Condition::Appearing
            } else {
                imgui::Condition::Appearing
            };

            ui.window("##about")
                .title_bar(false)
                .resizable(false)
                .movable(true)          // draggable
                .collapsible(false)
                .size([aw, ah], imgui::Condition::Always)
                .position(center, imgui::Condition::Once) // Once = only set pos on first appear
                //.bring_current_window_to_display_front()  // always on top
                .bring_to_front_on_focus(true)
                .build(|| {
                    // Title row with X button
                    let close_x = aw - 28.0;
                    ui.text("About");
                    ui.same_line_with_pos(close_x);
                    if ui.button("X##about_close") {
                        show_about = false;
                    }
                    ui.separator();
                    ui.spacing();

                    ui.text("fallout 2d20 character manager");
                    ui.spacing();
                    render_text_wrapped(true, false, ui, "v0.1.9, 20260408", 16.0, aw - 32.0);
                    ui.spacing();
                    ui.text_wrapped("A character creation and management tool for the 2d20 ttrpg system.");
                    ui.text_colored([0.90, 0.10, 0.50, 1.00], "by josh");
                    ui.spacing();
                    ui.separator();
                    ui.spacing();
                    render_text_wrapped(true, false, ui, "built with rust//imgui//sdl2", 16.0, aw - 32.0);
                });
        }

        // ── Screen content (offset below the bar) ────────────────────────────────────
        match screen {
            AppScreen::MainMenu => {
                render_main_menu(&ui, &window, &mut screen, &mut selected_menu_item, &menu_items);
            }
            AppScreen::NewCharacter => {
                let state = new_char_state.get_or_insert_with(|| NewCharacterState::load(&db));
                render_new_character(&ui, &window, state, &mut screen, &db);
                if screen == AppScreen::MainMenu {
                    new_char_state = None;
                }
            }
            AppScreen::LoadCharacter => {
                render_placeholder(&ui, &window, "Load Character", &mut screen);
            }
            AppScreen::ImportCharacter => {
                render_placeholder(&ui, &window, "Import Character", &mut screen);
            }
            // In the match:
            AppScreen::Special => {
                // Pull is_gifted and mutant_type from new_char_state
                let (is_gifted, mutant_type) = new_char_state
                    .as_ref()
                    .map(|s| (s.has_gifted_trait(), s.mutant_type()))
                    .unwrap_or((false, MutantType::None));

                let state = special_state.get_or_insert_with(|| {
                    SpecialState::new(is_gifted, mutant_type)
                });

                state.is_gifted = is_gifted;
                state.mutant_type = mutant_type;

                if !is_gifted {
                    state.gifted_selected = [false; 7];
                }

                render_special(&ui, &window, state, &mut screen);
                if screen == AppScreen::MainMenu {
                    special_state = None;
                }
            }
            AppScreen::Skills => {
                // Sync intelligence from completed SPECIAL state
                let intelligence = special_state
                    .as_ref()
                    .map(|s| s.display_value(crate::screens::special::I))
                    .unwrap_or(5);
                let level = new_char_state.as_ref().map(|s| s.level).unwrap_or(1);

                let state = skills_state.get_or_insert_with(|| {
                    SkillsState::new(intelligence, level)
                });

                state.intelligence = intelligence;
                state.level = level;

                // Sync trait effects each frame
                if let Some(nc) = &new_char_state {
                    let selected_ids: Vec<i32> = nc.traits.iter().enumerate()
                        .filter(|(i, _)| nc.selected_traits.get(*i).copied().unwrap_or(false))
                        .map(|(_, t)| t.id as i32)
                        .collect();            
                    sync_trait_effects(state, &selected_ids, nc.is_ghoul);
                }

                render_skills(&ui, &window, state, &mut screen);
                if screen == AppScreen::MainMenu {
                    skills_state = None;
                }
            }
            AppScreen::Perks => {
                let special_display = special_state
                    .as_ref()
                    .map(|s| std::array::from_fn(|i| s.display_value(i).into()))
                    .unwrap_or([5; 7]);
                let level = new_char_state.as_ref().map(|s| s.level).unwrap_or(1).into();
                let is_ghoul = new_char_state.as_ref().map(|s| s.is_ghoul).unwrap_or(false);
                let is_super_mutant = new_char_state.as_ref()
                    .map(|s| s.mutant_type() != MutantType::None)
                    .unwrap_or(false);
                let perk_trait = new_char_state.as_ref()
                    .map(|s| s.traits.iter().enumerate()
                        .any(|(i, t)| t.id == 10 && s.selected_traits.get(i).copied().unwrap_or(false)))
                    .unwrap_or(false);

                let state = perks_state.get_or_insert_with(|| {
                    let all_perks = load_perks(&db);
                    PerksState::new(all_perks, level, special_display,
                        is_ghoul, false, is_super_mutant, false, perk_trait)
                });

                // Sync mutable context each frame
                state.level = level;
                state.special = special_display;
                state.is_ghoul = is_ghoul;
                state.is_super_mutant = is_super_mutant;
                state.perk_trait = perk_trait;

                render_perks(&ui, &window, state, &mut screen, perk_resolution.is_some());
                // Open resolution popup if a special perk was just taken
                if let Some(pid) = state.pending_resolution.take() {
                    let pname = state.all_perks.iter()
                        .find(|p| p.id == pid)
                        .map(|p| p.name.as_str())
                        .unwrap_or("");
                    if let Some(popup) = state.begin_resolve(pid, pname) {
                        perk_resolution = Some(popup);
                    }
                }
                // Render resolution popup if open
                if let Some(popup) = &mut perk_resolution {
                    let special_max: [i32; 7] = std::array::from_fn(|i| {
                        special_state.as_ref().map(|s| s.stat_max(i)).unwrap_or(10)
                    });
                    let result = render_perk_resolution(
                        &ui, &window, popup,
                        &mut state.special,
                        &special_max,
                        skills_state.as_mut().unwrap(),
                        special_state.as_mut().unwrap(),
                    );
                    match result {
                        Some(false) => {
                            // Cancelled — remove the perk
                            let pid = popup.perk_id;
                            state.remove_perk(pid);
                            perk_resolution = None;
                        }
                        Some(true) => {
                            perk_resolution = None;
                        }
                        None => {} // still open
                    }
                }
                if screen == AppScreen::MainMenu {
                    perks_state = None;
                }
            }
            AppScreen::Stats => {
                let special = special_state.as_ref().unwrap();
                let traits = new_char_state.as_ref().unwrap();
                let perks = perks_state.as_ref().unwrap();
                let state = stats_state.get_or_insert_with(|| {
                    compute_stats(special, traits, perks)
                });
                render_stats(
                    &ui, &window,
                    special,
                    skills_state.as_ref().unwrap(),
                    perks,
                    traits,
                    &mut screen,
                    state,
                );
                if screen == AppScreen::MainMenu {
                    stats_state = None;
                }
            }
            AppScreen::Equipment => {
                let origin_id = new_char_state.as_ref()
                    .and_then(|s| s.selected_origin_id());
                let state = equipment_state.get_or_insert_with(|| {
                    let all_backgrounds = screens::equipment::load_backgrounds(&db);
                    EquipmentState::new(all_backgrounds)
                });

                if state.origin_id != origin_id {
                    state.origin_id = origin_id;
                    state.reset_selection();
                }

                render_equipment(&ui, &window, state, &db, &mut screen);

                if screen == AppScreen::MainMenu {
                    equipment_state = None;
                }
            }
            AppScreen::Review => {
                render_review(
                    &ui, &window,
                    new_char_state.as_ref().unwrap(),
                    special_state.as_ref().unwrap(),
                    skills_state.as_ref().unwrap(),
                    perks_state.as_ref().unwrap(),
                    stats_state.as_ref().unwrap(),
                    equipment_state.as_ref().unwrap(),
                    &mut screen,
                    themes[current_theme],
                    &db,
                );
            }

        }

        unsafe {
            gl.clear_color(0.05, 0.05, 0.05, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT);
        }

        imgui_sdl2.prepare_render(&ui, &window);
        renderer.render(&mut imgui);

        window.gl_swap_window();
    }

    Ok(())
}