use crate::tui::{
    screens::home::render_header,
    state::{AppState, SessionPhase},
    symbols,
    theme::ThemePalette,
    widgets::{
        input::render_input,
        palette::{command_palette_height, render_command_palette},
        timeline::render_timeline,
    },
    APP_VERSION,
};
use ratatui::{
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};
use unicode_width::UnicodeWidthStr;

const MIN_TERMINAL_WIDTH: u16 = 80;
const MIN_TERMINAL_HEIGHT: u16 = 24;

pub fn render_shell(frame: &mut Frame, area: Rect, state: &AppState, palette: ThemePalette) {
    frame.render_widget(Clear, area);
    if area.width < MIN_TERMINAL_WIDTH || area.height < MIN_TERMINAL_HEIGHT {
        render_too_small(frame, area, palette);
        return;
    }

    let shell = shell_area(area);
    let header_h = dashboard_height(shell);
    let palette_height = if state.ui.command_palette_open {
        command_palette_height(state, shell.height / 2).min(12)
    } else {
        0
    };
    let body_constraint = if shell.height <= 20 {
        Constraint::Length(0)
    } else {
        Constraint::Min(4)
    };
    let chunks = Layout::vertical([
        Constraint::Length(header_h),
        body_constraint,
        Constraint::Length(palette_height),
        Constraint::Length(input_block_height(state)),
    ])
    .split(shell);

    let bounds = frame.area();
    render_header_block(frame, safe_rect(chunks[0], bounds), state, palette);
    render_timeline(frame, safe_rect(chunks[1], bounds), state, palette);
    if state.ui.command_palette_open {
        render_command_palette(
            frame,
            safe_rect(
                Rect {
                    height: palette_height,
                    ..chunks[2]
                },
                bounds,
            ),
            state,
            palette,
        );
    }
    render_input_block(frame, safe_rect(chunks[3], bounds), state, palette);
}

fn render_header_block(frame: &mut Frame, area: Rect, state: &AppState, palette: ThemePalette) {
    if area.height < 3 {
        return;
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(palette.border_active())
        .title(Span::styled(
            format!(" Routis v{APP_VERSION} "),
            palette.section_title(),
        ))
        .title_alignment(Alignment::Center);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    render_header(frame, inner, state, palette);
}

fn shell_area(area: Rect) -> Rect {
    if area.width > MIN_TERMINAL_WIDTH + 2 && area.height > MIN_TERMINAL_HEIGHT + 2 {
        area.inner(Margin {
            horizontal: 1,
            vertical: 2,
        })
    } else {
        area
    }
}

fn dashboard_height(area: Rect) -> u16 {
    let desired = match area.width {
        144.. => 13.min(area.height.saturating_sub(8)),
        94..=143 => 10.min(area.height.saturating_sub(4)),
        _ => 11.min(area.height.saturating_sub(4)),
    };
    desired.max(8).min(area.height.saturating_sub(7).max(8))
}

fn input_block_height(state: &AppState) -> u16 {
    if state.session.phase == SessionPhase::AwaitingConfirmation && state.ui.input.is_empty() {
        5
    } else {
        4
    }
}

fn render_input_block(frame: &mut Frame, area: Rect, state: &AppState, palette: ThemePalette) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let input_height =
        if state.session.phase == SessionPhase::AwaitingConfirmation && state.ui.input.is_empty() {
            2
        } else {
            1
        };
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(input_height),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(area);

    render_top_rule(frame, chunks[0], state, palette);
    render_input(frame, chunks[1], state, palette);
    render_rule(frame, chunks[2], palette);
    render_runtime_line(frame, chunks[3], state, palette);
}

fn render_top_rule(frame: &mut Frame, area: Rect, state: &AppState, palette: ThemePalette) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let status = format!(" {} ", session_state_label(state.session.phase));
    let width = area.width as usize;
    let fill = width.saturating_sub(status.chars().count());

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(symbols::H.repeat(fill), palette.rail()),
            Span::styled(status, session_status_style(state.session.phase, palette)),
        ])),
        area,
    );
}

fn render_rule(frame: &mut Frame, area: Rect, palette: ThemePalette) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    frame.render_widget(
        Paragraph::new(Line::styled(
            symbols::H.repeat(area.width as usize),
            palette.rail(),
        )),
        area,
    );
}

fn render_runtime_line(frame: &mut Frame, area: Rect, state: &AppState, palette: ThemePalette) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let left = vec![
        Span::styled(
            provider_label(&state.config.provider).to_string(),
            palette.provider(&state.config.provider),
        ),
        separator(palette),
        Span::styled(state.current_plan.model.clone(), palette.text()),
        separator(palette),
        Span::styled(
            state.current_plan.reasoning.clone(),
            reasoning_style(&state.current_plan.reasoning, palette),
        ),
        separator(palette),
        Span::styled("branch ", palette.dim()),
        Span::styled(state.repo_context.branch.clone(), palette.muted()),
    ];
    let right_text = format!(
        "context {}% {} input {} tk",
        state.metrics.context_percent,
        symbols::SEP,
        state.metrics.input_tokens
    );
    let left_width = spans_width(&left);
    let right_width = UnicodeWidthStr::width(right_text.as_str());
    let available = area.width as usize;
    let gap = available
        .saturating_sub(left_width.saturating_add(right_width))
        .max(1);

    let mut spans = left;
    spans.push(Span::raw(" ".repeat(gap)));
    spans.push(Span::styled(right_text, palette.muted()));
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn separator(palette: ThemePalette) -> Span<'static> {
    Span::styled(format!(" {} ", symbols::SEP), palette.dim())
}

fn spans_width(spans: &[Span<'_>]) -> usize {
    spans
        .iter()
        .map(|span| UnicodeWidthStr::width(span.content.as_ref()))
        .sum()
}

fn reasoning_style(reasoning: &str, palette: ThemePalette) -> Style {
    match reasoning {
        "high" | "xhigh" => palette.warning().bold(),
        "low" => palette.muted(),
        _ => palette.text(),
    }
}

fn session_status_style(phase: SessionPhase, palette: ThemePalette) -> Style {
    match phase {
        SessionPhase::Idle => palette.muted(),
        SessionPhase::Running => palette.cyan().bold(),
        SessionPhase::AwaitingConfirmation => palette.warning().bold(),
        SessionPhase::Cancelled => palette.error().bold(),
        SessionPhase::Ready => palette.success().bold(),
    }
}

fn session_state_label(phase: SessionPhase) -> &'static str {
    match phase {
        SessionPhase::Idle => "idle",
        SessionPhase::Running => "planning",
        SessionPhase::AwaitingConfirmation => "waiting",
        SessionPhase::Cancelled => "stopped",
        SessionPhase::Ready => "ready",
    }
}

fn provider_label(value: &str) -> &str {
    match value {
        "Codex CLI" => "Codex",
        "Claude Code" => "Claude",
        other => other,
    }
}

fn safe_rect(area: Rect, bounds: Rect) -> Rect {
    let right = area.right().min(bounds.right());
    let bottom = area.bottom().min(bounds.bottom());
    if area.x >= right || area.y >= bottom {
        return Rect {
            x: bounds.x,
            y: bounds.y,
            width: 0,
            height: 0,
        };
    }
    Rect {
        x: area.x,
        y: area.y,
        width: right.saturating_sub(area.x),
        height: bottom.saturating_sub(area.y),
    }
}

fn render_too_small(frame: &mut Frame, area: Rect, palette: ThemePalette) {
    let lines = vec![
        Line::styled(format!("Routis v{APP_VERSION}"), palette.section_title()),
        Line::styled("Terminal too small.", palette.text()),
        Line::styled("Resize to at least 80x24.", palette.muted()),
    ];
    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), area);
}
