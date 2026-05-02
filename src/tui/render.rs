use crate::tui::{
    screens::{session::render_shell, setup::render_setup},
    state::{AppMode, AppState},
    theme::ThemePalette,
    widgets::mascot::mascot_lines as widget_mascot_lines,
};
use ratatui::{widgets::Clear, Frame};

#[must_use]
pub fn mascot_lines() -> &'static [&'static str] {
    widget_mascot_lines()
}

pub fn render_app(frame: &mut Frame, state: &AppState) {
    frame.render_widget(Clear, frame.area());
    let palette = ThemePalette::from_theme(&state.config.theme);
    match state.mode {
        AppMode::Setup => render_setup(frame, frame.area(), state, palette),
        AppMode::Home | AppMode::Session | AppMode::Exit => {
            render_shell(frame, frame.area(), state, palette);
        }
    }
}
