use crate::tui::{
    state::{AppState, SessionPhase},
    theme::ThemePalette,
};
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use unicode_width::UnicodeWidthStr;

pub fn render_input(frame: &mut Frame, area: Rect, state: &AppState, palette: ThemePalette) {
    let awaiting_confirmation =
        state.session.phase == SessionPhase::AwaitingConfirmation && state.ui.input.is_empty();
    let raw = if awaiting_confirmation {
        "[proceed] / [cancel]".to_string()
    } else if state.ui.input.is_empty() {
        "Type a task or / for commands...".to_string()
    } else {
        state.ui.input.clone()
    };
    let value = clip_to_width(&raw, area.width as usize);
    let value_style = if awaiting_confirmation {
        palette.warning().bold()
    } else if state.ui.input.is_empty() {
        palette.dim()
    } else {
        palette.text()
    };

    let line = Line::from(vec![Span::styled(value, value_style)]);

    frame.render_widget(Paragraph::new(line), area);
}

fn clip_to_width(value: &str, max: usize) -> String {
    if UnicodeWidthStr::width(value) <= max {
        return value.to_string();
    }
    let mut out = String::new();
    let mut width = 0;
    for ch in value.chars() {
        let ch_width = UnicodeWidthStr::width(ch.to_string().as_str());
        if width + ch_width > max {
            break;
        }
        out.push(ch);
        width += ch_width;
    }
    out
}
