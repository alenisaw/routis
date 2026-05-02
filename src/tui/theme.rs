use ratatui::style::{Color, Style};

/// Full terminal colour palette.  Every field is set for *every* theme —
/// we no longer silently inherit defaults for dim/surface/text.
#[derive(Debug, Clone, Copy)]
pub struct ThemePalette {
    pub accent: Color,
    pub accent_soft: Color,
    pub selected_bg: Color,
    pub text: Color,
    pub muted: Color,
    pub dim: Color,
    pub surface: Color,
    /// Slightly elevated surface — used for bordered panels, block backgrounds.
    pub surface_alt: Color,
    pub success: Color,
    pub cyan: Color,
    pub warning: Color,
    pub error: Color,
}

impl ThemePalette {
    #[must_use]
    pub fn from_theme(theme: &str) -> Self {
        // ── Routis Cyan (default) ────────────────────────────────────────
        let mut p = Self {
            accent: Color::Rgb(92, 200, 215),
            accent_soft: Color::Rgb(58, 166, 182),
            selected_bg: Color::Rgb(18, 58, 67),
            text: Color::Rgb(224, 234, 238),
            muted: Color::Rgb(148, 164, 172),
            dim: Color::Rgb(76, 94, 102),
            surface: Color::Rgb(10, 16, 19),
            surface_alt: Color::Rgb(16, 26, 30),
            success: Color::Rgb(117, 211, 154),
            cyan: Color::Rgb(92, 200, 215),
            warning: Color::Rgb(214, 179, 90),
            error: Color::Rgb(224, 122, 122),
        };

        match theme {
            "Midnight Blue" => {
                p.accent = Color::Rgb(96, 165, 250);
                p.accent_soft = Color::Rgb(37, 99, 235);
                p.selected_bg = Color::Rgb(30, 64, 175);
                p.text = Color::Rgb(219, 228, 245);
                p.muted = Color::Rgb(140, 160, 195);
                p.dim = Color::Rgb(66, 82, 112);
                p.surface = Color::Rgb(8, 14, 28);
                p.surface_alt = Color::Rgb(14, 22, 42);
                p.cyan = Color::Rgb(125, 211, 252);
                p.success = Color::Rgb(110, 210, 160);
                p.warning = Color::Rgb(208, 172, 80);
                p.error = Color::Rgb(220, 112, 112);
            }
            "Routis Violet" => {
                p.accent = Color::Rgb(167, 139, 250);
                p.accent_soft = Color::Rgb(124, 58, 237);
                p.selected_bg = Color::Rgb(59, 26, 120);
                p.text = Color::Rgb(228, 220, 248);
                p.muted = Color::Rgb(156, 140, 196);
                p.dim = Color::Rgb(82, 70, 112);
                p.surface = Color::Rgb(10, 8, 20);
                p.surface_alt = Color::Rgb(18, 14, 34);
                p.cyan = Color::Rgb(103, 232, 249);
                p.success = Color::Rgb(120, 220, 160);
                p.warning = Color::Rgb(212, 176, 88);
                p.error = Color::Rgb(228, 120, 140);
            }
            "Neon Magenta" => {
                p.accent = Color::Rgb(244, 114, 182);
                p.accent_soft = Color::Rgb(219, 39, 119);
                p.selected_bg = Color::Rgb(157, 23, 77);
                p.text = Color::Rgb(248, 224, 234);
                p.muted = Color::Rgb(196, 148, 172);
                p.dim = Color::Rgb(104, 68, 88);
                p.surface = Color::Rgb(12, 8, 14);
                p.surface_alt = Color::Rgb(22, 12, 20);
                p.cyan = Color::Rgb(103, 232, 249);
                p.success = Color::Rgb(120, 220, 155);
                p.warning = Color::Rgb(218, 182, 88);
                p.error = Color::Rgb(248, 112, 112);
            }
            "Monochrome" => {
                p.accent = Color::Rgb(229, 231, 235);
                p.accent_soft = Color::Rgb(156, 163, 175);
                p.selected_bg = Color::Rgb(58, 58, 58);
                p.text = Color::Rgb(220, 222, 226);
                p.muted = Color::Rgb(148, 152, 160);
                p.dim = Color::Rgb(76, 78, 82);
                p.surface = Color::Rgb(10, 10, 10);
                p.surface_alt = Color::Rgb(18, 18, 18);
                p.cyan = Color::Rgb(200, 204, 212);
                p.success = Color::Rgb(200, 206, 212);
                p.warning = Color::Rgb(172, 168, 164);
                p.error = Color::Rgb(232, 230, 228);
            }
            _ => {}
        }
        p
    }

    #[must_use]
    pub fn accent(self) -> Style {
        Style::default().fg(self.accent)
    }
    #[must_use]
    pub fn border(self) -> Style {
        Style::default().fg(self.accent_soft)
    }
    #[must_use]
    pub fn border_active(self) -> Style {
        Style::default().fg(self.accent)
    }
    #[must_use]
    pub fn section_title(self) -> Style {
        Style::default().fg(self.accent).bold()
    }
    #[must_use]
    pub fn text(self) -> Style {
        Style::default().fg(self.text)
    }
    #[must_use]
    pub fn muted(self) -> Style {
        Style::default().fg(self.muted)
    }
    #[must_use]
    pub fn dim(self) -> Style {
        Style::default().fg(self.dim)
    }
    #[must_use]
    pub fn success(self) -> Style {
        Style::default().fg(self.success)
    }
    #[must_use]
    pub fn cyan(self) -> Style {
        Style::default().fg(self.cyan)
    }
    #[must_use]
    pub fn warning(self) -> Style {
        Style::default().fg(self.warning)
    }
    #[must_use]
    pub fn error(self) -> Style {
        Style::default().fg(self.error)
    }
    #[must_use]
    pub fn path(self) -> Style {
        Style::default().fg(self.cyan).bold()
    }
    #[must_use]
    pub fn gauge_empty(self) -> Style {
        Style::default().fg(self.dim)
    }
    #[must_use]
    pub fn metric_input(self) -> Style {
        Style::default().fg(self.accent_soft)
    }
    #[must_use]
    pub fn metric_total(self) -> Style {
        Style::default().fg(self.muted)
    }

    #[must_use]
    pub fn selected(self) -> Style {
        Style::default().fg(Color::White).bg(self.selected_bg)
    }

    // ── Phase badge styles (filled bg, contrasting fg) ──────────────────

    #[must_use]
    pub fn badge_running(self) -> Style {
        Style::default().fg(self.surface).bg(self.cyan).bold()
    }
    #[must_use]
    pub fn badge_waiting(self) -> Style {
        Style::default().fg(self.surface).bg(self.warning).bold()
    }
    #[must_use]
    pub fn badge_cancelled(self) -> Style {
        Style::default().fg(self.surface).bg(self.error).bold()
    }
    #[must_use]
    pub fn badge_ready(self) -> Style {
        Style::default().fg(self.surface).bg(self.success).bold()
    }
    #[must_use]
    pub fn badge_idle(self) -> Style {
        Style::default().fg(self.surface).bg(self.muted).bold()
    }
}
