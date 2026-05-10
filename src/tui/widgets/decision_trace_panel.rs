use ratatui::{
    layout::Rect,
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use routis_core::DecisionTrace;

use crate::tui::theme::ThemePalette;

#[allow(dead_code)]
pub fn render_decision_trace_panel(
    frame: &mut Frame<'_>,
    area: Rect,
    palette: ThemePalette,
    trace: &DecisionTrace,
) {
    let mut lines = Vec::new();
    lines.push(Line::from(vec![Span::styled(
        "Routis Decision Trace",
        palette.accent().add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(""));
    for line in trace.render_compact_tree().lines() {
        lines.push(Line::from(Span::raw(line.to_string())));
    }

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" trace ")
                .borders(Borders::ALL)
                .border_style(palette.border()),
        )
        .style(palette.text())
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}
