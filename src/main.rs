#![forbid(unsafe_code)]
#![deny(warnings)]

use anyhow::Result;
use clap::{Parser, Subcommand};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use routis::tui::app::run_app;

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
        /// Task to classify and route.
        task: Vec<String>,
    },
    /// Print repository context summary.
    Context,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    match args.command {
        Some(Command::Route { task }) => run_route(task),
        Some(Command::Context) => run_context(),
        None => run_tui().await,
    }
}

fn run_route(task: Vec<String>) -> Result<()> {
    let task = task.join(" ");
    let task = task.trim();
    if task.is_empty() {
        anyhow::bail!("route task must not be empty");
    }
    let plan = routis::route_plan::build_execution_plan(
        task,
        "configs/policies/default.yaml",
        std::env::current_dir()?,
    )?;
    println!("task: {task}");
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
    Ok(())
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
