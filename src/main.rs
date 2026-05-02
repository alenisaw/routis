#![forbid(unsafe_code)]
#![deny(warnings)]

use anyhow::{bail, Context, Result};
use clap::Parser;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use routis::tui::app::run_app;
use routis_core::{route_task, Profile, RoutingDecision};
use routis_policy::{build_codex_command, format_command, PolicyFile};
use std::{path::PathBuf, process::Command, str::FromStr};

const DEFAULT_POLICY_PATH: &str = "configs/policies/default.yaml";

#[derive(Debug, Parser)]
#[command(name = "routis")]
#[command(version, about = "Policy-aware CLI for routing AI coding tasks.")]
struct Args {
    /// Task to route.
    #[arg(long)]
    task: Option<String>,

    /// Policy profile: cheap | balanced | deep | extradeep | default.
    #[arg(long, default_value = "default")]
    policy: String,

    /// Load execution policy from a YAML file.
    #[arg(long = "policy-file", value_name = "PATH", default_value = DEFAULT_POLICY_PATH)]
    policy_file: PathBuf,

    /// Plan only, do not execute.
    #[arg(long, conflicts_with = "execute")]
    dry_run: bool,

    /// Execute the planned Codex command.
    #[arg(long, conflicts_with = "dry_run")]
    execute: bool,

    /// Show expanded routing detail.
    #[arg(long)]
    explain: bool,

    /// Positional task text.
    #[arg(value_name = "TASK")]
    positional_task: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let Some(task) = resolve_task(&args)? else {
        return run_tui().await;
    };
    let requested_profile = Profile::from_str(&args.policy)?;
    let policy = PolicyFile::load(&args.policy_file).with_context(|| {
        format!(
            "failed to load policy file `{}`",
            args.policy_file.display()
        )
    })?;
    let decision = route_task(&task, requested_profile)?;
    let codex_command = build_codex_command(&policy, decision.effective_profile, &task)?;
    let execution_mode = match (args.execute, args.dry_run) {
        (true, _) => "execute",
        (false, true) | (false, false) => "dry-run",
    };

    print_decision(&decision, &codex_command, execution_mode, args.explain);

    if args.execute {
        execute_codex(&codex_command)?;
    }

    Ok(())
}

fn resolve_task(args: &Args) -> Result<Option<String>> {
    match (&args.task, args.positional_task.is_empty()) {
        (Some(_), false) => bail!("use either --task <TEXT> or positional TASK, not both"),
        (Some(task), true) => Ok(Some(task.clone())),
        (None, false) => Ok(Some(args.positional_task.join(" "))),
        (None, true) => Ok(None),
    }
}

async fn run_tui() -> Result<()> {
    if std::env::var_os("ROUTIS_TUI_SMOKE_EXIT").is_some() {
        return Ok(());
    }

    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(std::io::stdout(), LeaveAlternateScreen);
        original_hook(info);
    }));

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let result = run_app(&mut terminal).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn print_decision(
    decision: &RoutingDecision,
    codex_command: &[String],
    execution_mode: &str,
    explain: bool,
) {
    println!("Requested policy:  {}", decision.requested_profile);
    println!("Effective profile: {}", decision.effective_profile);
    println!("Codex command:     {}", format_command(codex_command));
    println!("Execution mode:    {execution_mode}");

    if explain {
        println!();
        println!("Signals matched:   {:?}", decision.signals_matched);
        println!("Routing reason:    {}", decision.explain);
    }
}

fn execute_codex(codex_command: &[String]) -> Result<()> {
    let (program, args) = codex_command
        .split_first()
        .context("codex command is empty")?;

    let status = Command::new(program).args(args).status().with_context(|| {
        format!("failed to start `{program}`; is Codex CLI installed and on PATH?")
    })?;

    if !status.success() {
        bail!("Codex command exited with status {status}");
    }

    Ok(())
}
