#![forbid(unsafe_code)]
#![deny(warnings)]

use anyhow::Result;
use clap::Parser;
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
}

#[tokio::main]
async fn main() -> Result<()> {
    let _args = Args::parse();
    run_tui().await
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
