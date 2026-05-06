use crate::tui::{
    screens::home::render_header,
    state::{AppState, SessionPhase},
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
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

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
    let input_height = input_block_height(state);
    let chunks = Layout::vertical([
        Constraint::Length(header_h),
        body_constraint,
        Constraint::Length(palette_height),
        Constraint::Length(input_height),
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
        144.. => 19.min(area.height.saturating_sub(8)),
        94..=143 => 12.min(area.height.saturating_sub(4)),
        _ => 13.min(area.height.saturating_sub(4)),
    };
    desired.max(10).min(area.height.saturating_sub(7).max(10))
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

    render_rule(frame, chunks[0], palette);
    render_input(frame, chunks[1], state, palette);
    render_rule(frame, chunks[2], palette);
    render_runtime_line(frame, chunks[3], state, palette);
}

fn render_rule(frame: &mut Frame, area: Rect, palette: ThemePalette) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    frame.render_widget(
        Paragraph::new(Line::styled(
            "─".repeat(area.width as usize),
            palette.border(),
        )),
        area,
    );
}

fn render_runtime_line(frame: &mut Frame, area: Rect, state: &AppState, palette: ThemePalette) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let status = session_state_label(state.session.phase);
    let help = if state.session.phase == SessionPhase::AwaitingConfirmation {
        "↑↓ choose  Enter confirm  Esc decline  ? help"
    } else {
        "Enter send  / commands  ? help  Esc back"
    };
    let line = format!(
        "{}  {}  {}  profile {}  {}   {}",
        provider_label(&state.config.provider),
        state.current_plan.model,
        state.current_plan.reasoning,
        state.current_plan.profile,
        status,
        help
    );
    frame.render_widget(
        Paragraph::new(Line::styled(
            truncate_to_width(&line, area.width as usize),
            palette.muted(),
        )),
        area,
    );
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

fn truncate_to_width(value: &str, max: usize) -> String {
    use unicode_width::UnicodeWidthStr;

    if UnicodeWidthStr::width(value) <= max {
        return value.to_string();
    }
    let mut out = String::new();
    let mut width = 0;
    for ch in value.chars() {
        let ch_width = UnicodeWidthStr::width(ch.to_string().as_str());
        if width + ch_width + 3 > max {
            break;
        }
        out.push(ch);
        width += ch_width;
    }
    out.push_str("...");
    out
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
