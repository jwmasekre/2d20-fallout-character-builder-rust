use fallout_2d20_core::structs::{AppConfig, AppScreen};
use imgui::Ui;
use sdl2::video::Window;
use crate::{
    config::{save_config},
    crt::CrtEffect,
    //theme::THEMES,
};

pub fn render_settings(
    ui: &Ui,
    window: &Window,
    screen: &mut AppScreen,
    cfg: &mut AppConfig,
    crt: &mut CrtEffect,
    _current_theme: &mut usize,
) {
    let (win_w, win_h) = window.size();
    let w = (win_w as f32 - 80.0).min(860.0);
    let h = win_h as f32 - 80.0;

    ui.window("##settings")
        .title_bar(false)
        .resizable(false)
        .movable(false)
        .size([w, h], imgui::Condition::Always)
        .position(
            [(win_w as f32 - w) * 0.5, (win_h as f32 - h) * 0.5],
            imgui::Condition::Always,
        )
        .build(|| {
            // Header
            ui.text("Settings");
            ui.same_line_with_pos(w - 84.0);
            if ui.button("< Back##settings_back") {
                *screen = AppScreen::MainMenu;
            }
            ui.separator();
            ui.spacing();

            // ── PATHS ────────────────────────────────────────────────
            ui.text("Paths");
            ui.separator();
            ui.spacing();

            ui.text("Database:");
            ui.same_line();
            ui.text_disabled(cfg.db_path.to_string_lossy().as_ref());
            ui.same_line();
            if ui.button("Browse##db_browse") {
                // rfd file dialog (see note below)
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("SQLite", &["db", "sqlite"])
                    .pick_file()
                {
                    cfg.db_path = path;
                    save_config(cfg);
                    // Note: db reconnect requires app restart — warn the user
                }
            }
            ui.same_line();
            ui.text_colored([1.0, 0.8, 0.2, 1.0], "(restart required)");

            ui.spacing();
            ui.text("Font:");
            ui.same_line();
            let font_label = cfg.font_path.as_ref()
                .map(|p| p.file_name().unwrap_or_default().to_string_lossy().into_owned())
                .unwrap_or_else(|| "Default".to_string());
            ui.text_disabled(&font_label);
            ui.same_line();
            if ui.button("Browse##font_browse") {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("TrueType Font", &["ttf", "otf"])
                    .pick_file()
                {
                    cfg.font_path = Some(path);
                    save_config(cfg);
                }
            }
            ui.same_line();
            if ui.button("Reset##font_reset") {
                cfg.font_path = None;
                save_config(cfg);
            }
            ui.same_line();
            ui.text_colored([1.0, 0.8, 0.2, 1.0], "(restart required)");

            ui.spacing();
            ui.text("Font Size:");
            ui.same_line();
            if ui.button("-##font_dec") {
                if cfg.font_size > 12.0 {
                    cfg.font_size -= 0.5;
                    save_config(cfg);
                }
            }
            ui.same_line();
            ui.text(format!("{:4.1}", cfg.font_size));
            ui.same_line();
            if ui.button("+##font_inc") {
                if cfg.font_size < 32.0 {
                    cfg.font_size += 0.5;
                    save_config(cfg);
                }
            }
            ui.same_line();
            ui.text_colored([1.0, 0.8, 0.2, 1.0], "(restart required)");

            ui.spacing();
            ui.spacing();

            // ── CRT EFFECT ───────────────────────────────────────────
            ui.text("CRT Effect");
            ui.separator();
            ui.spacing();

            let slider_w = w - 200.0;
            ui.text_disabled("Note: CTRL+Click allows you to manually input a value.");
            ui.spacing();

            ui.text("Distortion:    ");
            ui.same_line();
            ui.set_next_item_width(slider_w);
            if ui.slider("##crt_dist", 0.0_f32, 0.3, &mut crt.distortion) {
                cfg.crt_distortion = crt.distortion;
                save_config(cfg);
            }

            ui.text("Scanlines:     ");
            ui.same_line();
            ui.set_next_item_width(slider_w);
            if ui.slider("##crt_scan", 0.0_f32, 0.15, &mut crt.scanline_strength) {
                cfg.crt_scanline_strength = crt.scanline_strength;
                save_config(cfg);
            }

            ui.text("Vignette Size: ");
            ui.same_line();
            ui.set_next_item_width(slider_w);
            if ui.slider("##crt_vig_m", 1.0_f32, 20.0, &mut crt.vignette_multiplier) {
                cfg.crt_vignette_multiplier = crt.vignette_multiplier;
                save_config(cfg);
            }

            ui.text("Vignette Soft: ");
            ui.same_line();
            ui.set_next_item_width(slider_w);
            if ui.slider("##crt_vig_e", 0.01_f32, 0.6, &mut crt.vignette_exponent) {
                cfg.crt_vignette_exponent = crt.vignette_exponent;
                save_config(cfg);
            }

            ui.text("Roll Speed:    ");
            ui.same_line();
            ui.set_next_item_width(slider_w);
            if ui.slider("##crt_roll", 0.0_f32, 0.4, &mut crt.roll_speed) {
                cfg.crt_roll_speed = crt.roll_speed;
                save_config(cfg);
            }

            ui.text("Tint Strength: ");
            ui.same_line();
            ui.set_next_item_width(slider_w);
            if ui.slider("##crt_tint", 0.0_f32, 0.4, &mut crt.tint_strength) {
                cfg.crt_tint_strength = crt.tint_strength;
                save_config(cfg);
            }

            ui.text("Chromatic Aberration: ");
            ui.same_line();
            ui.set_next_item_width(slider_w);
            if ui.slider_config("##crt_chr_abr", 0.0_f32, 0.002)
                .display_format("%.4f")
                .build(&mut crt.chromatic_aberration) {
                cfg.crt_chromatic_aberration = crt.chromatic_aberration;
                save_config(cfg);
            }

            ui.spacing();
            ui.spacing();

            // ── RESET ────────────────────────────────────────────────
            ui.separator();
            ui.spacing();
            if ui.button("Reset CRT to Defaults") {
                let defaults = AppConfig::default();
                crt.distortion          = defaults.crt_distortion;
                crt.scanline_strength   = defaults.crt_scanline_strength;
                crt.vignette_multiplier = defaults.crt_vignette_multiplier;
                crt.vignette_exponent   = defaults.crt_vignette_exponent;
                crt.roll_speed          = defaults.crt_roll_speed;
                crt.tint_strength       = defaults.crt_tint_strength;
                cfg.crt_distortion          = defaults.crt_distortion;
                cfg.crt_scanline_strength   = defaults.crt_scanline_strength;
                cfg.crt_vignette_multiplier = defaults.crt_vignette_multiplier;
                cfg.crt_vignette_exponent   = defaults.crt_vignette_exponent;
                cfg.crt_roll_speed          = defaults.crt_roll_speed;
                cfg.crt_tint_strength       = defaults.crt_tint_strength;
                cfg.crt_chromatic_aberration       = defaults.crt_chromatic_aberration;
                save_config(cfg);
            }
        });
}