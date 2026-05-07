use crate::session_store::{default_session_store_path, SessionStore};
use crate::tui::{
    command::{complete_slash_command, matching_commands, parse_slash_command, SlashCommand},
    config::{default_config_path, load_config, save_config},
    history::{default_history_path, ShellHistory},
    render::render_app,
    state::{
        detect_provider_diagnostics, theme_name, AppMode, AppState, ConfirmationChoice,
        PaletteMode, SessionPhase, SessionPickerItem, SetupStep, THEME_MAX,
    },
};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io::Stdout, time::Duration};

pub async fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    let config_path = default_config_path();
    let mut state = match load_config(&config_path)? {
        Some(config) => AppState::with_config(config),
        None => AppState::setup(),
    };
    sync_repo_context(&mut state);
    let history_path = default_history_path();
    let mut history = ShellHistory::load(&history_path, 1000)?;

    loop {
        let width = terminal.size()?.width;
        state.ui.layout_mode = crate::tui::state::LayoutMode::for_width(width);
        terminal.draw(|frame| render_app(frame, &state))?;

        if std::env::var_os("ROUTIS_TUI_SMOKE_EXIT").is_some() {
            break;
        }
        if state.mode == AppMode::Exit {
            break;
        }

        if event::poll(Duration::from_millis(120))? {
            let Event::Key(key) = event::read()? else {
                continue;
            };
            if handle_key(&mut state, key, &mut history)? {
                history.save(&history_path)?;
            }
            if state.mode == AppMode::Home && state.setup.step == SetupStep::Finish {
                save_config(&config_path, &state.config)?;
            }
        } else {
            state.tick();
        }
    }

    history.save(&history_path)?;
    Ok(())
}

fn handle_key(state: &mut AppState, key: KeyEvent, history: &mut ShellHistory) -> Result<bool> {
    if key.kind != KeyEventKind::Press {
        return Ok(false);
    }

    match (key.modifiers, key.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('d')) => {
            state.mode = AppMode::Exit;
            return Ok(false);
        }
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
            handle_cancel(state);
            return Ok(false);
        }
        (KeyModifiers::CONTROL, KeyCode::Char('l')) => {
            clear_view(state);
            return Ok(false);
        }
        _ => {}
    }

    if key.code == KeyCode::Char('?') {
        state.ui.shortcuts_open = !state.ui.shortcuts_open;
        return Ok(false);
    }

    if state.mode == AppMode::Setup {
        handle_setup_key(state, key);
        return Ok(false);
    }

    if state.ui.command_palette_open {
        return Ok(handle_palette_key(state, key, history));
    }

    if state.mode == AppMode::Session && state.session.phase == SessionPhase::AwaitingConfirmation {
        return Ok(handle_confirmation_key(state, key));
    }

    match key.code {
        KeyCode::Esc => {
            if state.mode == AppMode::Session
                && matches!(
                    state.session.phase,
                    SessionPhase::Running | SessionPhase::AwaitingConfirmation
                )
            {
                state.cancel_session();
            } else if state.mode == AppMode::Session {
                state.mode = AppMode::Home;
            } else {
                state.ui.status_line = "use /quit to exit Routis".to_string();
            }
        }
        KeyCode::Enter => return submit_input(state, history),
        KeyCode::Tab => complete_input(state),
        KeyCode::Backspace => {
            state.ui.input.pop();
            state.ui.command_palette_open = state.ui.input.starts_with('/');
        }
        KeyCode::Char('/') if state.ui.input.is_empty() => {
            state.ui.input.push('/');
            state.ui.command_palette_open = true;
            state.ui.palette_mode = PaletteMode::Commands;
            state.ui.command_palette_index = 0;
        }
        KeyCode::Char(ch) => state.ui.input.push(ch),
        KeyCode::Up => {
            if state.session.events.is_empty() && !state.ui.shortcuts_open {
                state.session.scroll = 0;
            } else {
                state.session.follow = false;
                state.session.scroll = state.session.scroll.saturating_sub(1);
            }
        }
        KeyCode::Down => {
            if state.session.events.is_empty() && !state.ui.shortcuts_open {
                state.session.scroll = 0;
            } else {
                state.session.scroll = state.session.scroll.saturating_add(1);
                state.session.follow = true;
            }
        }
        _ => {}
    }
    Ok(false)
}

fn handle_confirmation_key(state: &mut AppState, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => state.cancel_session(),
        KeyCode::Up => state.ui.confirmation_index = state.ui.confirmation_index.saturating_sub(1),
        KeyCode::Down => {
            state.ui.confirmation_index =
                (state.ui.confirmation_index + 1).min(ConfirmationChoice::ALL.len() - 1);
        }
        KeyCode::Char('1') => state.ui.confirmation_index = 0,
        KeyCode::Char('2') => state.ui.confirmation_index = 1,
        KeyCode::Char('p') | KeyCode::Char('P') => confirm_provider_execution(state),
        KeyCode::Char('c') | KeyCode::Char('C') => state.cancel_session(),
        KeyCode::Enter => match ConfirmationChoice::ALL[state.ui.confirmation_index] {
            ConfirmationChoice::Proceed => confirm_provider_execution(state),
            ConfirmationChoice::Decline => state.cancel_session(),
        },
        _ => {}
    }
    false
}

pub fn handle_key_for_test(state: &mut AppState, key: KeyEvent) {
    let mut history = ShellHistory::new(1000);
    let _ = handle_key(state, key, &mut history);
}

pub fn handle_key_with_history_for_test(
    state: &mut AppState,
    history: &mut ShellHistory,
    key: KeyEvent,
) {
    let _ = handle_key(state, key, history);
}

fn handle_cancel(state: &mut AppState) {
    if state.ui.command_palette_open {
        state.ui.command_palette_open = false;
        state.ui.status_line = "command palette closed".to_string();
        return;
    }
    if !state.ui.input.is_empty() {
        state.ui.input.clear();
        state.ui.status_line = "input cleared".to_string();
        return;
    }
    if state.mode == AppMode::Session
        && matches!(
            state.session.phase,
            SessionPhase::Running | SessionPhase::AwaitingConfirmation | SessionPhase::Ready
        )
    {
        state.cancel_session();
        return;
    }
    state.mode = AppMode::Exit;
}

fn handle_palette_key(state: &mut AppState, key: KeyEvent, history: &mut ShellHistory) -> bool {
    match state.ui.palette_mode {
        PaletteMode::Commands => handle_command_palette_key(state, key, history),
        PaletteMode::Sessions => {
            handle_session_picker_key(state, key);
            false
        }
        PaletteMode::Themes => {
            handle_theme_picker_key(state, key);
            false
        }
        PaletteMode::Providers => {
            handle_provider_picker_key(state, key);
            false
        }
    }
}

fn handle_command_palette_key(
    state: &mut AppState,
    key: KeyEvent,
    history: &mut ShellHistory,
) -> bool {
    match key.code {
        KeyCode::Esc => {
            close_palette(state, "command palette closed");
            false
        }
        KeyCode::Up => {
            state.ui.command_palette_index = state.ui.command_palette_index.saturating_sub(1);
            false
        }
        KeyCode::Down => {
            let max = matching_commands(&state.ui.input).len().saturating_sub(1);
            state.ui.command_palette_index = (state.ui.command_palette_index + 1).min(max);
            false
        }
        KeyCode::Enter => apply_palette_selection(state, history),
        KeyCode::Tab => {
            complete_input(state);
            false
        }
        KeyCode::Backspace => {
            state.ui.input.pop();
            state.ui.command_palette_open = state.ui.input.starts_with('/');
            if !state.ui.command_palette_open {
                state.ui.palette_mode = PaletteMode::Commands;
            }
            state.ui.command_palette_index = 0;
            false
        }
        KeyCode::Char(ch) => {
            state.ui.input.push(ch);
            state.ui.command_palette_index = 0;
            false
        }
        _ => false,
    }
}

fn handle_session_picker_key(state: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => close_palette(state, "sessions closed"),
        KeyCode::Backspace => {
            state.ui.session_picker_query.pop();
            refresh_session_picker_filter(state);
        }
        KeyCode::Char(ch) if !ch.is_control() => {
            state.ui.session_picker_query.push(ch);
            refresh_session_picker_filter(state);
        }
        KeyCode::Up => {
            state.ui.command_palette_index = state.ui.command_palette_index.saturating_sub(1)
        }
        KeyCode::Down => {
            let max = state.ui.session_picker_items.len().saturating_sub(1);
            state.ui.command_palette_index = (state.ui.command_palette_index + 1).min(max);
        }
        KeyCode::Enter => {
            let Some(item) = state
                .ui
                .session_picker_items
                .get(state.ui.command_palette_index)
                .cloned()
            else {
                close_palette(state, "no sessions to resume");
                return;
            };
            state.ui.input = item.conversation.clone();
            close_palette(state, "session selected");
            state.start_session(&item.conversation, item.title);
        }
        _ => {}
    }
}

fn handle_theme_picker_key(state: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => close_palette(state, "theme picker closed"),
        KeyCode::Up => {
            state.ui.command_palette_index = state.ui.command_palette_index.saturating_sub(1);
            state.config.theme = theme_name(state.ui.command_palette_index).to_string();
        }
        KeyCode::Down => {
            state.ui.command_palette_index = (state.ui.command_palette_index + 1).min(THEME_MAX);
            state.config.theme = theme_name(state.ui.command_palette_index).to_string();
        }
        KeyCode::Char('1'..='5') => {
            if let KeyCode::Char(ch) = key.code {
                if let Some(index) = ch.to_digit(10).and_then(|v| v.checked_sub(1)) {
                    let index = (index as usize).min(THEME_MAX);
                    state.ui.command_palette_index = index;
                    state.config.theme = theme_name(index).to_string();
                }
            }
        }
        KeyCode::Enter => close_palette(state, "theme selected"),
        _ => {}
    }
}

fn handle_provider_picker_key(state: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => close_palette(state, "provider picker closed"),
        KeyCode::Up => {
            state.ui.command_palette_index = state.ui.command_palette_index.saturating_sub(1)
        }
        KeyCode::Down => {
            state.ui.command_palette_index = (state.ui.command_palette_index + 1).min(2)
        }
        KeyCode::Char('1'..='3') => {
            if let KeyCode::Char(ch) = key.code {
                if let Some(index) = ch.to_digit(10).and_then(|v| v.checked_sub(1)) {
                    state.ui.command_palette_index = (index as usize).min(2);
                }
            }
        }
        KeyCode::Enter => {
            if state.ui.command_palette_index == 0 {
                if state.provider_diagnostics.command != "Found" {
                    state.provider_diagnostics = detect_provider_diagnostics();
                }
                state.config.provider = "Codex CLI".to_string();
                if state.provider_diagnostics.command == "Found" {
                    close_palette(state, "Codex CLI Found");
                    push_command_event(
                        state,
                        "Command result",
                        vec![
                            "provider: Codex CLI".to_string(),
                            format!("binary: {}", state.provider_diagnostics.command),
                            format!("version: {}", state.provider_diagnostics.version),
                        ],
                    );
                } else {
                    state.ui.status_line =
                        "provider check failed: Codex CLI unavailable".to_string();
                }
            } else {
                state.ui.status_line = "provider is planned; Codex CLI remains active".to_string();
            }
        }
        _ => {}
    }
}

fn handle_setup_key(state: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Esc if state.setup.step != SetupStep::Welcome => state.back(),
        KeyCode::Esc => {
            state.ui.status_line = "use /quit to exit Routis".to_string();
        }
        KeyCode::Enter => confirm_setup(state),
        KeyCode::Right if state.setup.step == SetupStep::Provider => {
            confirm_setup(state);
        }
        KeyCode::Right => state.setup.step = state.setup.step.next(),
        KeyCode::Left => state.setup.step = state.setup.step.previous(),
        KeyCode::Up => setup_move_up(state),
        KeyCode::Down => setup_move_down(state),
        KeyCode::Char(' ') => setup_move_down(state),
        KeyCode::Char('1'..='5') => choose_setup_number(state, key.code),
        KeyCode::Backspace if state.setup.step == SetupStep::Name => {
            state.config.display_name.pop();
        }
        KeyCode::Char(ch) if state.setup.step == SetupStep::Name => {
            push_display_name_char(state, ch);
        }
        _ => {}
    }
}

fn confirm_setup(state: &mut AppState) {
    match state.setup.step {
        SetupStep::Welcome => match state.setup.selected {
            0 => state.setup.step = SetupStep::Name,
            1 => state.ui.status_line = "import config is not available yet".to_string(),
            _ => state.mode = AppMode::Exit,
        },
        SetupStep::Name => {
            if state.config.display_name.trim().is_empty() {
                state.config.display_name = "Alen".to_string();
            }
            state.setup.step = SetupStep::Provider;
        }
        SetupStep::Provider => {
            if !state.setup.provider_checked {
                state.provider_diagnostics = detect_provider_diagnostics();
                state.setup.provider_checked = true;
                state.ui.status_line = if state.provider_diagnostics.command == "Found" {
                    "provider check passed; press Enter again to continue".to_string()
                } else {
                    "provider check failed; install Codex CLI or choose another provider"
                        .to_string()
                };
                return;
            }
            if state.setup.provider_index == 0 {
                if state.provider_diagnostics.command == "Found" {
                    state.config.provider = "Codex CLI".to_string();
                    state.setup.step = SetupStep::Theme;
                } else {
                    state.provider_diagnostics = detect_provider_diagnostics();
                    state.ui.status_line = if state.provider_diagnostics.command == "Found" {
                        "provider check passed; press Enter again to continue".to_string()
                    } else {
                        "provider check failed; Codex CLI was not found on PATH".to_string()
                    };
                }
            } else {
                state.ui.status_line = "provider is planned, Codex CLI remains active".to_string();
            }
        }
        SetupStep::Theme => {
            state.config.theme = theme_name(state.setup.theme_index).to_string();
            state.setup.step = SetupStep::Finish;
        }
        SetupStep::Finish => {
            state.mode = AppMode::Home;
        }
    }
}

fn setup_move_up(state: &mut AppState) {
    match state.setup.step {
        SetupStep::Welcome | SetupStep::Finish => {
            state.setup.selected = state.setup.selected.saturating_sub(1);
        }
        SetupStep::Name => {}
        SetupStep::Provider => {
            state.setup.provider_index = state.setup.provider_index.saturating_sub(1);
            state.setup.provider_checked = false;
        }
        SetupStep::Theme => {
            state.setup.theme_index = state.setup.theme_index.saturating_sub(1);
            state.config.theme = theme_name(state.setup.theme_index).to_string();
        }
    }
}

fn setup_move_down(state: &mut AppState) {
    match state.setup.step {
        SetupStep::Welcome => state.setup.selected = (state.setup.selected + 1).min(2),
        SetupStep::Finish => state.setup.selected = (state.setup.selected + 1).min(1),
        SetupStep::Name => {}
        SetupStep::Provider => {
            state.setup.provider_index = (state.setup.provider_index + 1).min(2);
            state.setup.provider_checked = false;
        }
        SetupStep::Theme => {
            state.setup.theme_index =
                (state.setup.theme_index + 1).min(crate::tui::state::THEME_MAX);
            state.config.theme = theme_name(state.setup.theme_index).to_string();
        }
    }
}

fn choose_setup_number(state: &mut AppState, code: KeyCode) {
    let KeyCode::Char(ch) = code else {
        return;
    };
    let Some(index) = ch.to_digit(10).and_then(|value| value.checked_sub(1)) else {
        return;
    };
    let index = index as usize;

    match state.setup.step {
        SetupStep::Welcome if index <= 2 => state.setup.selected = index,
        SetupStep::Name => {}
        SetupStep::Provider if index <= 2 => {
            state.setup.provider_index = index;
            state.setup.provider_checked = false;
        }
        SetupStep::Theme if index <= crate::tui::state::THEME_MAX => {
            state.setup.theme_index = index;
            state.config.theme = theme_name(index).to_string();
        }
        SetupStep::Finish if index <= 1 => state.setup.selected = index,
        _ => {}
    }
}

fn push_display_name_char(state: &mut AppState, ch: char) {
    if ch.is_control() || state.config.display_name.chars().count() >= 24 {
        return;
    }
    state.config.display_name.push(ch);
}

fn submit_input(state: &mut AppState, history: &mut ShellHistory) -> Result<bool> {
    let input = state.ui.input.trim().to_string();
    if input.is_empty() {
        return Ok(false);
    }
    if state.mode == AppMode::Session && handle_session_confirmation(state, &input) {
        return Ok(false);
    }
    history.push(&input);
    if input.starts_with('/') {
        apply_command(state, parse_slash_command(&input), history);
    } else {
        state.confirm();
    }
    Ok(true)
}

fn apply_command(
    state: &mut AppState,
    command: Result<SlashCommand, String>,
    history: &ShellHistory,
) {
    let mut record_result = true;
    match command {
        Ok(SlashCommand::Help) => {
            state.ui.shortcuts_open = true;
            state.ui.status_line = "shortcuts opened".to_string();
        }
        Ok(SlashCommand::Status) => {
            state.ui.status_line = format!(
                "{} | {} | reasoning {} | theme {}",
                state.config.provider,
                state.config.model,
                state.config.reasoning,
                state.config.theme
            );
            push_command_event(
                state,
                "Command result",
                vec![
                    format!("provider: {}", state.config.provider),
                    format!("model: {}", state.config.model),
                    format!("reasoning: {}", state.config.reasoning),
                    format!("theme: {}", state.config.theme),
                    format!("policy file: {}", state.config.policy_file),
                    format!("branch: {}", state.repo_context.branch),
                    format!("changed files: {}", state.repo_context.changed_files),
                    format!("area: {}", state.repo_context.impact_area),
                    format!("mode: {:?}", state.mode),
                ],
            );
            record_result = false;
        }
        Ok(SlashCommand::Setup) => {
            state.ui.status_line = "opening setup wizard".to_string();
            push_status_event(state);
            record_result = false;
            state.open_setup();
        }
        Ok(SlashCommand::Config) => {
            state.ui.status_line = format!("config: {}", default_config_path().display());
            push_command_event(
                state,
                "Command result",
                vec![
                    format!("config: {}", default_config_path().display()),
                    format!("provider: {}", state.config.provider),
                    format!("model: {}", state.config.model),
                    format!("reasoning: {}", state.config.reasoning),
                    format!("theme: {}", state.config.theme),
                ],
            );
            record_result = false;
        }
        Ok(SlashCommand::Provider) => {
            state.ui.status_line = "provider picker opened".to_string();
            push_status_event(state);
            open_provider_picker(state);
            record_result = false;
        }
        Ok(SlashCommand::Theme) => {
            state.ui.status_line = "theme picker opened".to_string();
            push_status_event(state);
            open_theme_picker(state);
            record_result = false;
        }
        Ok(SlashCommand::Doctor) => {
            let config_path = default_config_path();
            let config_status = if config_path.exists() {
                "found"
            } else {
                "missing"
            };
            state.ui.status_line = format!(
                "doctor: codex {} | version {} | auth {} | config {} ({})",
                state.provider_diagnostics.command,
                state.provider_diagnostics.version,
                state.provider_diagnostics.auth_status,
                config_status,
                config_path.display()
            );
            push_command_event(
                state,
                "Command result",
                vec![
                    format!("provider: {}", state.config.provider),
                    format!("binary: {}", state.provider_diagnostics.command),
                    format!("version: {}", state.provider_diagnostics.version),
                    format!("auth: {}", state.provider_diagnostics.auth_status),
                    format!("config: {} ({config_status})", config_path.display()),
                ],
            );
            record_result = false;
        }
        Ok(SlashCommand::Clear) => {
            clear_view(state);
        }
        Ok(SlashCommand::Context) => {
            let lines = match crate::route_plan::collect_repo_context_for_cwd(
                std::env::current_dir().unwrap_or_default(),
            ) {
                Ok(context) => {
                    state.repo_context = repo_context_state_from_context(&context);
                    let impact_area = crate::route_plan::format_impact_area(&context);
                    let mut lines = vec![
                        format!("branch: {}", context.branch.as_deref().unwrap_or("-")),
                        format!("changed files: {}", context.changed_files.len()),
                        format!("area: {impact_area}"),
                    ];
                    lines.extend(
                        context
                            .changed_files
                            .iter()
                            .take(5)
                            .map(|path| format!("file: {}", path.display())),
                    );
                    lines
                }
                Err(error) => vec![format!("context unavailable: {error}")],
            };
            state.ui.status_line = "repository context loaded".to_string();
            push_command_event(state, "Command result", lines);
            record_result = false;
        }
        Ok(SlashCommand::PolicyFile) => {
            let path = state
                .ui
                .input
                .split_whitespace()
                .nth(1)
                .unwrap_or("configs/policies/default.yaml")
                .to_string();
            let repo_root = std::env::current_dir()
                .ok()
                .and_then(|cwd| crate::route_plan::discover_repo_root(&cwd));
            match crate::route_plan::load_policy(&path, repo_root.as_deref()) {
                Ok(_) => {
                    state.config.policy_file = path.clone();
                    state.ui.status_line = format!("policy file set: {path}");
                    push_command_event(
                        state,
                        "Command result",
                        vec![format!("policy file: {path}")],
                    );
                }
                Err(error) => {
                    state.ui.status_line = format!("policy file rejected: {error}");
                    push_command_event(
                        state,
                        "Command result",
                        vec![
                            format!("policy file rejected: {path}"),
                            format!("error: {error}"),
                        ],
                    );
                }
            }
            record_result = false;
        }
        Ok(SlashCommand::History) => {
            state.ui.status_line = "history: recent prompts are stored locally".to_string();
            push_command_event(state, "Command result", history_lines(history));
            record_result = false;
        }
        Ok(SlashCommand::Sessions) => {
            state.ui.status_line = "sessions opened".to_string();
            push_status_event(state);
            let store = SessionStore::new(default_session_store_path());
            open_session_picker(state, history, &store);
            record_result = false;
        }
        Ok(SlashCommand::Quit) => {
            state.ui.status_line = "exiting Routis".to_string();
            push_status_event(state);
            record_result = false;
            state.mode = AppMode::Exit;
        }
        Err(message) => state.ui.status_line = message,
    }
    if record_result && !state.ui.status_line.is_empty() {
        push_status_event(state);
    }
    state.ui.input.clear();
    if !state.ui.command_palette_open {
        state.ui.input.clear();
        state.ui.command_palette_index = 0;
    }
    state.ui.history_cursor = None;
}

fn sync_repo_context(state: &mut AppState) {
    if let Ok(context) =
        crate::route_plan::collect_repo_context_for_cwd(std::env::current_dir().unwrap_or_default())
    {
        state.repo_context = repo_context_state_from_context(&context);
    }
}

fn repo_context_state_from_context(
    context: &routis_context::RepoContext,
) -> crate::tui::state::RepoContextState {
    crate::tui::state::RepoContextState {
        branch: context.branch.clone().unwrap_or_else(|| "-".to_string()),
        changed_files: context.changed_files.len(),
        impact_area: crate::route_plan::format_impact_area(context),
    }
}

fn close_palette(state: &mut AppState, status: &str) {
    state.ui.command_palette_open = false;
    state.ui.palette_mode = PaletteMode::Commands;
    state.ui.command_palette_index = 0;
    state.ui.session_picker_items.clear();
    state.ui.session_picker_all_items.clear();
    state.ui.session_picker_query.clear();
    state.ui.input.clear();
    state.ui.status_line = status.to_string();
}

fn open_session_picker(state: &mut AppState, history: &ShellHistory, store: &SessionStore) {
    state.ui.session_picker_all_items = session_store_items(store).unwrap_or_default();
    if state.ui.session_picker_all_items.is_empty() {
        state.ui.session_picker_all_items = history
            .recent_detailed(12)
            .into_iter()
            .map(|item| SessionPickerItem {
                conversation: item.conversation,
                title: item.title,
                created: item.created,
                updated: item.updated,
                branch: item.branch,
            })
            .collect();
    }
    state.ui.session_picker_query.clear();
    refresh_session_picker_filter(state);
    state.ui.command_palette_open = true;
    state.ui.palette_mode = PaletteMode::Sessions;
    state.ui.command_palette_index = 0;
    state.ui.input.clear();
    state.ui.status_line = "sessions opened".to_string();
}

fn session_store_items(store: &SessionStore) -> Result<Vec<SessionPickerItem>> {
    Ok(store
        .list()?
        .into_iter()
        .take(12)
        .map(|session| SessionPickerItem {
            conversation: session.task,
            title: session.title,
            created: session.created_at.to_string(),
            updated: session.updated_at.to_string(),
            branch: session.branch,
        })
        .collect())
}

pub fn open_session_picker_for_test(
    state: &mut AppState,
    history: &ShellHistory,
    store: &SessionStore,
) {
    open_session_picker(state, history, store);
}

fn refresh_session_picker_filter(state: &mut AppState) {
    let query = state.ui.session_picker_query.trim().to_ascii_lowercase();
    state.ui.session_picker_items = state
        .ui
        .session_picker_all_items
        .iter()
        .filter(|item| {
            query.is_empty()
                || item.conversation.to_ascii_lowercase().contains(&query)
                || item.title.to_ascii_lowercase().contains(&query)
        })
        .cloned()
        .collect();
    let max = state.ui.session_picker_items.len().saturating_sub(1);
    state.ui.command_palette_index = state.ui.command_palette_index.min(max);
}

fn open_theme_picker(state: &mut AppState) {
    state.ui.command_palette_open = true;
    state.ui.palette_mode = PaletteMode::Themes;
    state.ui.command_palette_index = state.setup.theme_index.min(THEME_MAX);
    state.ui.input.clear();
    state.ui.status_line = "theme picker opened".to_string();
}

fn open_provider_picker(state: &mut AppState) {
    state.ui.command_palette_open = true;
    state.ui.palette_mode = PaletteMode::Providers;
    state.ui.command_palette_index = state.setup.provider_index.min(2);
    state.ui.input.clear();
    state.ui.status_line = "provider picker opened".to_string();
}

fn history_lines(history: &ShellHistory) -> Vec<String> {
    let mut lines = history
        .entries()
        .iter()
        .rev()
        .filter(|entry| !entry.trim_start().starts_with('/'))
        .take(5)
        .enumerate()
        .map(|(index, entry)| format!("{}: {}", index + 1, entry))
        .collect::<Vec<_>>();
    if lines.is_empty() {
        lines.push("no local prompt history yet".to_string());
    }
    lines.insert(0, format!("recent prompts: {}", lines.len()));
    lines
}

fn push_status_event(state: &mut AppState) {
    push_command_event(state, "Command result", vec![state.ui.status_line.clone()]);
}

fn push_command_event(state: &mut AppState, title: &str, lines: Vec<String>) {
    state.session.events.push(crate::tui::state::SessionEvent {
        source: "Routis".to_string(),
        title: title.to_string(),
        lines,
    });
    state.session.visible_lines = state.session_total_render_lines();
    state.session.scroll = 0;
    state.session.follow = true;
}

fn handle_session_confirmation(state: &mut AppState, input: &str) -> bool {
    if !matches!(
        state.session.phase,
        SessionPhase::AwaitingConfirmation | SessionPhase::Ready
    ) {
        return false;
    }

    match input.trim().to_ascii_lowercase().as_str() {
        "proceed" | "confirm" | "yes" => {
            confirm_provider_execution(state);
            true
        }
        "edit" => {
            state.mode = AppMode::Home;
            state.ui.input = state.session.current_task.clone();
            state.ui.status_line = "edit current task".to_string();
            true
        }
        "cancel" => {
            state.cancel_session();
            state.ui.input.clear();
            true
        }
        _ => false,
    }
}

fn confirm_provider_execution(state: &mut AppState) {
    state.session.phase = SessionPhase::Ready;
    state.ui.status_line = "confirmed: provider execution can start".to_string();
    state.ui.input.clear();
}

fn clear_view(state: &mut AppState) {
    state.ui.input.clear();
    state.session.events.clear();
    state.session.phase = SessionPhase::Idle;
    state.session.scroll = 0;
    state.session.follow = true;
    state.session.visible_lines = 0;
    state.ui.status_line = "view cleared".to_string();
}

fn complete_input(state: &mut AppState) {
    let options = complete_slash_command(&state.ui.input);
    if options.len() != 1 {
        state.ui.status_line = options.join("  ");
        return;
    }
    state.ui.input = options[0].to_string();
}

fn apply_palette_selection(state: &mut AppState, history: &mut ShellHistory) -> bool {
    let options = matching_commands(&state.ui.input);
    let Some(command) = options
        .get(state.ui.command_palette_index)
        .map(|spec| spec.name)
    else {
        return false;
    };
    history.push(command);
    state.ui.command_palette_open = false;
    state.ui.palette_mode = PaletteMode::Commands;
    state.ui.command_palette_index = 0;
    apply_command(state, parse_slash_command(command), history);
    true
}

pub fn tick_for_test(state: &mut AppState) {
    state.tick();
}
