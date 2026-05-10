use crate::tui::{
    history::recent_sessions,
    state::{AppState, LayoutMode},
    theme::ThemePalette,
    widgets::dividers::render_vertical_dots,
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
        horizontal: 0,
        vertical: 0,
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

    render_profile(frame, inset_x(cols[0], 1, 2), state, palette);
    render_separator(frame, cols[1], palette);
    render_updates_commands(frame, inset_x(cols[2], 0, 0), palette);
    render_separator(frame, cols[3], palette);
    render_activity_tracker(frame, inset_x(cols[4], 2, 1), state, palette);
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

    render_profile(frame, inset_x(cols[0], 1, 1), state, palette);
    render_separator(frame, cols[1], palette);
    render_updates_commands(frame, inset_x(cols[2], 0, 0), palette);
    render_separator(frame, cols[3], palette);
    render_activity_tracker(frame, inset_x(cols[4], 2, 1), state, palette);
}

fn render_small(frame: &mut Frame, area: Rect, state: &AppState, palette: ThemePalette) {
    let rows = Layout::vertical([
        Constraint::Length(5),
        Constraint::Length(5),
        Constraint::Min(1),
    ])
    .split(area);

    render_profile(frame, rows[0], state, palette);
    render_updates_commands(frame, rows[1], palette);
    render_activity_tracker(frame, rows[2], state, palette);
}

fn render_separator(frame: &mut Frame, area: Rect, palette: ThemePalette) {
    if area.height == 0 {
        return;
    }
    render_vertical_dots(frame, area.x, area.y, area.height, palette);
}

fn render_profile(frame: &mut Frame, area: Rect, state: &AppState, palette: ThemePalette) {
    let display_name = truncate(&state.config.display_name, 22);
    if area.height < 7 {
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
    let mascot = compact_mascot_lines();
    let content_height = mascot.len() + 3 + usize::from(area.height > 9);
    let top_pad = if area.height >= 10 {
        2.min(area.height as usize).min(
            (area.height as usize)
                .saturating_sub(content_height)
                .saturating_add(1),
        )
    } else {
        (area.height as usize)
            .saturating_sub(content_height)
            .saturating_div(2)
    };
    for _ in 0..top_pad {
        lines.push(Line::raw(""));
    }

    for line in mascot {
        lines.push(Line::styled(
            center_display(line.trim(), area.width),
            palette.accent(),
        ));
    }

    if area.height > 9 {
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
    let chunks = header_sections(area);
    let inner_top = inset_x(chunks[0], 3, 2);
    let releases = vec![
        Line::styled("Releases", palette.section_title()),
        bullet(release_notes()[0], inner_top.width, palette),
        bullet(release_notes()[1], inner_top.width, palette),
    ];
    frame.render_widget(
        Paragraph::new(releases).wrap(Wrap { trim: false }),
        inner_top,
    );
    frame.render_widget(Paragraph::new(dotted_rule(area.width, palette)), chunks[1]);

    let inner_bottom = inset_x(chunks[2], 3, 2);
    let mut lines = vec![Line::styled("Recent Sessions", palette.section_title())];
    let recent = recent_sessions(2);
    if recent.is_empty() {
        lines.push(Line::styled("no local sessions yet", palette.muted()));
    } else {
        for (index, (task, when)) in recent.into_iter().enumerate() {
            lines.push(recent_line(
                &task,
                &when,
                inner_bottom.width,
                index == 0,
                palette,
            ));
        }
    }

    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: false }),
        inner_bottom,
    );
}

fn render_activity_tracker(frame: &mut Frame, area: Rect, state: &AppState, palette: ThemePalette) {
    let lines = vec![
        Line::styled("Activity Tracker", palette.section_title()),
        kv_line_width("tasks", state.metrics.tasks, area.width, palette),
        kv_line_width("sessions", "2 shown", area.width, palette),
        kv_line_width("today", "0 done", area.width, palette),
        kv_line_width(
            "confidence",
            format!("{}%", state.metrics.saved_percent),
            area.width,
            palette,
        ),
    ];
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), area);
}

fn header_sections(area: Rect) -> std::rc::Rc<[Rect]> {
    Layout::vertical([
        Constraint::Length(5.min(area.height)),
        Constraint::Length(1),
        Constraint::Min(1),
    ])
    .split(area)
}

fn dotted_rule(width: u16, palette: ThemePalette) -> Line<'static> {
    let line = "\u{2504}".repeat(width as usize);
    Line::styled(line, palette.border())
}

fn compact_mascot_lines() -> [&'static str; 5] {
    [
        "   ██    ██   ",
        "  ████  ████  ",
        " █  ██████  █ ",
        " ████████████ ",
        "   ██    ██   ",
    ]
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
        "v0.4.0 Auditable routing.",
        "v0.3.0 Repo context and session store.",
    ]
}
