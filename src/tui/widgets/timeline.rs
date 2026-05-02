use crate::tui::{
    state::{AppState, SessionPhase},
    theme::ThemePalette,
};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
    Frame,
};

pub fn render_timeline(frame: &mut Frame, area: Rect, state: &AppState, palette: ThemePalette) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let title_text = format!(" {} ", &state.session.title);
    let (badge_text, badge_style) = phase_badge(state.session.phase, palette);
    let inner_width = area.width as usize;
    let title_len = title_text.chars().count();
    let badge_len = badge_text.chars().count();
    let gap = inner_width.saturating_sub(title_len + badge_len);
    let title_line = Line::from(vec![
        Span::styled(title_text, palette.muted()),
        Span::styled(" ".repeat(gap), Style::default()),
        Span::styled(badge_text, badge_style),
    ]);

    let mut lines: Vec<Line<'_>> = vec![title_line];

    if state.ui.shortcuts_open {
        lines.extend(shortcut_lines(palette));
    } else if !state.session.events.is_empty() {
        for (event_index, event) in state.session.events.iter().enumerate() {
            lines.push(Line::from(vec![
                Span::styled(
                    role_prefix(&event.source),
                    role_style(&event.source, palette).bold(),
                ),
                Span::styled(
                    format!("{:<10}", &event.source),
                    role_style(&event.source, palette).bold(),
                ),
                Span::styled(&event.title, role_style(&event.source, palette)),
            ]));

            for (detail_index, detail) in event.lines.iter().enumerate() {
                let branch = if detail_index + 1 == event.lines.len() {
                    "   └─ "
                } else {
                    "   ├─ "
                };
                lines.push(Line::from(vec![
                    Span::styled(branch, palette.dim()),
                    Span::styled(detail, detail_style(detail, palette)),
                ]));
            }

            if event_index + 1 < state.session.events.len() {
                lines.push(Line::styled("   │", palette.dim()));
            }
        }

        lines.truncate(state.session.visible_lines.max(1).saturating_add(1));

        // Animated phase indicator appended below visible events
        match state.session.phase {
            SessionPhase::Running => lines.push(Line::from(vec![
                Span::styled("   └─ ", palette.dim()),
                Span::styled(spinner(state.ui.frame), palette.cyan().bold()),
                Span::styled("  processing…", palette.muted()),
            ])),
            SessionPhase::AwaitingConfirmation => lines.push(Line::from(vec![
                Span::styled("   └─ ", palette.dim()),
                Span::styled(
                    "◆  Awaiting confirmation — type 'proceed' or 'cancel'",
                    palette.warning().bold(),
                ),
            ])),
            SessionPhase::Ready => lines.push(Line::from(vec![
                Span::styled("   └─ ", palette.dim()),
                Span::styled("✓  Confirmed — ready to execute", palette.success().bold()),
            ])),
            SessionPhase::Cancelled => lines.push(Line::from(vec![
                Span::styled("   └─ ", palette.dim()),
                Span::styled("✗  Session stopped", palette.error()),
            ])),
            SessionPhase::Idle => {}
        }
    }

    let max_scroll = lines.len().saturating_sub(area.height as usize);
    let scroll = state.session.scroll.min(max_scroll) as u16;

    frame.render_widget(
        Paragraph::new(lines)
            .scroll((scroll, 0))
            .wrap(Wrap { trim: false }),
        area,
    );
}

// ── Helpers ───────────────────────────────────────────────────────────────

fn phase_badge(phase: SessionPhase, palette: ThemePalette) -> (&'static str, Style) {
    match phase {
        SessionPhase::Running => (" RUNNING ", palette.badge_running()),
        SessionPhase::AwaitingConfirmation => (" WAITING ", palette.badge_waiting()),
        SessionPhase::Cancelled => (" STOPPED ", palette.badge_cancelled()),
        SessionPhase::Ready => (" READY   ", palette.badge_ready()),
        SessionPhase::Idle => (" IDLE    ", palette.badge_idle()),
    }
}

fn role_prefix(source: &str) -> &'static str {
    match source {
        "You" => "●  ",
        s if s.contains("Codex") || s.contains("Claude") => "◆  ",
        _ => "◇  ",
    }
}

fn role_style(source: &str, palette: ThemePalette) -> Style {
    match source {
        "You" => palette.text(),
        s if s.contains("Claude") => Style::default().fg(Color::Rgb(214, 132, 92)),
        s if s.contains("Codex") => palette.text(),
        _ => palette.accent(),
    }
}

fn detail_style(line: &str, palette: ThemePalette) -> Style {
    if line.contains("Awaiting") || line.contains("confirm") {
        palette.warning()
    } else if line.contains("cancel") || line.contains("error") || line.contains("Error") {
        palette.error()
    } else if line.contains("codex exec") || line.contains("Command preview") {
        palette.cyan()
    } else if line.contains("checked") || line.contains("resolved") || line.contains("Found") {
        palette.success()
    } else {
        palette.text()
    }
}

fn shortcut_lines(palette: ThemePalette) -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            Span::styled("Routis  ", palette.accent().bold()),
            Span::styled("Keyboard shortcuts", palette.text().bold()),
        ]),
        Line::styled("   ├─ Enter    send or confirm", palette.text()),
        Line::styled("   ├─ /        open command palette", palette.text()),
        Line::styled(
            "   ├─ Esc      stop task, close palette, or go back",
            palette.text(),
        ),
        Line::styled("   ├─ Ctrl+C   cancel task or clear input", palette.text()),
        Line::styled("   ├─ Ctrl+D   exit Routis", palette.text()),
        Line::styled("   └─ ?        close this help", palette.text()),
    ]
}

fn spinner(frame: u64) -> &'static str {
    const FRAMES: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    FRAMES[(frame as usize) % FRAMES.len()]
}
