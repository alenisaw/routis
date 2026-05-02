use crate::tui::theme::ThemePalette;
use ratatui::{layout::Rect, text::Line, widgets::Paragraph, Frame};

/// A horizontal dotted rule, filling the available width.
#[must_use]
pub fn h_rule(width: u16, palette: ThemePalette) -> Paragraph<'static> {
    let line = "· ".repeat((width as usize).saturating_add(1) / 2);
    Paragraph::new(Line::styled(line, palette.border()))
}

/// Vertical column separator rendered as one continuous dotted grid line.
pub fn render_vertical_dots(frame: &mut Frame, x: u16, y: u16, height: u16, palette: ThemePalette) {
    let lines: Vec<Line<'_>> = (0..height)
        .map(|_| Line::styled("┊", palette.border()))
        .collect();
    frame.render_widget(Paragraph::new(lines), Rect::new(x, y, 1, height));
}
