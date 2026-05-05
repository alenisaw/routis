use crate::tui::{
    screens::home::render_header,
    state::AppState,
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
    let input_height = 1;
    let chunks = Layout::vertical([
        Constraint::Length(header_h),
        Constraint::Length(1),
        body_constraint,
        Constraint::Length(palette_height),
        Constraint::Length(1),
        Constraint::Length(input_height),
    ])
    .split(shell);

    let bounds = frame.area();
    render_header_block(frame, safe_rect(chunks[0], bounds), state, palette);
    render_rule(frame, safe_rect(chunks[1], bounds), palette);
    render_timeline(frame, safe_rect(chunks[2], bounds), state, palette);
    if state.ui.command_palette_open {
        render_command_palette(
            frame,
            safe_rect(
                Rect {
                    height: palette_height,
                    ..chunks[3]
                },
                bounds,
            ),
            state,
            palette,
        );
    }
    render_rule(frame, safe_rect(chunks[4], bounds), palette);
    render_input(frame, safe_rect(chunks[5], bounds), state, palette);
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
