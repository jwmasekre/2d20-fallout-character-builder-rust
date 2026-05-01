use imgui::{ Ui, WindowToken };
use sdl2::video::Window;

use crate::crt::CrtEffect;

pub struct Theme {
    pub name: &'static str,
    pub text: [f32; 4],
    pub text_dim: [f32; 4],
    pub text_desc: [f32; 4],
    pub window_bg: [f32; 4],
    pub header: [f32; 4],
    pub header_hovered: [f32; 4],
    pub header_active: [f32; 4],
    pub button: [f32; 4],
    pub button_hovered: [f32; 4],
    pub button_active: [f32; 4],
    pub slider_grab: [f32; 4],
    pub slider_grab_active: [f32; 4],
    pub frame_bg: [f32; 4],
    pub frame_bg_hovered: [f32; 4],
    pub tab: [f32; 4],
    pub tab_hovered: [f32; 4],
    pub tab_active: [f32; 4],
    pub title_bg: [f32; 4],
    pub title_bg_active: [f32; 4],
    pub separator: [f32; 4],
}

pub const BAR_HEIGHT: f32 = 40.0;

pub const THEME_CAPITAL: Theme = Theme {
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

pub const THEME_MOJAVE: Theme = Theme {
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

pub const THEME_COMMONWEALTH: Theme = Theme {
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

pub const THEME_NUCLEAR_WINTER: Theme = Theme {
    name: "Nuclear Winter",
    text:              [0.10, 0.10, 0.10, 1.0],
    text_dim:          [0.45, 0.45, 0.45, 1.0],
    text_desc:         [0.30, 0.30, 0.30, 1.0],
    window_bg:         [0.88, 0.88, 0.88, 1.0],
    header:            [0.70, 0.70, 0.70, 1.0],
    header_hovered:    [0.80, 0.80, 0.80, 1.0],
    header_active:     [0.60, 0.60, 0.60, 1.0],
    button:            [0.75, 0.75, 0.75, 1.0],
    button_hovered:    [0.85, 0.85, 0.85, 1.0],
    button_active:     [0.60, 0.60, 0.60, 1.0],
    slider_grab:       [0.35, 0.35, 0.35, 1.0],
    slider_grab_active:[0.20, 0.20, 0.20, 1.0],
    frame_bg:          [0.82, 0.82, 0.82, 1.0],
    frame_bg_hovered:  [0.88, 0.88, 0.88, 1.0],
    tab:               [0.78, 0.78, 0.78, 1.0],
    tab_hovered:       [0.88, 0.88, 0.88, 1.0],
    tab_active:        [0.85, 0.85, 0.85, 1.0],
    title_bg:          [0.75, 0.75, 0.75, 1.0],
    title_bg_active:   [0.65, 0.65, 0.65, 1.0],
    separator:         [0.55, 0.55, 0.55, 1.0],
};

pub const THEME_NUCLEAR_SHADOW: Theme = Theme {
    name: "Nuclear Shadow",
    text:              [0.85, 0.85, 0.85, 1.0],
    text_dim:          [0.50, 0.50, 0.50, 1.0],
    text_desc:         [0.65, 0.65, 0.65, 1.0],
    window_bg:         [0.10, 0.10, 0.10, 1.0],
    header:            [0.22, 0.22, 0.22, 1.0],
    header_hovered:    [0.32, 0.32, 0.32, 1.0],
    header_active:     [0.40, 0.40, 0.40, 1.0],
    button:            [0.20, 0.20, 0.20, 1.0],
    button_hovered:    [0.30, 0.30, 0.30, 1.0],
    button_active:     [0.40, 0.40, 0.40, 1.0],
    slider_grab:       [0.60, 0.60, 0.60, 1.0],
    slider_grab_active:[0.80, 0.80, 0.80, 1.0],
    frame_bg:          [0.15, 0.15, 0.15, 1.0],
    frame_bg_hovered:  [0.22, 0.22, 0.22, 1.0],
    tab:               [0.17, 0.17, 0.17, 1.0],
    tab_hovered:       [0.30, 0.30, 0.30, 1.0],
    tab_active:        [0.25, 0.25, 0.25, 1.0],
    title_bg:          [0.12, 0.12, 0.12, 1.0],
    title_bg_active:   [0.18, 0.18, 0.18, 1.0],
    separator:         [0.38, 0.38, 0.38, 1.0],
};

pub const THEMES: [&Theme; 5] = [&THEME_CAPITAL, &THEME_MOJAVE, &THEME_COMMONWEALTH, &THEME_NUCLEAR_WINTER, &THEME_NUCLEAR_SHADOW];

pub fn apply_theme(imgui: &mut imgui::Context, theme: &Theme, crt: &mut CrtEffect) {
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
    let dim = theme.text_dim;
    let max = dim[0].max(dim[1]).max(dim[2]).max(0.001);
    crt.tint = [dim[0] / max, dim[1] / max, dim[2] / max];
}

pub fn render_window<'ui>(
    ui: &'ui Ui,
    window: &Window,
    label: &str,
    title: &str,
) -> Option<(f32, f32, WindowToken<'ui>)> {
    let (win_w, win_h) = window.size();
    let bar_h = BAR_HEIGHT;
    let content_h = win_h as f32 - bar_h;
    let w = (win_w as f32 * 0.85).min(1100.0);
    let h = content_h * 0.92;

    let token = ui.window(label)
        .title_bar(false)
        .resizable(false)
        .movable(false)
        .size([w, h], imgui::Condition::Always)
        .position(
            [(win_w as f32 - w) * 0.5, BAR_HEIGHT + (content_h - h) * 0.5],
            imgui::Condition::Always,
        )
        .begin()?;

    ui.text(title);
    ui.separator();
    ui.spacing();

    Some((w, h, token))
}

pub fn sanitize(s: &str) -> String {
    s.replace('\u{2019}', "'")
}

pub fn render_text_wrapped(disabled: bool, colored: bool, ui: &Ui, text: &str, indent_x: f32, wrap_pos: f32) {
    let cleaned = sanitize(text);
    let lines: Vec<&str> = cleaned.split("\\n").collect();

    let desc_color = ui.style_color(imgui::StyleColor::DragDropTarget);
    let dis_color = ui.style_color(imgui::StyleColor::TextDisabled);

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() { ui.spacing(); continue; }

        if i > 0 {
            let y = ui.cursor_pos()[1];
            ui.set_cursor_pos([indent_x, y]);
        }

        let _wrap = ui.push_text_wrap_pos_with_pos(wrap_pos);

        if disabled {
            let _c = ui.push_style_color(imgui::StyleColor::Text, dis_color);
            ui.text_wrapped(trimmed);
        } else if colored {
            let _c = ui.push_style_color(imgui::StyleColor::Text, desc_color);
            ui.text_wrapped(trimmed);
        } else {
            ui.text_wrapped(trimmed);
        }
    }
}