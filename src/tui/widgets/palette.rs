use crate::tui::{
    command::matching_commands,
    state::{theme_name, AppState, PaletteMode, THEME_MAX},
    theme::ThemePalette,
};
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph},
    Frame,
};

pub fn command_palette_height(state: &AppState, max_height: u16) -> u16 {
    let rows = match state.ui.palette_mode {
        PaletteMode::Commands => matching_commands(&state.ui.input).len().min(5) as u16,
        PaletteMode::Sessions => state.ui.session_picker_items.len() as u16 + 3,
        PaletteMode::Themes => THEME_MAX as u16 + 5,
        PaletteMode::Providers => 7,
    };
    rows.saturating_add(2).clamp(3, max_height.max(3))
}

pub fn render_command_palette(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    palette: ThemePalette,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    match state.ui.palette_mode {
        PaletteMode::Commands => render_commands(frame, area, state, palette),
        PaletteMode::Sessions => render_sessions(frame, area, state, palette),
        PaletteMode::Themes => render_themes(frame, area, state, palette),
        PaletteMode::Providers => render_providers(frame, area, state, palette),
    }
}

fn render_commands(frame: &mut Frame, area: Rect, state: &AppState, palette: ThemePalette) {
    let rows = matching_commands(&state.ui.input);
    let palette_height = command_palette_height(state, area.height);
    let area = Rect {
        height: palette_height,
        ..area
    };
    frame.render_widget(Clear, area);
    let visible = area.height.saturating_sub(2).min(5) as usize;
    let selected = state
        .ui
        .command_palette_index
        .min(rows.len().saturating_sub(1));
    let start = selected.saturating_sub(visible.saturating_sub(1));
    let lines = rows
        .iter()
        .skip(start)
        .take(visible)
        .enumerate()
        .map(|(visible_index, spec)| {
            let row_index = start + visible_index;
            let selected = row_index == state.ui.command_palette_index;
            let style = if selected {
                palette.selected()
            } else {
                palette.text()
            };
            let desc_width = (area.width as usize).saturating_sub(26).max(12);
            Line::from(vec![
                Span::styled(if selected { "› " } else { "  " }, style),
                Span::styled(
                    format!("{:<11}", spec.name),
                    if selected { style } else { palette.cyan() },
                ),
                Span::styled(format!("{:<desc_width$}", spec.description), style),
                Span::styled(
                    spec.shortcut,
                    if selected {
                        style.bold()
                    } else {
                        palette.accent()
                    },
                ),
            ])
        })
        .collect::<Vec<_>>();
    frame.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(palette.accent())
                .padding(Padding::horizontal(1)),
        ),
        area,
    );
}

fn render_sessions(frame: &mut Frame, area: Rect, state: &AppState, palette: ThemePalette) {
    let palette_height = command_palette_height(state, area.height);
    let area = Rect {
        height: palette_height,
        ..area
    };
    frame.render_widget(Clear, area);

    let mut lines = vec![
        Line::from(vec![
            Span::styled("Resume a previous session", palette.section_title()),
            Span::raw("  "),
            Span::styled("Sort: Updated", palette.accent()),
        ]),
        Line::from(vec![
            Span::styled("Search: ", palette.muted()),
            Span::styled(
                if state.ui.session_picker_query.is_empty() {
                    "type to filter"
                } else {
                    &state.ui.session_picker_query
                },
                if state.ui.session_picker_query.is_empty() {
                    palette.dim()
                } else {
                    palette.text()
                },
            ),
        ]),
        Line::from(vec![
            Span::styled(format!("{:<12}", "Created"), palette.text().bold()),
            Span::styled(format!("{:<12}", "Updated"), palette.text().bold()),
            Span::styled(format!("{:<8}", "Branch"), palette.text().bold()),
            Span::styled("Conversation", palette.text().bold()),
        ]),
    ];

    if state.ui.session_picker_items.is_empty() {
        lines.push(Line::styled("No local sessions yet", palette.muted()));
    } else {
        for (index, item) in state.ui.session_picker_items.iter().enumerate() {
            let selected = index == state.ui.command_palette_index;
            let style = if selected {
                palette.selected()
            } else {
                palette.text()
            };
            lines.push(Line::from(vec![
                Span::styled(if selected { "> " } else { "  " }, style),
                Span::styled(format!("{:<12}", item.created), style),
                Span::styled(format!("{:<12}", item.updated), style),
                Span::styled(format!("{:<8}", item.branch), style),
                Span::styled(&item.conversation, style),
            ]));
        }
    }

    frame.render_widget(panel(lines, palette), area);
}

fn render_themes(frame: &mut Frame, area: Rect, state: &AppState, palette: ThemePalette) {
    let palette_height = command_palette_height(state, area.height);
    let area = Rect {
        height: palette_height,
        ..area
    };
    frame.render_widget(Clear, area);
    let mut lines = vec![
        Line::styled("Choose theme", palette.section_title()),
        Line::styled("Live preview is applied immediately", palette.dim()),
    ];
    for index in 0..=THEME_MAX {
        let selected = index == state.ui.command_palette_index;
        let style = if selected {
            palette.selected()
        } else {
            palette.text()
        };
        lines.push(Line::from(vec![
            Span::styled(if selected { "> " } else { "  " }, style),
            Span::styled(format!("{}  {}", index + 1, theme_name(index)), style),
        ]));
    }
    lines.push(Line::from(vec![
        Span::styled("Current: ", palette.muted()),
        Span::styled(&state.config.theme, palette.accent().bold()),
    ]));

    frame.render_widget(panel(lines, palette), area);
}

fn render_providers(frame: &mut Frame, area: Rect, state: &AppState, palette: ThemePalette) {
    let palette_height = command_palette_height(state, area.height);
    let area = Rect {
        height: palette_height,
        ..area
    };
    frame.render_widget(Clear, area);
    let providers = [
        ("1  Codex CLI", "active provider"),
        ("2  Claude Code", "planned"),
        ("3  Custom OpenAI-compatible", "planned"),
    ];
    let mut lines = vec![
        Line::styled("Choose provider", palette.section_title()),
        Line::styled("Enter runs check for the selected provider", palette.dim()),
    ];
    for (index, (name, hint)) in providers.iter().enumerate() {
        let selected = index == state.ui.command_palette_index;
        let style = if selected {
            palette.selected()
        } else {
            palette.text()
        };
        lines.push(Line::from(vec![
            Span::styled(if selected { "> " } else { "  " }, style),
            Span::styled(format!("{name:<30}"), style),
            Span::styled(*hint, palette.dim()),
        ]));
    }
    lines.push(Line::from(vec![
        Span::styled("binary  ", palette.muted()),
        Span::styled(&state.provider_diagnostics.command, palette.text()),
        Span::raw("  "),
        Span::styled("version  ", palette.muted()),
        Span::styled(&state.provider_diagnostics.version, palette.text()),
    ]));
    frame.render_widget(panel(lines, palette), area);
}

fn panel(lines: Vec<Line<'_>>, palette: ThemePalette) -> Paragraph<'_> {
    Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(palette.accent())
            .padding(Padding::horizontal(1)),
    )
}
