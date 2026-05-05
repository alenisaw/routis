#![forbid(unsafe_code)]
#![deny(warnings)]

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use routis::session_store::{default_session_store_path, SessionRecord, SessionStore};
use routis::tui::app::run_app;
use routis_context::{collect_repo_context, RepoContext};
use routis_core::{route_task_with_repo_context, Profile, RoutingDecision};
use routis_policy::{
    apply_policy_rules, build_codex_command, format_command, PolicyFile, ProfileExecutionConfig,
};
use std::{path::PathBuf, process::Command, str::FromStr};

const DEFAULT_POLICY_PATH: &str = "configs/policies/default.yaml";

#[derive(Debug, Parser)]
#[command(name = "routis")]
#[command(version, about = "Policy-aware CLI for routing AI coding tasks.")]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

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

#[derive(Debug, Subcommand)]
enum Commands {
    /// Show repository context used by routing.
    Context,
    /// Work with local Routis sessions.
    Session {
        #[command(subcommand)]
        command: SessionCommands,
    },
}

#[derive(Debug, Subcommand)]
enum SessionCommands {
    /// List saved local sessions.
    List,
    /// Print a saved session route preview.
    Resume {
        /// Session id prefix or title.
        query: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    if let Some(command) = &args.command {
        return match command {
            Commands::Context => print_context_command(),
            Commands::Session {
                command: SessionCommands::List,
            } => print_session_list(),
            Commands::Session {
                command: SessionCommands::Resume { query },
            } => print_session_resume(query),
        };
    }
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
    let repo_context = collect_repo_context(std::env::current_dir()?)?;
    let mut decision = route_task_with_repo_context(
        &task,
        requested_profile,
        &repo_context.risk_zone_hints,
        repo_context.changed_files.len(),
    )?;
    if requested_profile == Profile::Default {
        decision.effective_profile = apply_policy_rules(
            &policy,
            decision.effective_profile,
            &repo_context.risk_zone_hints,
            &repo_context.changed_files,
        );
    }
    let codex_command = build_codex_command(&policy, decision.effective_profile, &task)?;
    let execution = policy
        .execution_config(decision.effective_profile)
        .context("selected profile has no execution config")?;
    save_cli_session(&task, &args.policy, &decision, execution, &repo_context)?;
    let execution_mode = match (args.execute, args.dry_run) {
        (true, _) => "execute",
        (false, true) | (false, false) => "dry-run",
    };

    print_decision(
        &decision,
        &codex_command,
        execution_mode,
        args.explain,
        &repo_context,
    );

    if args.execute {
        execute_codex(&codex_command)?;
    }

    Ok(())
}

fn print_session_list() -> Result<()> {
    let sessions = SessionStore::new(default_session_store_path()).list()?;
    if sessions.is_empty() {
        println!("No local sessions found.");
        return Ok(());
    }
    println!("Title              Profile     Model       Branch");
    for session in sessions {
        println!(
            "{:<18} {:<11} {:<11} {}",
            session.title, session.effective_profile, session.model, session.branch
        );
    }
    Ok(())
}

fn print_session_resume(query: &str) -> Result<()> {
    let Some(session) = SessionStore::new(default_session_store_path()).find(query)? else {
        bail!("session `{query}` was not found");
    };
    println!("Session:          {}", session.title);
    println!("Task:             {}", session.task);
    println!("Branch:           {}", session.branch);
    println!("Policy:           {}", session.policy);
    println!("Effective profile: {}", session.effective_profile);
    println!("Model:            {}", session.model);
    println!("Reasoning:        {}", session.reasoning);
    Ok(())
}

fn save_cli_session(
    task: &str,
    policy: &str,
    decision: &RoutingDecision,
    execution: &ProfileExecutionConfig,
    repo_context: &RepoContext,
) -> Result<()> {
    let record = SessionRecord::new(
        task,
        repo_context.branch.as_deref().unwrap_or("-"),
        policy,
        decision.effective_profile.as_str(),
        &execution.model,
        &execution.reasoning,
    );
    SessionStore::new(default_session_store_path()).save(&record)
}

fn print_context_command() -> Result<()> {
    let repo_context = collect_repo_context(std::env::current_dir()?)?;
    println!(
        "Branch:           {}",
        repo_context.branch.as_deref().unwrap_or("-")
    );
    println!("Changed files:    {}", repo_context.changed_files.len());
    println!("Risk zones:       {}", format_risk_zones(&repo_context));
    if !repo_context.changed_files.is_empty() {
        println!("Files:");
        for path in &repo_context.changed_files {
            println!("  {}", path.display());
        }
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
    repo_context: &RepoContext,
) {
    println!("Requested policy:  {}", decision.requested_profile);
    println!("Effective profile: {}", decision.effective_profile);
    println!("Codex command:     {}", format_command(codex_command));
    println!("Execution mode:    {execution_mode}");

    if explain {
        println!();
        println!(
            "Branch:           {}",
            repo_context.branch.as_deref().unwrap_or("-")
        );
        println!("Changed files:    {}", repo_context.changed_files.len());
        println!("Risk zones:       {}", format_risk_zones(repo_context));
        println!("Signals matched:   {:?}", decision.signals_matched);
        println!("Routing reason:    {}", decision.explain);
    }
}

fn format_risk_zones(repo_context: &RepoContext) -> String {
    if repo_context.risk_zone_hints.is_empty() {
        return "-".to_string();
    }
    repo_context
        .risk_zone_hints
        .iter()
        .map(|zone| zone.as_str())
        .collect::<Vec<_>>()
        .join(", ")
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
