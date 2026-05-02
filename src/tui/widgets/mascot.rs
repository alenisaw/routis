use crate::tui::theme::ThemePalette;
use ratatui::{style::Style, text::Line};

const MASCOT_LINES: [&str; 7] = [
    "        ▄▄      ▄▄",
    "       ████    ████",
    "      ██████████████",
    "     ██  ██    ██  ██",
    "     ████████████████",
    "      ██████████████",
    "        ██      ██",
];

#[must_use]
pub fn mascot_lines() -> &'static [&'static str] {
    &MASCOT_LINES
}

/// Mascot rendered with the theme's accent colour.
#[must_use]
pub fn mascot_render_lines(palette: ThemePalette) -> Vec<Line<'static>> {
    mascot_lines()
        .iter()
        .map(|line| Line::styled(*line, Style::default().fg(palette.accent)))
        .collect()
}
