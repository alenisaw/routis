#![forbid(unsafe_code)]
#![deny(warnings)]

mod trace_cli;
mod trace_store;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use routis::tui::app::run_app;
use routis_core::{ProviderCommandPreview, RepoFact};

#[derive(Debug, Parser)]
#[command(name = "routis")]
#[command(version, about = "Interactive TUI for routing AI coding tasks.")]
struct Args {
    /// Start the interactive TUI and exit immediately in smoke tests.
    #[arg(long, hide = true)]
    smoke: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Preview the routing decision without provider execution.
    Route {
        /// Print the decision tree. A local JSONL trace is stored by default.
        #[arg(long)]
        explain: bool,

        /// Task to classify and route.
        task: Vec<String>,
    },
    /// Print repository context summary.
    Context,
    /// Show local decision traces.
    Traces {
        /// Print the latest full trace tree instead of the summary table.
        #[arg(long)]
        latest: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    match args.command {
        Some(Command::Route { explain, task }) => run_route(task, explain),
        Some(Command::Context) => run_context(),
        Some(Command::Traces { latest }) => run_traces(latest),
        None => run_tui().await,
    }
}

fn run_route(task: Vec<String>, explain: bool) -> Result<()> {
    let task = task.join(" ");
    let task = task.trim();
    if task.is_empty() {
        anyhow::bail!("route task must not be empty");
    }
    let route = routis::route_plan::build_execution_plan_with_decision(
        task,
        routis::route_plan::DEFAULT_POLICY_PATH,
        std::env::current_dir()?,
    )?;
    let plan = route.plan;
    let trace = trace_cli::build_cli_decision_trace(
        task,
        &route.decision,
        trace_cli::CliDecisionTraceInput {
            selected_model: plan.model.clone(),
            selected_reasoning: plan.reasoning.clone(),
            execution_mode: "preview".to_string(),
            provider_command_preview: Some(ProviderCommandPreview {
                program: "codex".to_string(),
                args: vec![
                    "exec".to_string(),
                    "-m".to_string(),
                    plan.model.clone(),
                    "--reasoning".to_string(),
                    plan.reasoning.clone(),
                    "--".to_string(),
                    "<task-redacted>".to_string(),
                ],
            }),
            policy_source: plan.policy_source.clone(),
            policy_overrides: route.policy_overrides,
            risk_zones: route.repo_context.risk_zone_hints,
            repo_facts: vec![RepoFact::new("policy-source", plan.policy_source.clone())],
        },
    )?;
    trace_cli::append_cli_trace(&trace).context("failed to store decision trace")?;

    println!("task_hash: {}", trace.task_hash);
    println!(
        "selected: {} / {} / {}",
        plan.profile, plan.model, plan.reasoning
    );
    println!("intent: {}", plan.intent);
    println!("area: {}", plan.area);
    println!("scope: {}", plan.scope);
    println!("risk: {}", plan.risk);
    println!("confidence: {}", plan.confidence);
    println!("reason: {}", plan.reason);
    if explain {
        println!();
        trace_cli::print_trace_tree(&trace);
    }
    Ok(())
}

fn run_traces(latest: bool) -> Result<()> {
    if latest {
        trace_cli::print_latest_trace()
    } else {
        trace_cli::print_trace_list()
    }
}

fn run_context() -> Result<()> {
    let context = routis::route_plan::collect_repo_context_for_cwd(std::env::current_dir()?)?;
    let summary = routis::route_plan::repo_map_summary(&context);
    println!("branch: {}", summary.branch);
    println!("changed files: {}", summary.changed_files);
    println!("markers: {}", list(&summary.repo_markers));
    println!("manifests: {}", list(&summary.manifests));
    println!("docs: {}", list(&summary.docs));
    println!("tests: {}", list(&summary.tests));
    println!("workflows: {}", list(&summary.workflows));
    println!("instructions: {}", list(&summary.instruction_files));
    Ok(())
}

fn list(values: &[String]) -> String {
    if values.is_empty() {
        "-".to_string()
    } else {
        values.join(", ")
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
