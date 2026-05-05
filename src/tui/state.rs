#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Setup,
    Home,
    Session,
    Exit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupStep {
    Welcome,
    Name,
    Provider,
    Theme,
    Finish,
}

impl SetupStep {
    pub const ALL: [Self; 5] = [
        Self::Welcome,
        Self::Name,
        Self::Provider,
        Self::Theme,
        Self::Finish,
    ];

    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Welcome => "Welcome",
            Self::Name => "Name",
            Self::Provider => "Provider",
            Self::Theme => "Theme",
            Self::Finish => "Finish",
        }
    }

    #[must_use]
    pub fn index(self) -> usize {
        Self::ALL.iter().position(|s| *s == self).unwrap_or(0)
    }

    #[must_use]
    pub fn next(self) -> Self {
        Self::ALL
            .get(self.index() + 1)
            .copied()
            .unwrap_or(Self::Finish)
    }

    #[must_use]
    pub fn previous(self) -> Self {
        self.index()
            .checked_sub(1)
            .and_then(|i| Self::ALL.get(i).copied())
            .unwrap_or(Self::Welcome)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigState {
    pub display_name: String,
    pub provider: String,
    pub model: String,
    pub reasoning: String,
    pub theme: String,
}

impl Default for ConfigState {
    fn default() -> Self {
        Self {
            display_name: default_display_name(),
            provider: "Codex CLI".to_string(),
            model: "gpt-5.5".to_string(),
            reasoning: "medium".to_string(),
            theme: "Routis Cyan".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderDiagnostics {
    pub command: String,
    pub version: String,
    pub auth_status: String,
    pub config_path: String,
}

impl Default for ProviderDiagnostics {
    fn default() -> Self {
        Self {
            command: "Not checked".to_string(),
            version: "Not checked".to_string(),
            auth_status: "Not checked".to_string(),
            config_path: crate::tui::config::default_config_path()
                .display()
                .to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetupState {
    pub step: SetupStep,
    pub selected: usize,
    pub provider_index: usize,
    pub provider_checked: bool,
    pub model_index: usize,
    pub reasoning_index: usize,
    pub theme_index: usize,
}

impl Default for SetupState {
    fn default() -> Self {
        Self {
            step: SetupStep::Welcome,
            selected: 0,
            provider_index: 0,
            provider_checked: false,
            model_index: 0,
            reasoning_index: 0,
            theme_index: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    Minimal,
    Stacked,
    Compact,
    Wide,
}

impl LayoutMode {
    #[must_use]
    pub fn for_width(width: u16) -> Self {
        match width {
            0..=99 => Self::Minimal,
            100..=149 => Self::Compact,
            _ => Self::Wide,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionPhase {
    Idle,
    Running,
    AwaitingConfirmation,
    Cancelled,
    Ready,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteMode {
    Commands,
    Sessions,
    Themes,
    Providers,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionPickerItem {
    pub title: String,
    pub created: String,
    pub updated: String,
    pub branch: String,
    pub conversation: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionEvent {
    pub source: String,
    pub title: String,
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionState {
    pub title: String,
    pub current_task: String,
    pub events: Vec<SessionEvent>,
    pub scroll: usize,
    pub follow: bool,
    pub phase: SessionPhase,
    pub visible_lines: usize,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            title: "new-session".to_string(),
            current_task: String::new(),
            events: Vec::new(),
            scroll: 0,
            follow: true,
            phase: SessionPhase::Idle,
            visible_lines: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MetricsState {
    pub tasks: usize,
    pub context_percent: usize,
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub total_tokens: usize,
    pub saved_percent: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiState {
    pub input: String,
    pub status_line: String,
    pub layout_mode: LayoutMode,
    pub frame: u64,
    pub command_palette_open: bool,
    pub palette_mode: PaletteMode,
    pub command_palette_index: usize,
    pub session_picker_items: Vec<SessionPickerItem>,
    pub session_picker_all_items: Vec<SessionPickerItem>,
    pub session_picker_query: String,
    pub history_cursor: Option<usize>,
    pub shortcuts_open: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            input: String::new(),
            status_line: "Type a task or / for commands".to_string(),
            layout_mode: LayoutMode::Wide,
            frame: 0,
            command_palette_open: false,
            palette_mode: PaletteMode::Commands,
            command_palette_index: 0,
            session_picker_items: Vec::new(),
            session_picker_all_items: Vec::new(),
            session_picker_query: String::new(),
            history_cursor: None,
            shortcuts_open: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppState {
    pub mode: AppMode,
    pub setup: SetupState,
    pub config: ConfigState,
    pub provider_diagnostics: ProviderDiagnostics,
    pub session: SessionState,
    pub metrics: MetricsState,
    pub ui: UiState,
}

impl AppState {
    #[must_use]
    pub fn setup() -> Self {
        let mut state = Self {
            mode: AppMode::Setup,
            setup: SetupState::default(),
            config: ConfigState::default(),
            provider_diagnostics: detect_provider_diagnostics(),
            session: SessionState::default(),
            metrics: MetricsState::default(),
            ui: UiState::default(),
        };
        state.sync_setup_from_config();
        state
    }

    #[must_use]
    pub fn home() -> Self {
        let mut state = Self::setup();
        state.mode = AppMode::Home;
        state
    }

    #[must_use]
    pub fn with_config(config: ConfigState) -> Self {
        let mut state = Self::home();
        state.config = config;
        state.sync_setup_from_config();
        state
    }

    pub fn confirm(&mut self) {
        match self.mode {
            AppMode::Setup if self.setup.step == SetupStep::Finish => self.mode = AppMode::Home,
            AppMode::Setup if self.setup.step == SetupStep::Provider => {
                if !self.setup.provider_checked {
                    self.provider_diagnostics = detect_provider_diagnostics();
                    self.setup.provider_checked = true;
                    return;
                }
                if self.provider_diagnostics.command == "Found" {
                    self.config.provider = "Codex CLI".to_string();
                    self.setup.step = SetupStep::Theme;
                }
            }
            AppMode::Setup => self.setup.step = self.setup.step.next(),
            AppMode::Home | AppMode::Session => self.start_session_from_input(),
            AppMode::Exit => {}
        }
    }

    pub fn back(&mut self) {
        match self.mode {
            AppMode::Setup if self.setup.step == SetupStep::Welcome => self.mode = AppMode::Home,
            AppMode::Setup => self.setup.step = self.setup.step.previous(),
            AppMode::Session => self.mode = AppMode::Home,
            AppMode::Home => self.mode = AppMode::Exit,
            AppMode::Exit => {}
        }
    }

    pub fn open_setup(&mut self) {
        self.mode = AppMode::Setup;
        self.setup = SetupState::default();
        self.sync_setup_from_config();
    }

    pub fn start_session(&mut self, task: &str, title: String) {
        let task = task.trim();
        let mut events = std::mem::take(&mut self.session.events);
        self.mode = AppMode::Session;
        self.session.title = title;
        self.session.current_task = task.to_string();
        self.session.phase = SessionPhase::Running;
        let existing_lines = events
            .iter()
            .map(|event| 1 + event.lines.len())
            .sum::<usize>();
        self.session.visible_lines = existing_lines.saturating_add(3);
        self.session.scroll = 0;
        self.session.follow = true;
        self.metrics.tasks = self.metrics.tasks.saturating_add(1);
        self.metrics.input_tokens = estimate_input_tokens(task, &self.config);
        self.metrics.output_tokens = 0;
        self.metrics.total_tokens = self.metrics.input_tokens;
        self.metrics.context_percent = 18;
        self.metrics.saved_percent = 32;
        events.extend([
            SessionEvent {
                source: "You".to_string(),
                title: task.to_string(),
                lines: Vec::new(),
            },
            SessionEvent {
                source: "Routis".to_string(),
                title: "Preparing local execution plan".to_string(),
                lines: vec![
                    format!("Task: {task}"),
                    format!("Provider: {}", self.config.provider),
                    format!(
                        "Model: {}  reasoning: {}",
                        self.config.model, self.config.reasoning
                    ),
                    format!(
                        "Command preview: codex exec -m {} --reasoning {} -- \"{}\"",
                        self.config.model, self.config.reasoning, task
                    ),
                ],
            },
            SessionEvent {
                source: "Codex CLI".to_string(),
                title: "Ready to start when confirmed".to_string(),
                lines: vec![
                    "provider binary checked".to_string(),
                    "local config path resolved".to_string(),
                    "Awaiting confirmation".to_string(),
                ],
            },
        ]);
        self.session.events = events;
        self.session.visible_lines = self.session_total_render_lines();
        self.ui.input.clear();
        self.ui.command_palette_open = false;
        self.ui.shortcuts_open = false;
    }

    pub fn cancel_session(&mut self) {
        if self.mode != AppMode::Session {
            return;
        }
        self.session.phase = SessionPhase::Cancelled;
        self.session.visible_lines = self.session_total_render_lines();
        self.session.events.push(SessionEvent {
            source: "Routis".to_string(),
            title: "Session cancelled".to_string(),
            lines: vec!["No provider process is running now.".to_string()],
        });
        self.ui.status_line = "cancelled current session".to_string();
    }

    pub fn tick(&mut self) {
        self.ui.frame = self.ui.frame.wrapping_add(1);
        if self.mode != AppMode::Session || self.session.phase == SessionPhase::Idle {
            return;
        }
        if self.session.phase == SessionPhase::Cancelled {
            return;
        }
        let max_lines = self.session_total_render_lines();
        if self.session.visible_lines < max_lines {
            self.session.visible_lines = (self.session.visible_lines + 2).min(max_lines);
            self.session.phase = SessionPhase::Running;
        } else {
            self.session.phase = SessionPhase::AwaitingConfirmation;
        }
    }

    #[must_use]
    pub fn session_total_render_lines(&self) -> usize {
        self.session
            .events
            .iter()
            .map(|event| 1 + event.lines.len())
            .sum::<usize>()
            .saturating_add(self.session.events.len().saturating_sub(1))
    }

    fn start_session_from_input(&mut self) {
        let task = self.ui.input.trim().to_string();
        if task.is_empty() {
            return;
        }
        let title = crate::tui::session::make_session_title(&task);
        self.start_session(&task, title);
    }

    fn sync_setup_from_config(&mut self) {
        self.setup.provider_index = match self.config.provider.as_str() {
            "Codex CLI" => 0,
            "Claude Code" => 1,
            _ => 2,
        };
        self.setup.model_index = model_index(&self.config.model);
        self.setup.reasoning_index = reasoning_index(&self.config.reasoning);
        self.setup.theme_index = theme_index(&self.config.theme);
    }
}

// ── Public name/index helpers ─────────────────────────────────────────────

#[must_use]
pub fn model_name(index: usize) -> &'static str {
    match index {
        1 => "gpt-5.4",
        2 => "gpt-5.4-mini",
        3 => "custom",
        _ => "gpt-5.5",
    }
}

#[must_use]
pub fn reasoning_name(index: usize) -> &'static str {
    match index {
        1 => "high",
        2 => "xhigh",
        3 => "low",
        _ => "medium",
    }
}

/// 5 themes: 0 = Routis Cyan, 1 = Routis Violet, 2 = Neon Magenta,
///           3 = Midnight Blue, 4 = Monochrome.
#[must_use]
pub fn theme_name(index: usize) -> &'static str {
    match index {
        1 => "Routis Violet",
        2 => "Neon Magenta",
        3 => "Midnight Blue",
        4 => "Monochrome",
        _ => "Routis Cyan",
    }
}

/// Maximum valid theme index (inclusive).
pub const THEME_MAX: usize = 4;

fn model_index(value: &str) -> usize {
    (0..4).find(|i| model_name(*i) == value).unwrap_or(0)
}

fn reasoning_index(value: &str) -> usize {
    (0..4).find(|i| reasoning_name(*i) == value).unwrap_or(0)
}

fn theme_index(value: &str) -> usize {
    (0..=THEME_MAX)
        .find(|i| theme_name(*i) == value)
        .unwrap_or(0)
}

fn default_display_name() -> String {
    std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "Alen".to_string())
}

pub fn detect_provider_diagnostics() -> ProviderDiagnostics {
    let mut d = ProviderDiagnostics::default();
    let Some(command) = find_codex_command() else {
        d.command = "Missing".to_string();
        d.version = "Unavailable".to_string();
        d.auth_status = "Codex CLI executable was not found on PATH".to_string();
        return d;
    };

    match std::process::Command::new(&command)
        .arg("--version")
        .output()
    {
        Ok(out) => {
            d.command = "Found".to_string();
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            let ver = stdout
                .trim()
                .lines()
                .next()
                .or_else(|| stderr.trim().lines().next())
                .unwrap_or("")
                .to_string();
            d.version = if ver.is_empty() {
                "Version unavailable".to_string()
            } else {
                ver
            };
            d.auth_status = if out.status.success() {
                "Ready to use local Codex CLI".to_string()
            } else {
                "Codex CLI found; version check returned a non-zero status".to_string()
            };
        }
        Err(error) => {
            d.command = "Found".to_string();
            d.version = "Unavailable".to_string();
            d.auth_status = format!("Codex CLI found but could not be executed: {error}");
        }
    }
    d
}

#[must_use]
pub fn select_codex_candidate<I, S>(candidates: I) -> Option<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut fallback = None;
    for candidate in candidates {
        let candidate = candidate.as_ref().trim();
        if candidate.is_empty() {
            continue;
        }
        let lower = candidate.to_ascii_lowercase();
        if lower.ends_with(".ps1") {
            continue;
        }
        if lower.ends_with(".exe") || lower.ends_with(".cmd") || lower.ends_with(".bat") {
            return Some(candidate.to_string());
        }
        fallback.get_or_insert_with(|| candidate.to_string());
    }
    fallback
}

fn find_codex_command() -> Option<String> {
    #[cfg(windows)]
    {
        let output = std::process::Command::new("where.exe")
            .arg("codex")
            .output()
            .ok()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        select_codex_candidate(stdout.lines())
    }
    #[cfg(not(windows))]
    {
        Some("codex".to_string())
    }
}

fn estimate_input_tokens(task: &str, config: &ConfigState) -> usize {
    ((task.chars().count() + config.model.chars().count() + config.reasoning.chars().count()) / 4)
        .max(128)
}
