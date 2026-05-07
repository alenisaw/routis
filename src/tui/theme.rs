use ratatui::style::{Color, Style};

/// Full terminal color palette. Every field is set for every theme.
#[derive(Debug, Clone, Copy)]
pub struct ThemePalette {
    pub accent: Color,
    pub accent_soft: Color,
    pub selected_bg: Color,
    pub text: Color,
    pub muted: Color,
    pub dim: Color,
    pub surface: Color,
    pub surface_alt: Color,
    pub success: Color,
    pub cyan: Color,
    pub warning: Color,
    pub error: Color,
    pub provider_codex: Color,
    pub provider_claude: Color,
    pub provider_qwen: Color,
    pub rail: Color,
    pub rail_glow: Color,
}

impl ThemePalette {
    #[must_use]
    pub fn from_theme(theme: &str) -> Self {
        let mut p = Self {
            accent: Color::Rgb(82, 174, 188),
            accent_soft: Color::Rgb(54, 126, 138),
            selected_bg: Color::Rgb(16, 50, 57),
            text: Color::Rgb(220, 228, 231),
            muted: Color::Rgb(139, 154, 162),
            dim: Color::Rgb(70, 86, 94),
            surface: Color::Rgb(9, 13, 15),
            surface_alt: Color::Rgb(14, 22, 25),
            success: Color::Rgb(112, 196, 145),
            cyan: Color::Rgb(82, 174, 188),
            warning: Color::Rgb(202, 168, 88),
            error: Color::Rgb(210, 112, 112),
            provider_codex: Color::Rgb(232, 236, 238),
            provider_claude: Color::Rgb(204, 126, 82),
            provider_qwen: Color::Rgb(124, 138, 220),
            rail: Color::Rgb(48, 118, 130),
            rail_glow: Color::Rgb(82, 174, 188),
        };

        match theme {
            "Midnight Blue" => {
                p.accent = Color::Rgb(92, 145, 218);
                p.accent_soft = Color::Rgb(52, 92, 152);
                p.selected_bg = Color::Rgb(22, 48, 94);
                p.text = Color::Rgb(218, 226, 240);
                p.muted = Color::Rgb(136, 153, 184);
                p.dim = Color::Rgb(64, 78, 104);
                p.surface = Color::Rgb(8, 13, 23);
                p.surface_alt = Color::Rgb(13, 21, 36);
                p.cyan = Color::Rgb(104, 184, 214);
                p.success = Color::Rgb(108, 196, 152);
                p.warning = Color::Rgb(196, 163, 84);
                p.error = Color::Rgb(210, 108, 108);
                p.provider_qwen = Color::Rgb(132, 154, 230);
                p.rail = Color::Rgb(52, 105, 172);
                p.rail_glow = Color::Rgb(92, 145, 218);
            }
            "Routis Violet" => {
                p.accent = Color::Rgb(146, 122, 214);
                p.accent_soft = Color::Rgb(96, 72, 158);
                p.selected_bg = Color::Rgb(44, 32, 82);
                p.text = Color::Rgb(226, 220, 240);
                p.muted = Color::Rgb(150, 138, 182);
                p.dim = Color::Rgb(76, 66, 102);
                p.surface = Color::Rgb(10, 9, 17);
                p.surface_alt = Color::Rgb(17, 14, 29);
                p.cyan = Color::Rgb(96, 188, 202);
                p.success = Color::Rgb(116, 200, 152);
                p.warning = Color::Rgb(202, 166, 88);
                p.error = Color::Rgb(214, 116, 132);
                p.provider_qwen = Color::Rgb(138, 142, 226);
                p.rail = Color::Rgb(88, 72, 146);
                p.rail_glow = Color::Rgb(146, 122, 214);
            }
            "Neon Magenta" => {
                p.accent = Color::Rgb(210, 104, 158);
                p.accent_soft = Color::Rgb(156, 58, 106);
                p.selected_bg = Color::Rgb(82, 28, 52);
                p.text = Color::Rgb(238, 224, 230);
                p.muted = Color::Rgb(176, 142, 160);
                p.dim = Color::Rgb(92, 64, 78);
                p.surface = Color::Rgb(12, 8, 13);
                p.surface_alt = Color::Rgb(22, 13, 19);
                p.cyan = Color::Rgb(98, 184, 198);
                p.success = Color::Rgb(116, 196, 148);
                p.warning = Color::Rgb(204, 168, 90);
                p.error = Color::Rgb(222, 112, 112);
                p.provider_codex = Color::Rgb(238, 232, 236);
                p.provider_qwen = Color::Rgb(138, 146, 222);
                p.rail = Color::Rgb(142, 58, 100);
                p.rail_glow = Color::Rgb(210, 104, 158);
            }
            "Monochrome" => {
                p.accent = Color::Rgb(206, 210, 216);
                p.accent_soft = Color::Rgb(142, 148, 156);
                p.selected_bg = Color::Rgb(48, 50, 54);
                p.text = Color::Rgb(220, 222, 226);
                p.muted = Color::Rgb(146, 150, 158);
                p.dim = Color::Rgb(72, 76, 82);
                p.surface = Color::Rgb(10, 10, 10);
                p.surface_alt = Color::Rgb(18, 18, 18);
                p.cyan = Color::Rgb(190, 196, 204);
                p.success = Color::Rgb(190, 202, 196);
                p.warning = Color::Rgb(174, 168, 156);
                p.error = Color::Rgb(218, 210, 206);
                p.provider_codex = Color::Rgb(232, 234, 236);
                p.provider_claude = Color::Rgb(186, 174, 162);
                p.provider_qwen = Color::Rgb(184, 190, 204);
                p.rail = Color::Rgb(132, 136, 144);
                p.rail_glow = Color::Rgb(206, 210, 216);
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
    pub fn rail(self) -> Style {
        Style::default().fg(self.rail)
    }
    #[must_use]
    pub fn rail_glow(self) -> Style {
        Style::default().fg(self.rail_glow).bold()
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
    #[must_use]
    pub fn provider(self, source: &str) -> Style {
        let color = if source.contains("Claude") {
            self.provider_claude
        } else if source.contains("Qwen") {
            self.provider_qwen
        } else if source.contains("Codex") {
            self.provider_codex
        } else {
            self.accent
        };
        Style::default().fg(color)
    }
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
