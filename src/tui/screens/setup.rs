use crate::tui::{
    config::default_config_path,
    screens::home::{kv_line, sep_line, workspace_label},
    state::{AppState, SetupStep, THEME_MAX},
    symbols,
    theme::ThemePalette,
    widgets::{dividers::h_rule, mascot::mascot_render_lines},
    APP_VERSION,
};
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
    Frame,
};
use unicode_width::UnicodeWidthStr;

pub fn render_setup(frame: &mut Frame, area: Rect, state: &AppState, palette: ThemePalette) {
    let horizontal_margin = if area.width >= 100 { 3 } else { 1 };
    let area = area.inner(ratatui::layout::Margin {
        horizontal: horizontal_margin,
        vertical: 1,
    });

    let chunks = Layout::vertical([
        Constraint::Length(1),  // title bar
        Constraint::Length(2),  // step progress indicator
        Constraint::Length(1),  // thin gap
        Constraint::Length(14), // mascot + copy panel
        Constraint::Length(1),  // separator
        Constraint::Min(8),     // step choices
        Constraint::Length(1),  // footer
    ])
    .split(area);

    // ── Title ─────────────────────────────────────────────────────────────
    frame.render_widget(
        Paragraph::new(Line::from(vec![Span::styled(
            format!("Routis Setup v{APP_VERSION}"),
            palette.accent().bold(),
        )]))
        .alignment(Alignment::Left),
        chunks[0],
    );

    // ── Step progress indicator ───────────────────────────────────────────
    render_step_indicator(frame, chunks[1], state, palette);

    // ── Mascot + contextual copy ──────────────────────────────────────────
    let top = Layout::horizontal([Constraint::Percentage(36), Constraint::Percentage(64)])
        .spacing(2)
        .split(chunks[3]);
    render_setup_mascot(frame, top[0], palette);
    render_setup_copy(frame, top[1], state, palette);

    // ── Separator ─────────────────────────────────────────────────────────
    frame.render_widget(h_rule(chunks[4].width, palette), chunks[4]);

    // ── Interactive choices ───────────────────────────────────────────────
    render_setup_choices(frame, chunks[5], state, palette);

    // ── Footer keybindings ────────────────────────────────────────────────
    render_setup_footer(frame, chunks[6], palette);
}

// ── Step progress indicator ───────────────────────────────────────────────

fn render_step_indicator(frame: &mut Frame, area: Rect, state: &AppState, palette: ThemePalette) {
    let current = state.setup.step.index();
    let steps = SetupStep::ALL;
    let mut spans: Vec<Span<'_>> = Vec::new();

    for (i, step) in steps.iter().enumerate() {
        if i > 0 {
            let connector_style = if i <= current {
                palette.accent()
            } else {
                palette.dim()
            };
            spans.push(Span::styled(
                format!("  {}{}  ", symbols::H, symbols::H),
                connector_style,
            ));
        }
        let (dot, style) = if i < current {
            ("+", palette.success().bold())
        } else if i == current {
            ("*", palette.accent().bold())
        } else {
            ("o", palette.dim())
        };
        spans.push(Span::styled(format!("{dot} {}", step.label()), style));
    }

    // Second line: version info
    let info_line = Line::from(vec![
        Span::styled(format!("v{APP_VERSION}  "), palette.dim()),
        Span::styled(
            format!("step {} of {}", current + 1, steps.len()),
            palette.muted(),
        ),
    ]);

    let rows = vec![Line::from(spans), info_line];
    frame.render_widget(Paragraph::new(rows).alignment(Alignment::Left), area);
}

// ── Mascot panel ─────────────────────────────────────────────────────────

fn render_setup_mascot(frame: &mut Frame, area: Rect, palette: ThemePalette) {
    let mut lines = mascot_render_lines(palette)
        .into_iter()
        .map(|line| {
            let text = line
                .spans
                .first()
                .map(|span| span.content.to_string())
                .unwrap_or_default();
            Line::styled(center_display(text.trim(), area.width), palette.accent())
        })
        .collect::<Vec<_>>();
    lines.push(Line::raw(""));
    lines.push(Line::styled(
        center_display("Welcome to Routis!", area.width),
        palette.accent().bold(),
    ));
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), area);
}

fn center_display(value: &str, width: u16) -> String {
    let visible = UnicodeWidthStr::width(value);
    let width = width as usize;
    if visible >= width {
        return value.to_string();
    }
    format!("{}{}", " ".repeat((width - visible) / 2), value)
}

// ── Contextual copy panel ─────────────────────────────────────────────────

fn render_setup_copy(frame: &mut Frame, area: Rect, state: &AppState, palette: ThemePalette) {
    let (title, copy) = step_copy(state.setup.step);
    let current = state.setup.step;

    let mut lines = vec![
        Line::styled(title, palette.accent().bold()),
        Line::styled(copy[0], palette.text()),
        Line::styled(copy[1], palette.muted()),
        Line::raw(""),
    ];

    // Step list — tick done steps, highlight current, dim future
    for step in &SetupStep::ALL {
        lines.push(step_list_line(*step, current, palette));
    }

    lines.push(Line::raw(""));
    lines.push(Line::from(vec![
        Span::styled("workspace  ", palette.muted()),
        Span::styled(workspace_label(), palette.path()),
    ]));

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), area);
}

fn step_copy(step: SetupStep) -> (&'static str, [&'static str; 2]) {
    match step {
        SetupStep::Welcome => (
            "What this setup does",
            [
                "Routis routes AI coding tasks locally through your chosen CLI.",
                "This wizard writes only ~/.routis/config.toml.",
            ],
        ),
        SetupStep::Name => (
            "Your Name",
            [
                "Set the display name used in prompts and status lines.",
                "Stored locally on this machine; editable any time.",
            ],
        ),
        SetupStep::Provider => (
            "Provider",
            [
                "Check that the selected CLI is installed and visible here.",
                "Routis will use it to prepare local command previews.",
            ],
        ),
        SetupStep::Theme => (
            "Colour Theme",
            [
                "Choose a readable terminal palette with enough contrast.",
                "All themes are designed for dark backgrounds.",
            ],
        ),
        SetupStep::Finish => (
            "Review & Save",
            [
                "Review your settings before entering the shell.",
                "Press Enter to write config and start Routis.",
            ],
        ),
    }
}

fn step_list_line<'a>(step: SetupStep, current: SetupStep, palette: ThemePalette) -> Line<'a> {
    let idx = step.index();
    let cur = current.index();
    let (dot, label_style, value_style) = if idx < cur {
        ("+", palette.success(), palette.muted())
    } else if idx == cur {
        ("*", palette.accent().bold(), palette.text())
    } else {
        ("o", palette.dim(), palette.dim())
    };

    Line::from(vec![
        Span::styled(format!("{dot} {:<10}", step.label()), label_style),
        Span::styled(step_hint(step), value_style),
    ])
}

fn step_hint(step: SetupStep) -> &'static str {
    match step {
        SetupStep::Welcome => "start or import config",
        SetupStep::Name => "local display name",
        SetupStep::Provider => "Codex CLI / Claude Code",
        SetupStep::Theme => "5 palettes, live preview",
        SetupStep::Finish => "write config and launch",
    }
}

// ── Interactive choices ───────────────────────────────────────────────────

fn render_setup_choices(frame: &mut Frame, area: Rect, state: &AppState, palette: ThemePalette) {
    match state.setup.step {
        SetupStep::Welcome => frame.render_widget(
            Paragraph::new(vec![
                selectable(
                    "1  Start setup",
                    "configure name, provider and theme",
                    state.setup.selected == 0,
                    palette,
                ),
                selectable(
                    "2  Import config",
                    "load existing ~/.routis/config.toml",
                    state.setup.selected == 1,
                    palette,
                ),
                selectable(
                    "3  Exit",
                    "quit without changes",
                    state.setup.selected == 2,
                    palette,
                ),
            ]),
            area,
        ),

        SetupStep::Name => {
            let cursor_val = if state.config.display_name.is_empty() {
                "_".to_string()
            } else {
                format!("{}_", state.config.display_name)
            };
            frame.render_widget(
                Paragraph::new(vec![
                    Line::styled("What should Routis call you?", palette.accent().bold()),
                    Line::raw(""),
                    Line::styled(cursor_val, palette.text().bold()),
                    Line::styled("local only · 24 chars max", palette.dim()),
                ]),
                area,
            );
        }

        SetupStep::Provider => render_provider_choices(frame, area, state, palette),

        SetupStep::Theme => {
            let mut theme_lines = vec![Line::styled("Colour themes", palette.accent().bold())];
            for i in 0..=THEME_MAX {
                let name = crate::tui::state::theme_name(i);
                theme_lines.push(selectable(
                    label_for_theme(i, name),
                    "",
                    state.setup.theme_index == i,
                    palette,
                ));
            }
            theme_lines.push(Line::raw(""));
            theme_lines.push(Line::styled(
                format!("Preview: {}", state.config.theme),
                palette.accent().bold(),
            ));
            theme_lines.push(Line::from(vec![
                swatch("  text  ", palette.text, palette),
                Span::raw(" "),
                swatch("  muted  ", palette.muted, palette),
                Span::raw(" "),
                swatch("  accent  ", palette.accent, palette),
                Span::raw(" "),
                swatch("  success  ", palette.success, palette),
                Span::raw(" "),
                swatch("  warning  ", palette.warning, palette),
                Span::raw(" "),
                swatch("  error  ", palette.error, palette),
            ]));
            frame.render_widget(Paragraph::new(theme_lines), area);
        }

        SetupStep::Finish => frame.render_widget(
            Paragraph::new(vec![
                Line::styled("Settings to save", palette.accent().bold()),
                kv_line("name", &state.config.display_name, palette),
                kv_line("provider", &state.config.provider, palette),
                kv_line("theme", &state.config.theme, palette),
                kv_line(
                    "config",
                    default_config_path().display().to_string(),
                    palette,
                ),
                sep_line(palette),
                selectable(
                    "1  Start shell",
                    "enter Routis",
                    state.setup.selected == 0,
                    palette,
                ),
                selectable(
                    "2  Back to setup",
                    "adjust settings",
                    state.setup.selected == 1,
                    palette,
                ),
            ]),
            area,
        ),
    }
}

fn render_provider_choices(frame: &mut Frame, area: Rect, state: &AppState, palette: ThemePalette) {
    let rows = vec![
        provider_row(
            "1",
            "Codex CLI",
            provider_status(state),
            state.setup.provider_index == 0,
            palette,
            area.width,
        ),
        provider_row(
            "2",
            "Claude Code",
            "planned",
            state.setup.provider_index == 1,
            palette,
            area.width,
        ),
        provider_row(
            "3",
            "Custom OpenAI-compatible",
            "planned",
            state.setup.provider_index == 2,
            palette,
            area.width,
        ),
        Line::raw(""),
        Line::styled(
            "Enter checks selected provider; arrows switch rows",
            palette.muted(),
        ),
    ];
    frame.render_widget(Paragraph::new(rows), area);
}

fn provider_status(state: &AppState) -> String {
    if !state.setup.provider_checked {
        return "press Enter to check".to_string();
    }
    if state.provider_diagnostics.command == "Found" {
        format!("Found / {}", state.provider_diagnostics.version)
    } else {
        format!("Missing / {}", state.provider_diagnostics.auth_status)
    }
}

fn provider_row(
    index: &'static str,
    label: &'static str,
    status: impl ToString,
    selected: bool,
    palette: ThemePalette,
    width: u16,
) -> Line<'static> {
    let status = truncate_width(
        &status.to_string(),
        (width as usize).saturating_sub(34).max(10),
    );
    let left = format!("{index}  {label}");
    let gap = (width as usize)
        .saturating_sub(
            UnicodeWidthStr::width(left.as_str()) + UnicodeWidthStr::width(status.as_str()),
        )
        .max(2);
    Line::from(vec![
        Span::styled(
            if selected { "> " } else { "  " },
            if selected {
                palette.accent().bold()
            } else {
                palette.dim()
            },
        ),
        Span::styled(
            left,
            if selected {
                palette.text().bold()
            } else {
                palette.text()
            },
        ),
        Span::raw(" ".repeat(gap)),
        Span::styled(status, palette.dim()),
    ])
}

fn truncate_width(value: &str, max: usize) -> String {
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

// ── Footer ────────────────────────────────────────────────────────────────

fn render_setup_footer(frame: &mut Frame, area: Rect, palette: ThemePalette) {
    const KEYS: &[(&str, &str)] = &[
        ("[up/down]", "move"),
        ("[1-5]", "pick"),
        ("[left/right]", "step"),
        ("[Enter]", "confirm"),
        ("[Esc]", "back"),
    ];
    let spans: Vec<Span<'_>> = KEYS
        .iter()
        .flat_map(|(key, desc)| {
            [
                Span::styled(*key, palette.accent().bold()),
                Span::styled(format!(" {desc}  "), palette.muted()),
            ]
        })
        .collect();
    frame.render_widget(
        Paragraph::new(Line::from(spans)).alignment(Alignment::Center),
        area,
    );
}

// ── Widget helpers ────────────────────────────────────────────────────────

/// Selectable row: highlighted if `selected`, with an optional hint suffix.
fn selectable(
    label: &'static str,
    hint: &'static str,
    selected: bool,
    palette: ThemePalette,
) -> Line<'static> {
    if selected {
        Line::from(vec![
            Span::styled("› ", palette.accent().bold()),
            Span::styled(format!("{label:<32}"), palette.selected()),
            Span::styled(format!(" {hint}"), palette.dim()),
        ])
    } else {
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(label, palette.text()),
            Span::styled(
                if hint.is_empty() {
                    "".to_string()
                } else {
                    format!("  {hint}")
                },
                palette.dim(),
            ),
        ])
    }
}

fn swatch<'a>(label: &'static str, color: Color, palette: ThemePalette) -> Span<'a> {
    Span::styled(label, Style::default().fg(palette.surface).bg(color).bold())
}

fn label_for_theme(index: usize, name: &'static str) -> &'static str {
    match index {
        0 => "1  Routis Cyan",
        1 => "2  Routis Violet",
        2 => "3  Neon Magenta",
        3 => "4  Midnight Blue",
        4 => "5  Monochrome",
        _ => name,
    }
}
