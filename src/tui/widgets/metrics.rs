use crate::tui::{state::MetricsState, theme::ThemePalette};
use ratatui::style::Style;
use ratatui::text::{Line, Span};

pub fn metric_lines(
    metrics: &MetricsState,
    palette: ThemePalette,
    area_width: u16,
) -> Vec<Line<'static>> {
    metric_lines_with_total(metrics, palette, area_width, true)
}

pub fn metric_lines_compact(
    metrics: &MetricsState,
    palette: ThemePalette,
    area_width: u16,
) -> Vec<Line<'static>> {
    metric_lines_with_total(metrics, palette, area_width, false)
}

fn metric_lines_with_total(
    metrics: &MetricsState,
    palette: ThemePalette,
    area_width: u16,
    include_total: bool,
) -> Vec<Line<'static>> {
    let gauge_width = gauge_width(area_width);
    let mut lines = vec![
        metric_line(
            "input",
            metrics.input_tokens,
            4096,
            " tk",
            palette.metric_input(),
            palette,
            gauge_width,
        ),
        metric_line(
            "output",
            metrics.output_tokens,
            4096,
            " tk",
            palette.success(),
            palette,
            gauge_width,
        ),
    ];
    if include_total {
        lines.push(metric_line(
            "total",
            metrics.total_tokens,
            8192,
            " tk",
            palette.metric_total(),
            palette,
            gauge_width,
        ));
    }
    lines.push(metric_line(
        "saved",
        metrics.saved_percent,
        100,
        "%",
        palette.success(),
        palette,
        gauge_width,
    ));
    lines
}

fn metric_line(
    label: &'static str,
    value: usize,
    max: usize,
    suffix: &'static str,
    fill_style: Style,
    palette: ThemePalette,
    gauge_width: Option<usize>,
) -> Line<'static> {
    let value_str = format!("{value}{suffix}");
    let fill_style = if value * 100 / max.max(1) >= 90 {
        palette.error()
    } else if value * 100 / max.max(1) >= 70 && label == "context" {
        palette.warning()
    } else {
        fill_style
    };

    let mut spans = vec![Span::styled(format!("{label:<8}"), palette.muted())];
    if let Some(width) = gauge_width {
        let (filled, empty) = bar_parts(value, max, width);
        spans.push(Span::raw(" ["));
        spans.push(Span::styled(filled, fill_style));
        spans.push(Span::styled(empty, palette.gauge_empty()));
        spans.push(Span::raw("] "));
    } else {
        spans.push(Span::raw("  "));
    }
    spans.push(Span::styled(format!("{value_str:>6}"), palette.text()));
    Line::from(spans)
}

fn bar_parts(value: usize, max: usize, width: usize) -> (String, String) {
    let filled = (value.min(max) * width)
        .checked_div(max.max(1))
        .unwrap_or(0)
        .min(width);
    let empty = width.saturating_sub(filled);
    ("•".repeat(filled), "·".repeat(empty))
}

#[must_use]
pub fn gauge_width(area_width: u16) -> Option<usize> {
    match area_width {
        0..=27 => None,
        28..=35 => Some(6),
        36..=47 => Some(8),
        48..=63 => Some(10),
        _ => Some(14),
    }
}
