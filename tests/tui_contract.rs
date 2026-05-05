use assert_cmd::Command;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use predicates::prelude::*;
use ratatui::{backend::TestBackend, style::Color, Terminal};
use routis::tui::{
    app::{handle_key_for_test, handle_key_with_history_for_test, tick_for_test},
    command::{complete_slash_command, parse_slash_command, SlashCommand, COMMANDS},
    history::ShellHistory,
    render::{mascot_lines, render_app},
    session::make_session_title,
    state::{
        select_codex_candidate, AppMode, AppState, LayoutMode, ProviderDiagnostics, SessionPhase,
        SetupStep,
    },
};

#[test]
fn setup_flow_is_simple_and_reopenable() {
    let mut state = AppState::setup();

    assert_eq!(state.setup.step, SetupStep::Welcome);
    state.confirm();
    assert_eq!(state.setup.step, SetupStep::Name);
    state.config.display_name.clear();
    for ch in "Mira".chars() {
        handle_key_for_test(
            &mut state,
            KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE),
        );
    }
    assert_eq!(state.config.display_name, "Mira");
    state.confirm();
    assert_eq!(state.setup.step, SetupStep::Provider);
    state.confirm();
    assert!(state.setup.provider_checked);
    state.provider_diagnostics.command = "Found".to_string();
    state.confirm();
    assert_eq!(state.setup.step, SetupStep::Theme);
    state.confirm();
    assert_eq!(state.setup.step, SetupStep::Finish);
    state.confirm();
    assert_eq!(state.mode, AppMode::Home);

    state.ui.input = "/setup".to_string();
    handle_key_for_test(
        &mut state,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );

    assert_eq!(state.mode, AppMode::Setup);
    assert_eq!(state.setup.step, SetupStep::Welcome);
}

#[test]
fn setup_screen_uses_left_mascot_right_copy_and_no_outer_frame() {
    let state = AppState::setup();
    let text = render_to_text(120, 36, &state);

    assert!(text.contains("Routis Setup v0.2.2"));
    assert!(text.contains("Welcome to Routis!"));
    assert!(text.contains("What this setup does"));
    assert!(text.contains("1  Start setup"));
    assert!(text.contains("2  Import config"));
    assert!(text.contains("3  Exit"));
    assert!(text.contains("▄      ▄▄"));
    assert!(text.contains("· · ·"));
    assert!(!text.contains('╭'));
    assert!(!text.contains('╮'));
    assert!(!text.contains('╰'));
    assert!(!text.contains('╯'));
    assert!(!text.contains("security policy"));
    assert!(!text.contains("security-strict"));
}

#[test]
fn setup_name_step_allows_user_name_entry() {
    let mut state = AppState::setup();
    state.setup.step = SetupStep::Name;
    state.config.display_name.clear();

    for ch in "Alena".chars() {
        handle_key_for_test(
            &mut state,
            KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE),
        );
    }
    let text = render_to_text(120, 36, &state);

    assert_eq!(state.config.display_name, "Alena");
    assert!(text.contains("What should Routis call you?"));
    assert!(text.contains("Alena"));
    assert!(text.contains("local only"));
}

#[test]
fn setup_model_and_theme_are_configurable() {
    let mut state = AppState::setup();
    state.setup.step = SetupStep::Theme;
    handle_key_for_test(
        &mut state,
        KeyEvent::new(KeyCode::Char('3'), KeyModifiers::NONE),
    );
    let (text, buffer) = render_to_text_and_buffer(120, 36, &state);

    assert_eq!(state.config.theme, "Neon Magenta");
    assert!(text.contains("Preview: Neon Magenta"));
    assert!(buffer_has_fg(&buffer, Color::Rgb(244, 114, 182)));
    assert!(buffer_has_bg(&buffer, Color::Rgb(157, 23, 77)));
}

#[test]
fn setup_provider_shows_diagnostics_below_selection() {
    let mut state = AppState::setup();
    state.setup.step = SetupStep::Provider;
    let text = render_to_text(140, 40, &state);

    assert!(text.contains("Provider check"));
    assert!(text.contains("binary"));
    assert!(text.contains("version"));
    assert!(text.contains("config"));
    assert!(text.contains("auth"));
    assert!(text.contains("next"));
    assert!(text.find("1  Codex CLI").unwrap() < text.find("Provider check").unwrap());
}

#[test]
fn setup_provider_check_shows_result_and_enter_continues_when_found() {
    let mut state = AppState::setup();
    state.setup.step = SetupStep::Provider;
    state.provider_diagnostics.command = "Unavailable".to_string();
    state.provider_diagnostics.version = "Unavailable".to_string();

    handle_key_for_test(
        &mut state,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );

    assert!(state.setup.provider_checked);
    assert!(
        state.ui.status_line.contains("passed") || state.ui.status_line.contains("failed"),
        "{}",
        state.ui.status_line
    );
    assert_ne!(state.provider_diagnostics.command.trim(), "");
    assert_ne!(state.provider_diagnostics.version.trim(), "");

    state.provider_diagnostics.command = "Found".to_string();
    state.provider_diagnostics.version = "codex-cli 1.0.0".to_string();
    handle_key_for_test(
        &mut state,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );

    assert_eq!(state.setup.step, SetupStep::Theme);
}

#[test]
fn codex_candidate_lookup_prefers_executable_wrappers_over_powershell_scripts() {
    let candidate = select_codex_candidate([
        "C:\\Users\\alenk\\AppData\\Roaming\\npm\\codex.ps1",
        "C:\\Users\\alenk\\AppData\\Roaming\\npm\\codex.cmd",
        "C:\\Tools\\codex.exe",
    ]);

    assert_eq!(
        candidate.as_deref(),
        Some("C:\\Users\\alenk\\AppData\\Roaming\\npm\\codex.cmd")
    );
}

#[test]
fn setup_no_longer_contains_model_step_or_selection() {
    let state = AppState::setup();
    let text = render_to_text(140, 40, &state);

    assert!(!SetupStep::ALL.iter().any(|step| step.label() == "Model"));
    assert!(!text.contains("3. Model"));
    assert!(!text.contains("Choose model"));
    assert!(!text.contains("Reasoning effort"));
}

#[test]
fn slash_command_registry_matches_real_tui_surface() {
    assert_eq!(parse_slash_command("/help").unwrap(), SlashCommand::Help);
    assert_eq!(parse_slash_command("/setup").unwrap(), SlashCommand::Setup);
    assert_eq!(
        parse_slash_command("/status").unwrap(),
        SlashCommand::Status
    );
    assert_eq!(
        parse_slash_command("/provider").unwrap(),
        SlashCommand::Provider
    );
    assert_eq!(parse_slash_command("/theme").unwrap(), SlashCommand::Theme);
    assert_eq!(
        parse_slash_command("/doctor").unwrap(),
        SlashCommand::Doctor
    );
    assert_eq!(parse_slash_command("/clear").unwrap(), SlashCommand::Clear);
    assert_eq!(
        parse_slash_command("/history").unwrap(),
        SlashCommand::History
    );
    assert_eq!(
        parse_slash_command("/sessions").unwrap(),
        SlashCommand::Sessions
    );
    assert_eq!(parse_slash_command("/quit").unwrap(), SlashCommand::Quit);
    assert!(parse_slash_command("/model").is_err());
    assert!(parse_slash_command("/effort").is_err());
    assert!(parse_slash_command("/login").is_err());
    assert!(parse_slash_command("/context").is_err());
    assert!(parse_slash_command("/security").is_err());
}

#[test]
fn slash_palette_is_filterable_and_rich_enough() {
    let all = complete_slash_command("/");
    assert!(all.len() >= 9);
    assert!(all.contains(&"/setup"));
    assert!(all.contains(&"/doctor"));
    assert!(all.contains(&"/sessions"));
    assert!(!all.contains(&"/model"));
    assert!(!all.contains(&"/effort"));
    assert!(!all.contains(&"/context"));
    assert!(!all.contains(&"/login"));
    assert!(!all.contains(&"/security"));
    assert_eq!(complete_slash_command("/th"), vec!["/theme"]);

    let mut state = AppState::home();
    handle_key_for_test(
        &mut state,
        KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE),
    );
    let text = render_to_text(140, 44, &state);

    assert!(state.ui.command_palette_open);
    assert!(text.contains("/status"));
    assert!(text.contains("/doctor"));
    assert!(text.contains("/history"));
    assert!(!text.contains("/provider"));
    assert!(!text.contains("/sessions"));
    assert!(!text.contains("Run a security review"));
    assert!(!text.contains("routis>"));

    state.ui.input = "/se".to_string();
    let filtered = render_to_text(140, 44, &state);
    assert!(filtered.contains("/sessions"));
}

#[test]
fn command_palette_uses_dedicated_bottom_panel() {
    let mut state = AppState::home();
    state.ui.input = "/p".to_string();
    state.ui.command_palette_open = true;

    let text = render_to_text(150, 44, &state);
    let lines = text.lines().collect::<Vec<_>>();
    let command_y = lines
        .iter()
        .position(|line| line.contains("/provider"))
        .unwrap();
    let rule_y = lines
        .iter()
        .rposition(|line| line.contains("─") || line.contains("в”Ђ"))
        .unwrap();

    assert!(command_y > 20);
    assert!(command_y < rule_y);
}

#[test]
fn command_selected_from_palette_is_saved_to_history() {
    let mut state = AppState::home();
    let mut history = ShellHistory::new(10);
    handle_key_with_history_for_test(
        &mut state,
        &mut history,
        KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE),
    );
    handle_key_with_history_for_test(
        &mut state,
        &mut history,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );

    assert_eq!(history.entries(), &["/status"]);
    assert!(state
        .session
        .events
        .iter()
        .any(|event| event.title == "Command result"));
}

#[test]
fn sessions_command_opens_selectable_session_picker() {
    let mut state = AppState::home();
    let mut history = ShellHistory::new(10);
    history.push("first task");
    history.push("second task");
    state.ui.input = "/sessions".to_string();

    handle_key_with_history_for_test(
        &mut state,
        &mut history,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );

    let text = render_to_text(150, 44, &state);
    assert!(state.ui.command_palette_open);
    assert!(text.contains("Resume a previous session"));
    assert!(text.contains("Created"));
    assert!(text.contains("Updated"));
    assert!(text.contains("Branch"));
    assert!(text.contains("Conversation"));
    assert!(text.contains("second task"));

    handle_key_with_history_for_test(
        &mut state,
        &mut history,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );

    assert_eq!(state.mode, AppMode::Session);
    assert_eq!(state.session.title, "second");
}

#[test]
fn sessions_picker_switches_selection_with_arrow_keys() {
    let mut state = AppState::home();
    let mut history = ShellHistory::new(10);
    history.push("first task");
    history.push("second task");
    state.ui.input = "/sessions".to_string();

    handle_key_with_history_for_test(
        &mut state,
        &mut history,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );
    handle_key_with_history_for_test(
        &mut state,
        &mut history,
        KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
    );
    handle_key_with_history_for_test(
        &mut state,
        &mut history,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );

    assert_eq!(state.mode, AppMode::Session);
    assert_eq!(state.session.title, "first");
    assert_eq!(state.session.current_task, "first task");
}

#[test]
fn sessions_picker_filters_by_typed_search() {
    let mut state = AppState::home();
    let mut history = ShellHistory::new(10);
    history.push("debug auth flow");
    history.push("update docs");
    state.ui.input = "/sessions".to_string();

    handle_key_with_history_for_test(
        &mut state,
        &mut history,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );
    handle_key_with_history_for_test(
        &mut state,
        &mut history,
        KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE),
    );
    handle_key_with_history_for_test(
        &mut state,
        &mut history,
        KeyEvent::new(KeyCode::Char('u'), KeyModifiers::NONE),
    );

    let text = render_to_text(150, 44, &state);
    assert!(text.contains("Search: au"));
    assert!(text.contains("debug auth flow"));
    assert!(!text.contains("update docs"));
}

#[test]
fn sessions_command_does_not_dump_sessions_into_chat() {
    let mut state = AppState::home();
    let mut history = ShellHistory::new(10);
    history.push("first task");
    state.ui.input = "/sessions".to_string();

    handle_key_with_history_for_test(
        &mut state,
        &mut history,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );

    assert!(state.ui.command_palette_open);
    assert!(state
        .session
        .events
        .iter()
        .any(|event| event.title == "Command result"));
    assert!(!state
        .session
        .events
        .iter()
        .any(|event| event.title == "Recent sessions"));
}

#[test]
fn theme_command_opens_inline_picker_without_setup() {
    let mut state = AppState::home();
    state.ui.input = "/theme".to_string();

    handle_key_for_test(
        &mut state,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );

    assert_eq!(state.mode, AppMode::Home);
    assert!(state.ui.command_palette_open);

    let text = render_to_text(150, 44, &state);
    assert!(text.contains("Choose theme"));
    assert!(text.contains("Routis Cyan"));
    assert!(text.contains("Monochrome"));
}

#[test]
fn provider_picker_enter_checks_and_accepts_codex_provider() {
    let mut state = AppState::home();
    state.provider_diagnostics = ProviderDiagnostics {
        command: "Found".to_string(),
        version: "codex-cli 0.128.0".to_string(),
        auth_status: "Ready to use local Codex CLI".to_string(),
        config_path: "C:\\Users\\alenk\\.routis\\config.toml".to_string(),
    };
    state.ui.input = "/provider".to_string();

    handle_key_for_test(
        &mut state,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );
    handle_key_for_test(
        &mut state,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );

    assert_eq!(state.config.provider, "Codex CLI");
    assert!(!state.ui.command_palette_open);
    assert!(state.ui.status_line.contains("Codex CLI Found"));
}

#[test]
fn ctrl_c_cancels_or_clears_without_immediate_exit() {
    let mut state = AppState::home();
    state.ui.input = "draft task".to_string();

    handle_key_for_test(
        &mut state,
        KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
    );

    assert_eq!(state.mode, AppMode::Home);
    assert!(state.ui.input.is_empty());

    state.ui.input = "run slow task".to_string();
    handle_key_for_test(
        &mut state,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );
    assert_eq!(state.mode, AppMode::Session);
    assert_ne!(state.session.phase, SessionPhase::Cancelled);

    handle_key_for_test(
        &mut state,
        KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
    );

    assert_eq!(state.mode, AppMode::Session);
    assert_eq!(state.session.phase, SessionPhase::Cancelled);
    assert!(state.ui.status_line.contains("cancelled"));
}

#[test]
fn ctrl_d_exits_from_home() {
    let mut state = AppState::home();
    handle_key_for_test(
        &mut state,
        KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL),
    );

    assert_eq!(state.mode, AppMode::Exit);
}

#[test]
fn escape_does_not_exit_routis_from_home_or_setup_welcome() {
    let mut home = AppState::home();
    handle_key_for_test(&mut home, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert_eq!(home.mode, AppMode::Home);

    let mut setup = AppState::setup();
    setup.setup.step = SetupStep::Welcome;
    handle_key_for_test(&mut setup, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert_eq!(setup.mode, AppMode::Setup);
}

#[test]
fn home_header_has_greeting_metrics_and_dotted_internal_dividers() {
    let state = AppState::home();
    let text = render_to_text(150, 44, &state);

    assert!(text.contains("Routis v0.2.2"));
    assert!(text.contains("Welcome,"));
    assert!(text.contains("Workspace:"));
    assert!(text.contains("~/"));
    assert!(text.contains("Updates"));
    assert!(text.find("Updates").unwrap() < text.find("Recent Sessions").unwrap());
    assert!(text.contains("Metrics"));
    assert!(text.contains("context"));
    assert!(text.contains("input"));
    assert!(text.contains("output"));
    assert!(text.contains("total"));
    assert!(text.contains("saved"));
    assert!(text.contains("context  "));
    assert!(text.contains("input    "));
    assert!(text.contains("output   "));
    assert!(text.contains("total    "));
    assert!(text.contains("Recent Sessions"));
    assert!(!text.contains("CHANGELOG"));
    assert!(!text.contains("more"));
    assert!(text.contains("provider"));
    assert!(!text.contains("tasks    "));
    assert!(text.contains("┊"));
    assert!(!text.contains("Context"));
    assert!(!text.contains("cwd      "));
    assert!(!text.contains("cwd:"));
    assert!(!text.contains("security-strict"));
    assert!(!text.contains("security-first"));
}

#[test]
fn header_is_compact_and_left_greeting_is_centered_with_workspace() {
    let state = AppState::home();
    let text = render_to_text(150, 44, &state);
    let lines = text.lines().collect::<Vec<_>>();
    let input_line = lines
        .iter()
        .rposition(|line| line.contains("Type a task or / for commands"))
        .unwrap();

    assert!(input_line > 20);
    assert!(text.contains("Welcome,"));
    assert!(text.contains("Workspace:"));
    assert!(text.contains("~/"));
    assert!(!text.contains("Local shell for"));
}

#[test]
fn full_header_omits_context_block() {
    let state = AppState::home();
    let text = render_to_text(150, 44, &state);

    assert!(text.contains("Metrics"));
    assert!(!text.contains("Context"));
    assert!(!text.contains("cwd      "));
    assert!(!text.contains("config   ~/.routis/config.toml"));
    assert!(!text.contains("state    idle"));
}

#[test]
fn default_theme_is_cyan_and_static_across_frames() {
    let mut state = AppState::home();
    assert_eq!(state.config.theme, "Routis Cyan");

    let (_text_a, buffer_a) = render_to_text_and_buffer(150, 44, &state);
    state.ui.frame = 9;
    let (_text_b, buffer_b) = render_to_text_and_buffer(150, 44, &state);

    assert!(buffer_has_fg(&buffer_a, Color::Rgb(92, 200, 215)));
    assert_eq!(buffer_a.content(), buffer_b.content());
}

#[test]
fn session_timeline_uses_plain_routis_execution_flow() {
    let mut state = AppState::home();
    state.ui.input = "debug auth flow".to_string();
    handle_key_for_test(
        &mut state,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );

    for _ in 0..8 {
        tick_for_test(&mut state);
    }

    let text = render_to_text(150, 48, &state);

    assert!(text.contains("active-session"));
    assert!(!text.contains("debug-auth-flow"));
    assert!(text.contains("You"));
    assert!(text.contains("Routis"));
    assert!(text.contains("Command preview"));
    assert!(text.contains("codex exec"));
    assert!(text.contains("Awaiting confirmation"));
    assert!(text.contains("├─"));
    assert!(text.contains("└─"));
    assert!(!text.contains("Security policy"));
    assert!(!text.contains("security-strict"));
}

#[test]
fn session_timeline_follows_bottom_when_new_prompt_starts() {
    let mut state = AppState::home();
    for index in 0..12 {
        state.session.events.push(routis::tui::state::SessionEvent {
            source: "Routis".to_string(),
            title: format!("old event {index}"),
            lines: vec!["detail".to_string()],
        });
    }
    state.ui.input = "fresh prompt".to_string();

    handle_key_for_test(
        &mut state,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );

    let text = render_to_text(120, 36, &state);
    assert!(text.contains("fresh prompt"));
    assert!(!text.contains("old event 0"));
}

#[test]
fn empty_session_area_has_no_scroll_artifacts() {
    let state = AppState::home();
    let text = render_to_text(150, 44, &state);

    assert!(!text.contains("↑"));
    assert!(!text.contains("↓"));
    assert!(!text.contains("scroll"));
}

#[test]
fn empty_session_does_not_scroll_on_arrow_keys() {
    let mut state = AppState::home();
    handle_key_for_test(&mut state, KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));

    assert_eq!(state.session.scroll, 0);
}

#[test]
fn slash_command_result_renders_in_session_area() {
    let mut state = AppState::home();
    state.ui.input = "/status".to_string();
    handle_key_for_test(
        &mut state,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );

    let text = render_to_text(150, 44, &state);
    assert!(text.contains("Command result"));
    assert!(text.contains("reasoning"));
    assert!(!text.contains("Routis  Type a task"));
}

#[test]
fn status_doctor_config_and_history_render_useful_command_blocks() {
    let mut history = ShellHistory::new(10);
    history.push("debug auth flow");
    history.push("update docs");

    for command in ["/status", "/doctor", "/config", "/history"] {
        let mut state = AppState::home();
        state.ui.input = command.to_string();
        handle_key_with_history_for_test(
            &mut state,
            &mut history,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        );

        let event = state
            .session
            .events
            .iter()
            .find(|event| event.title == "Command result")
            .unwrap_or_else(|| panic!("{command} did not render a command result"));
        assert!(
            event.lines.len() >= 3,
            "{command} rendered too little detail: {:?}",
            event.lines
        );
    }

    let mut state = AppState::home();
    state.ui.input = "/history".to_string();
    handle_key_with_history_for_test(
        &mut state,
        &mut history,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );
    let text = render_to_text(150, 44, &state);
    assert!(text.contains("debug auth flow"));
    assert!(text.contains("update docs"));
}

#[test]
fn slash_commands_append_results_without_replacing_previous_events() {
    let mut state = AppState::home();
    let mut history = ShellHistory::new(10);

    for command in ["/status", "/doctor", "/config", "/history"] {
        state.ui.input = command.to_string();
        handle_key_with_history_for_test(
            &mut state,
            &mut history,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        );
    }

    let command_results = state
        .session
        .events
        .iter()
        .filter(|event| event.title == "Command result")
        .count();
    assert_eq!(command_results, 4);
    assert_eq!(
        history.entries(),
        &["/status", "/doctor", "/config", "/history"]
    );
}

#[test]
fn slash_command_history_survives_next_prompt() {
    let mut state = AppState::home();
    state.ui.input = "/status".to_string();
    handle_key_for_test(
        &mut state,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );
    state.ui.input = "next task".to_string();
    handle_key_for_test(
        &mut state,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );

    assert!(state
        .session
        .events
        .iter()
        .any(|event| event.title == "Command result"));
    let text = render_to_text(150, 44, &state);
    assert!(text.contains("next task"));
}

#[test]
fn clear_command_does_not_drop_session_command_history() {
    let mut state = AppState::home();
    state.ui.input = "/status".to_string();
    handle_key_for_test(
        &mut state,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );
    assert!(!state.session.events.is_empty());

    state.ui.input = "/clear".to_string();
    handle_key_for_test(
        &mut state,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );

    assert_eq!(state.session.events.len(), 1);
    assert_eq!(state.session.events[0].title, "Command result");
    assert_eq!(state.session.events[0].lines, vec!["view cleared"]);
    assert_eq!(state.session.phase, SessionPhase::Idle);
}

#[test]
fn every_slash_command_records_a_session_result() {
    for spec in COMMANDS {
        if spec.name == "/quit" {
            continue;
        }
        let mut state = AppState::home();
        state.ui.input = spec.name.to_string();
        handle_key_for_test(
            &mut state,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        );

        assert!(
            state
                .session
                .events
                .iter()
                .any(|event| event.title == "Command result"),
            "{} did not record a command result",
            spec.name
        );
    }
}

#[test]
fn shortcuts_render_inside_session_area_not_as_overlay() {
    let mut state = AppState::home();
    handle_key_for_test(
        &mut state,
        KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE),
    );
    let text = render_to_text(150, 44, &state);

    assert!(text.contains("Keyboard shortcuts"));
    assert!(text.contains("Ctrl+C"));
    assert!(text.contains("Esc"));
    assert!(text.find("new-session").unwrap() < text.find("Keyboard shortcuts").unwrap());
}

#[test]
fn setup_stage_copy_changes_for_provider_and_theme() {
    let mut provider = AppState::setup();
    provider.setup.step = SetupStep::Provider;
    let provider_text = render_to_text(140, 40, &provider);

    let mut theme = AppState::setup();
    theme.setup.step = SetupStep::Theme;
    let theme_text = render_to_text(140, 40, &theme);

    assert!(provider_text.contains("Provider"));
    assert!(provider_text.contains("Check that the selected CLI is installed"));
    assert!(theme_text.contains("Theme"));
    assert!(theme_text.contains("Choose a readable terminal palette"));
    assert_ne!(provider_text, theme_text);
}

#[test]
fn doctor_command_reports_real_provider_and_config_diagnostics() {
    let mut state = AppState::home();
    state.ui.input = "/doctor".to_string();

    handle_key_for_test(
        &mut state,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );

    assert!(state.ui.status_line.contains("doctor:"));
    assert!(state.ui.status_line.contains("codex"));
    assert!(state.ui.status_line.contains("config"));
    assert!(state.ui.status_line.contains(".routis"));
}

#[test]
fn input_history_is_not_scrolled_with_arrow_keys() {
    let mut state = AppState::home();
    let mut history = ShellHistory::new(10);
    history.push("first task");
    history.push("second task");

    handle_key_with_history_for_test(
        &mut state,
        &mut history,
        KeyEvent::new(KeyCode::Up, KeyModifiers::NONE),
    );
    assert_eq!(state.ui.input, "");

    handle_key_with_history_for_test(
        &mut state,
        &mut history,
        KeyEvent::new(KeyCode::Up, KeyModifiers::NONE),
    );
    assert_eq!(state.ui.input, "");

    handle_key_with_history_for_test(
        &mut state,
        &mut history,
        KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
    );
    assert_eq!(state.ui.input, "");
}

#[test]
fn session_confirm_flow_accepts_proceed_edit_and_cancel() {
    let mut state = AppState::home();
    state.ui.input = "debug auth flow".to_string();
    handle_key_for_test(
        &mut state,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );
    for _ in 0..8 {
        tick_for_test(&mut state);
    }
    assert_eq!(state.session.phase, SessionPhase::AwaitingConfirmation);

    state.ui.input = "edit".to_string();
    handle_key_for_test(
        &mut state,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );
    assert_eq!(state.mode, AppMode::Home);
    assert_eq!(state.ui.input, "debug auth flow");

    handle_key_for_test(
        &mut state,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );
    for _ in 0..8 {
        tick_for_test(&mut state);
    }
    state.ui.input = "proceed".to_string();
    handle_key_for_test(
        &mut state,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );
    assert_eq!(state.session.phase, SessionPhase::Ready);
    assert!(state.ui.status_line.contains("confirmed"));

    state.ui.input = "cancel".to_string();
    handle_key_for_test(
        &mut state,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );
    assert_eq!(state.session.phase, SessionPhase::Cancelled);
}

#[test]
fn home_header_inherits_configured_theme_palette() {
    let config = routis::tui::state::ConfigState {
        theme: "Midnight Blue".to_string(),
        ..Default::default()
    };
    let state = AppState::with_config(config);

    let (_text, buffer) = render_to_text_and_buffer(150, 44, &state);

    assert!(buffer_has_fg(&buffer, Color::Rgb(96, 165, 250)));
}

#[test]
fn mascot_is_exactly_the_approved_shape() {
    let lines = mascot_lines();

    assert_eq!(
        lines,
        &[
            "        ▄▄      ▄▄",
            "       ████    ████",
            "      ██████████████",
            "     ██  ██    ██  ██",
            "     ████████████████",
            "      ██████████████",
            "        ██      ██",
        ]
    );
    assert!(lines.iter().all(|line| !line.contains('\n')));
}

#[test]
fn history_prunes_oldest_entries() {
    let mut history = ShellHistory::new(3);

    history.push("one");
    history.push("two");
    history.push("three");
    history.push("four");

    assert_eq!(history.entries(), &["two", "three", "four"]);
}

#[test]
fn history_recent_sessions_include_relative_time() {
    let mut history = ShellHistory::new(3);

    history.push("debug auth flow");

    let recent = history.recent(1);
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].0, "debug-auth-flow");
    assert!(recent[0].1 == "now" || recent[0].1.ends_with(" ago"));
}

#[test]
fn session_title_uses_first_meaningful_task() {
    assert_eq!(make_session_title(""), "new-session");
    assert_eq!(
        make_session_title("Review the auth implementation and login callback"),
        "review-auth-implementation-login"
    );
    assert_eq!(
        make_session_title("сделай проверку команд в сетапе"),
        "проверку-команд-сетапе"
    );
}

#[test]
fn recent_sessions_ignore_slash_commands_and_use_titles() {
    let mut history = ShellHistory::new(10);
    history.push("/status");
    history.push("почини вывод команд в истории");
    let recent = history.recent(3);

    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].0, "почини-вывод-команд-истории");
}

#[test]
fn layout_mode_breakpoints_match_prompt() {
    assert_eq!(LayoutMode::for_width(99), LayoutMode::Minimal);
    assert_eq!(LayoutMode::for_width(100), LayoutMode::Compact);
    assert_eq!(LayoutMode::for_width(149), LayoutMode::Compact);
    assert_eq!(LayoutMode::for_width(150), LayoutMode::Wide);
}

#[test]
fn render_smoke_for_small_and_wide_sizes() {
    for (width, height) in [(60, 20), (150, 44)] {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        let state = AppState::home();

        terminal.draw(|frame| render_app(frame, &state)).unwrap();

        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("Routis"));
    }
}

#[test]
fn shell_layout_survives_required_terminal_sizes() {
    for (width, height) in [
        (204, 30),
        (180, 40),
        (150, 36),
        (120, 30),
        (120, 32),
        (100, 28),
        (80, 24),
    ] {
        let text = render_to_text(width, height, &AppState::home());

        assert!(text.contains("Routis v0.2.2"));
        assert!(text.contains("Type a task or / for commands"));
        assert!(text.contains("Metrics"));
        assert!(
            text.contains("Metrics"),
            "missing metrics at {width}x{height}"
        );
        assert!(
            text.contains("~/"),
            "missing workspace path at {width}x{height}"
        );
        assert!(!text.contains("Terminal too small"));
    }
}

#[test]
fn shell_too_small_fallback_is_plain_and_safe() {
    let text = render_to_text(79, 23, &AppState::home());

    assert!(text.contains("Routis v0.2.2"));
    assert!(text.contains("Terminal too small."));
    assert!(text.contains("80x24"));
}

#[test]
fn routis_without_task_enters_tui_path_when_smoke_env_is_set() {
    let mut cmd = Command::cargo_bin("routis").unwrap();
    cmd.env("ROUTIS_TUI_SMOKE_EXIT", "1")
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

fn render_to_text(width: u16, height: u16, state: &AppState) -> String {
    render_to_text_and_buffer(width, height, state).0
}

fn render_to_text_and_buffer(
    width: u16,
    height: u16,
    state: &AppState,
) -> (String, ratatui::buffer::Buffer) {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|frame| render_app(frame, state)).unwrap();

    let buffer = terminal.backend().buffer().clone();
    (buffer_text(&buffer), buffer)
}

fn buffer_text(buffer: &ratatui::buffer::Buffer) -> String {
    let mut text = String::new();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            text.push_str(buffer[(x, y)].symbol());
        }
        text.push('\n');
    }
    text
}

fn buffer_has_fg(buffer: &ratatui::buffer::Buffer, color: Color) -> bool {
    buffer.content().iter().any(|cell| cell.fg == color)
}

fn buffer_has_bg(buffer: &ratatui::buffer::Buffer, color: Color) -> bool {
    buffer.content().iter().any(|cell| cell.bg == color)
}
