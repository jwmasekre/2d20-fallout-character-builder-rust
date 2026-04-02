mod db;
mod config;
mod screens;

use db::Db;
use sdl2::video::{GLProfile,Window};
use imgui_sdl2::ImguiSdl2;
use imgui_opengl_renderer::Renderer;
use imgui::{Ui};
//use fallout_2d20_core::special::SpecialStats;
use std::os::raw::c_void;
use anyhow::Result;
use glow::HasContext;
use screens::main_menu::render_main_menu;
use screens::new_character::{NewCharacterState, render_new_character};
use screens::special::{ render_special, SpecialState, MutantType };
use screens::skills::{ render_skills, SkillsState, sync_trait_effects };
use screens::perks:: { render_perks, PerksState, load_perks };

use crate::screens::new_character::render_text_wrapped;

struct Theme {
    name: &'static str,
    text: [f32; 4],
    text_dim: [f32; 4],
    text_desc: [f32; 4],
    window_bg: [f32; 4],
    header: [f32; 4],
    header_hovered: [f32; 4],
    header_active: [f32; 4],
    button: [f32; 4],
    button_hovered: [f32; 4],
    button_active: [f32; 4],
    slider_grab: [f32; 4],
    slider_grab_active: [f32; 4],
    frame_bg: [f32; 4],
    frame_bg_hovered: [f32; 4],
    tab: [f32; 4],
    tab_hovered: [f32; 4],
    tab_active: [f32; 4],
    title_bg: [f32; 4],
    title_bg_active: [f32; 4],
    separator: [f32; 4],
}

pub const BAR_HEIGHT: f32 = 32.0;

const THEME_CAPITAL: Theme = Theme {
    name: "Capital",
    text:              [0.10, 1.00, 0.10, 1.0],
    text_dim:          [0.05, 0.55, 0.05, 1.0],
    text_desc:         [0.07, 0.75, 0.07, 1.0],
    window_bg:         [0.02, 0.07, 0.02, 1.0],
    header:            [0.00, 0.30, 0.00, 1.0],
    header_hovered:    [0.00, 0.45, 0.00, 1.0],
    header_active:     [0.00, 0.60, 0.00, 1.0],
    button:            [0.00, 0.25, 0.00, 1.0],
    button_hovered:    [0.00, 0.40, 0.00, 1.0],
    button_active:     [0.00, 0.55, 0.00, 1.0],
    slider_grab:       [0.10, 0.90, 0.10, 1.0],
    slider_grab_active:[0.20, 1.00, 0.20, 1.0],
    frame_bg:          [0.00, 0.12, 0.00, 1.0],
    frame_bg_hovered:  [0.00, 0.20, 0.00, 1.0],
    tab:               [0.00, 0.18, 0.00, 1.0],
    tab_hovered:       [0.00, 0.40, 0.00, 1.0],
    tab_active:        [0.00, 0.30, 0.00, 1.0],
    title_bg:          [0.00, 0.10, 0.00, 1.0],
    title_bg_active:   [0.00, 0.20, 0.00, 1.0],
    separator:         [0.00, 0.45, 0.00, 1.0],
};

const THEME_MOJAVE: Theme = Theme {
    name: "Mojave",
    text:              [1.00, 0.75, 0.10, 1.0],
    text_dim:          [0.65, 0.45, 0.05, 1.0],
    text_desc:         [0.85, 0.60, 0.07, 1.0],
    window_bg:         [0.08, 0.05, 0.01, 1.0],
    header:            [0.35, 0.20, 0.00, 1.0],
    header_hovered:    [0.50, 0.30, 0.00, 1.0],
    header_active:     [0.65, 0.40, 0.00, 1.0],
    button:            [0.30, 0.18, 0.00, 1.0],
    button_hovered:    [0.50, 0.28, 0.00, 1.0],
    button_active:     [0.65, 0.38, 0.00, 1.0],
    slider_grab:       [0.90, 0.65, 0.10, 1.0],
    slider_grab_active:[1.00, 0.80, 0.20, 1.0],
    frame_bg:          [0.18, 0.10, 0.00, 1.0],
    frame_bg_hovered:  [0.28, 0.16, 0.00, 1.0],
    tab:               [0.22, 0.13, 0.00, 1.0],
    tab_hovered:       [0.50, 0.30, 0.00, 1.0],
    tab_active:        [0.38, 0.22, 0.00, 1.0],
    title_bg:          [0.12, 0.07, 0.00, 1.0],
    title_bg_active:   [0.25, 0.14, 0.00, 1.0],
    separator:         [0.70, 0.50, 0.05, 1.0],
};

const THEME_COMMONWEALTH: Theme = Theme {
    name: "Commonwealth",
    text:              [0.90, 0.88, 0.60, 1.0],
    text_dim:          [0.55, 0.53, 0.30, 1.0],
    text_desc:         [0.70, 0.68, 0.40, 1.0],
    window_bg:         [0.04, 0.06, 0.15, 1.0],
    header:            [0.10, 0.18, 0.45, 1.0],
    header_hovered:    [0.15, 0.28, 0.60, 1.0],
    header_active:     [0.20, 0.38, 0.75, 1.0],
    button:            [0.08, 0.15, 0.40, 1.0],
    button_hovered:    [0.15, 0.25, 0.58, 1.0],
    button_active:     [0.20, 0.35, 0.70, 1.0],
    slider_grab:       [0.85, 0.80, 0.20, 1.0],
    slider_grab_active:[1.00, 0.95, 0.30, 1.0],
    frame_bg:          [0.06, 0.10, 0.25, 1.0],
    frame_bg_hovered:  [0.10, 0.16, 0.38, 1.0],
    tab:               [0.06, 0.12, 0.30, 1.0],
    tab_hovered:       [0.15, 0.28, 0.60, 1.0],
    tab_active:        [0.12, 0.22, 0.50, 1.0],
    title_bg:          [0.04, 0.07, 0.20, 1.0],
    title_bg_active:   [0.08, 0.14, 0.38, 1.0],
    separator:         [0.70, 0.65, 0.15, 1.0],
};


fn apply_theme(imgui: &mut imgui::Context, theme: &Theme) {
    let style = imgui.style_mut();
    style.colors[imgui::StyleColor::Text as usize]             = theme.text;
    style.colors[imgui::StyleColor::TextDisabled as usize]     = theme.text_dim;
    style.colors[imgui::StyleColor::DragDropTarget as usize]   = theme.text_desc;
    style.colors[imgui::StyleColor::WindowBg as usize]         = theme.window_bg;
    style.colors[imgui::StyleColor::Header as usize]           = theme.header;
    style.colors[imgui::StyleColor::HeaderHovered as usize]    = theme.header_hovered;
    style.colors[imgui::StyleColor::HeaderActive as usize]     = theme.header_active;
    style.colors[imgui::StyleColor::Button as usize]           = theme.button;
    style.colors[imgui::StyleColor::ButtonHovered as usize]    = theme.button_hovered;
    style.colors[imgui::StyleColor::ButtonActive as usize]     = theme.button_active;
    style.colors[imgui::StyleColor::SliderGrab as usize]       = theme.slider_grab;
    style.colors[imgui::StyleColor::SliderGrabActive as usize] = theme.slider_grab_active;
    style.colors[imgui::StyleColor::FrameBg as usize]          = theme.frame_bg;
    style.colors[imgui::StyleColor::FrameBgHovered as usize]   = theme.frame_bg_hovered;
    style.colors[imgui::StyleColor::Tab as usize]              = theme.tab;
    style.colors[imgui::StyleColor::TabHovered as usize]       = theme.tab_hovered;
    style.colors[imgui::StyleColor::TabActive as usize]        = theme.tab_active;
    style.colors[imgui::StyleColor::TitleBg as usize]          = theme.title_bg;
    style.colors[imgui::StyleColor::TitleBgActive as usize]    = theme.title_bg_active;
    style.colors[imgui::StyleColor::Separator as usize]        = theme.separator;
    style.colors[imgui::StyleColor::PopupBg as usize]          = theme.window_bg;
    style.colors[imgui::StyleColor::ChildBg as usize]          = theme.window_bg;
}

#[derive(Debug, Clone, PartialEq)]
enum AppScreen {
    MainMenu,
    NewCharacter,
    LoadCharacter,
    ImportCharacter,
    Special,
    Skills,
    Perks,
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

    let themes: [&Theme; 3] = [&THEME_CAPITAL, &THEME_MOJAVE, &THEME_COMMONWEALTH];
    let mut current_theme: usize = 0;

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
    
    let mut pending_theme: Option<usize> = Some(0);

    let mut show_about = false;
    let mut new_char_state: Option<NewCharacterState> = None;
    let mut special_state: Option<SpecialState> = None;
    let mut skills_state: Option<SkillsState> = None;
    let mut perks_state: Option<PerksState> = None;
    
    let db_path = config::db_path();
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
            render_text_wrapped(true, false, ui, "v0.1.5, 20260401", 16.0, aw - 32.0);
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

                render_perks(&ui, &window, state, &mut screen);
                if screen != AppScreen::Perks {
                    perks_state = None;
                }
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