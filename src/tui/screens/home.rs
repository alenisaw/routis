use crate::tui::{
    history::recent_sessions,
    state::{AppState, LayoutMode},
    symbols,
    theme::ThemePalette,
    widgets::{
        dividers::render_vertical_dots,
        mascot::mascot_lines,
        metrics::{metric_lines, metric_lines_compact},
    },
};
use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
    Frame,
};
use unicode_width::UnicodeWidthStr;

const LEFT_WIDE: u16 = 34;
const RIGHT_WIDE: u16 = 42;

pub fn render_header(frame: &mut Frame, area: Rect, state: &AppState, palette: ThemePalette) {
    if area.height == 0 || area.width == 0 {
        return;
    }

    let content = area.inner(Margin {
        horizontal: 1,
        vertical: if area.height >= 16 { 1 } else { 0 },
    });

    match LayoutMode::for_width(area.width.saturating_add(6)) {
        LayoutMode::Wide => render_wide(frame, content, state, palette),
        LayoutMode::Compact | LayoutMode::Stacked => render_medium(frame, content, state, palette),
        LayoutMode::Minimal => render_small(frame, content, state, palette),
    }
}

fn render_wide(frame: &mut Frame, area: Rect, state: &AppState, palette: ThemePalette) {
    let cols = Layout::horizontal([
        Constraint::Length(LEFT_WIDE),
        Constraint::Length(1),
        Constraint::Min(50),
        Constraint::Length(1),
        Constraint::Length(RIGHT_WIDE),
    ])
    .split(area);

    render_profile(frame, inset_right(cols[0], 2), state, palette);
    render_separator(frame, cols[1], palette);
    render_updates_commands(frame, inset_x(cols[2], 2, 2), palette);
    render_separator(frame, cols[3], palette);
    render_model_metrics(frame, inset_left(cols[4], 2), state, palette);
}

fn render_medium(frame: &mut Frame, area: Rect, state: &AppState, palette: ThemePalette) {
    if area.width < 86 {
        render_small(frame, area, state, palette);
        return;
    }

    let cols = Layout::horizontal([
        Constraint::Length(30),
        Constraint::Length(1),
        Constraint::Min(32),
        Constraint::Length(1),
        Constraint::Length(30),
    ])
    .split(area);

    render_profile(frame, inset_right(cols[0], 1), state, palette);
    render_separator(frame, cols[1], palette);
    render_updates_commands(frame, inset_x(cols[2], 2, 1), palette);
    render_separator(frame, cols[3], palette);
    render_model_metrics(frame, inset_left(cols[4], 2), state, palette);
}

fn render_small(frame: &mut Frame, area: Rect, state: &AppState, palette: ThemePalette) {
    let rows = Layout::vertical([
        Constraint::Length(8),
        Constraint::Length(7),
        Constraint::Min(1),
    ])
    .split(area);

    render_profile(frame, rows[0], state, palette);
    render_updates_commands(frame, rows[1], palette);
    render_model_metrics(frame, rows[2], state, palette);
}

fn render_separator(frame: &mut Frame, area: Rect, palette: ThemePalette) {
    if area.height == 0 {
        return;
    }
    render_vertical_dots(frame, area.x, area.y, area.height, palette);
}

fn render_profile(frame: &mut Frame, area: Rect, state: &AppState, palette: ThemePalette) {
    let display_name = truncate(&state.config.display_name, 22);
    if area.height < 10 {
        let lines = vec![
            Line::styled(
                center_display(&format!("Welcome, {display_name}!"), area.width),
                palette.section_title(),
            ),
            centered_workspace_line(area.width, palette),
        ];
        frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), area);
        return;
    }

    let mut lines = Vec::new();
    let content_height = mascot_lines().len() + 3;
    let top_pad = (area.height as usize)
        .saturating_sub(content_height)
        .saturating_div(2)
        .min(2);
    for _ in 0..top_pad {
        lines.push(Line::raw(""));
    }

    for line in mascot_lines() {
        lines.push(Line::styled(
            center_display(line.trim(), area.width),
            palette.accent(),
        ));
    }

    if area.height > 10 {
        lines.push(Line::raw(""));
    }
    lines.push(Line::styled(
        center_display(&format!("Welcome, {display_name}!"), area.width),
        palette.section_title(),
    ));
    lines.push(centered_workspace_line(area.width, palette));

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), area);
}

fn render_updates_commands(frame: &mut Frame, area: Rect, palette: ThemePalette) {
    let mut lines = vec![
        Line::styled("Releases", palette.section_title()),
        bullet(release_notes()[0], area.width, palette),
        bullet(release_notes()[1], area.width, palette),
        Line::raw(""),
        section_rule(area.width, palette),
        Line::styled("Recent Sessions", palette.section_title()),
    ];

    let recent = recent_sessions(2);
    if recent.is_empty() {
        lines.push(Line::styled("no local sessions yet", palette.muted()));
    } else {
        for (index, (task, when)) in recent.into_iter().enumerate() {
            lines.push(recent_line(&task, &when, area.width, index == 0, palette));
        }
    }

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), area);
}

fn render_model_metrics(frame: &mut Frame, area: Rect, state: &AppState, palette: ThemePalette) {
    let mut lines = vec![Line::styled("Metrics", palette.section_title())];

    let metrics = if area.height <= 10 {
        metric_lines_compact(&state.metrics, palette, area.width)
    } else {
        metric_lines(&state.metrics, palette, area.width)
    };
    lines.extend(metrics);
    if area.height > 8 {
        lines.push(section_rule(area.width, palette));
        lines.push(kv_line_width("today", "0 tasks", area.width, palette));
        lines.push(kv_line_width("sessions", "2 shown", area.width, palette));
    }
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), area);
}

fn bullet(text: &'static str, width: u16, palette: ThemePalette) -> Line<'static> {
    Line::from(vec![
        Span::styled("* ", palette.accent()),
        Span::styled(
            truncate(text, (width as usize).saturating_sub(2)),
            palette.text(),
        ),
    ])
}

pub fn kv_line(label: &'static str, value: impl ToString, palette: ThemePalette) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label:<9}"), palette.muted()),
        Span::styled(value.to_string(), palette.text()),
    ])
}

fn kv_line_width(
    label: &'static str,
    value: impl ToString,
    width: u16,
    palette: ThemePalette,
) -> Line<'static> {
    let value = truncate(&value.to_string(), value_width(width));
    Line::from(vec![
        Span::styled(format!("{label:<9}"), palette.muted()),
        Span::styled(value, palette.text()),
    ])
}

fn recent_line(
    task: &str,
    when: &str,
    width: u16,
    selected: bool,
    palette: ThemePalette,
) -> Line<'static> {
    let marker = if selected { "> " } else { "  " };
    let when_width = UnicodeWidthStr::width(when);
    let available = width as usize;
    let task_width = available.saturating_sub(2 + when_width + 2).max(8);
    let task = truncate(task, task_width);
    let used = 2 + UnicodeWidthStr::width(task.as_str()) + when_width;
    let gap = available.saturating_sub(used).max(2);

    Line::from(vec![
        Span::styled(
            marker,
            if selected {
                palette.accent()
            } else {
                palette.dim()
            },
        ),
        Span::styled(
            task,
            if selected {
                palette.text().bold()
            } else {
                palette.text()
            },
        ),
        Span::raw(" ".repeat(gap)),
        Span::styled(when.to_string(), palette.muted()),
    ])
}

pub fn sep_line(palette: ThemePalette) -> Line<'static> {
    Line::styled(". . . . . . . . . . . .", palette.dim())
}

pub fn workspace_label() -> String {
    std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .map(|n| format!("~/{n}"))
        .unwrap_or_else(|| "~/local".to_string())
}

fn center_display(value: &str, width: u16) -> String {
    let visible = UnicodeWidthStr::width(value);
    let width = width as usize;
    if visible >= width {
        return truncate(value, width);
    }
    format!("{}{}", " ".repeat((width - visible) / 2), value)
}

fn truncate(value: &str, max: usize) -> String {
    if UnicodeWidthStr::width(value) <= max {
        return value.to_string();
    }
    let mut out = String::new();
    let mut width = 0;
    for ch in value.chars() {
        let ch_width = UnicodeWidthStr::width(ch.to_string().as_str());
        if width + ch_width + 1 > max {
            break;
        }
        out.push(ch);
        width += ch_width;
    }
    out.push_str("...");
    out
}

fn value_width(width: u16) -> usize {
    (width as usize).saturating_sub(10).max(8)
}

fn section_rule(width: u16, palette: ThemePalette) -> Line<'static> {
    let line = symbols::H.repeat(width as usize);
    Line::styled(line, palette.border())
}

fn centered_workspace_line(width: u16, palette: ThemePalette) -> Line<'static> {
    let path = workspace_label();
    let label = "Workspace: ";
    let visible = UnicodeWidthStr::width(label) + UnicodeWidthStr::width(path.as_str());
    let left = (width as usize).saturating_sub(visible) / 2;
    Line::from(vec![
        Span::raw(" ".repeat(left)),
        Span::styled(label, palette.muted()),
        Span::styled(path, palette.path()),
    ])
}

fn inset_left(area: Rect, left: u16) -> Rect {
    inset_x(area, left, 0)
}

fn inset_right(area: Rect, right: u16) -> Rect {
    inset_x(area, 0, right)
}

fn inset_x(area: Rect, left: u16, right: u16) -> Rect {
    Rect {
        x: area.x.saturating_add(left),
        y: area.y,
        width: area.width.saturating_sub(left.saturating_add(right)),
        height: area.height,
    }
}

fn release_notes() -> [&'static str; 2] {
    [
        "v0.3.0 Repo context and session store.",
        "v0.2.2 TUI command and layout polish.",
    ]
}
