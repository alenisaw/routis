pub mod app;
pub mod command;
pub mod config;
pub mod history;
pub mod render;
mod screens;
pub mod session;
pub mod state;
pub mod theme;
mod widgets;

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
