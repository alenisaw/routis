#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlashCommand {
    Help,
    Status,
    Setup,
    Config,
    Provider,
    Theme,
    Doctor,
    Clear,
    Context,
    Route,
    PolicyFile,
    History,
    Sessions,
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandSpec {
    pub name: &'static str,
    pub description: &'static str,
    pub shortcut: &'static str,
    pub command: SlashCommand,
}

pub const COMMANDS: [CommandSpec; 14] = [
    CommandSpec {
        name: "/status",
        description: "show current provider and shell state",
        shortcut: "",
        command: SlashCommand::Status,
    },
    CommandSpec {
        name: "/doctor",
        description: "check provider binary, version, auth, and config",
        shortcut: "",
        command: SlashCommand::Doctor,
    },
    CommandSpec {
        name: "/config",
        description: "print local config path and saved settings",
        shortcut: "",
        command: SlashCommand::Config,
    },
    CommandSpec {
        name: "/clear",
        description: "clear current session view",
        shortcut: "Ctrl+L",
        command: SlashCommand::Clear,
    },
    CommandSpec {
        name: "/history",
        description: "show recent prompts from local history",
        shortcut: "Up",
        command: SlashCommand::History,
    },
    CommandSpec {
        name: "/context",
        description: "show repository branch, changed files, and area",
        shortcut: "",
        command: SlashCommand::Context,
    },
    CommandSpec {
        name: "/route",
        description: "preview route decision without execution",
        shortcut: "",
        command: SlashCommand::Route,
    },
    CommandSpec {
        name: "/policy-file",
        description: "set the routing policy file for this shell",
        shortcut: "",
        command: SlashCommand::PolicyFile,
    },
    CommandSpec {
        name: "/sessions",
        description: "resume a previous session",
        shortcut: "",
        command: SlashCommand::Sessions,
    },
    CommandSpec {
        name: "/provider",
        description: "choose provider and run diagnostics",
        shortcut: "",
        command: SlashCommand::Provider,
    },
    CommandSpec {
        name: "/setup",
        description: "open local setup wizard",
        shortcut: "",
        command: SlashCommand::Setup,
    },
    CommandSpec {
        name: "/theme",
        description: "choose theme with live preview",
        shortcut: "",
        command: SlashCommand::Theme,
    },
    CommandSpec {
        name: "/help",
        description: "show commands and shortcuts",
        shortcut: "F1",
        command: SlashCommand::Help,
    },
    CommandSpec {
        name: "/quit",
        description: "exit Routis",
        shortcut: "Ctrl+D",
        command: SlashCommand::Quit,
    },
];

pub fn parse_slash_command(input: &str) -> Result<SlashCommand, String> {
    let command = input
        .split_whitespace()
        .next()
        .ok_or_else(|| "empty command".to_string())?;
    COMMANDS
        .iter()
        .find(|spec| spec.name == command)
        .map(|spec| spec.command)
        .ok_or_else(|| format!("unknown command `{command}`"))
}

pub fn complete_slash_command(input: &str) -> Vec<&'static str> {
    let prefix = input.split_whitespace().next().unwrap_or(input);
    COMMANDS
        .iter()
        .filter(|spec| spec.name.starts_with(prefix))
        .map(|spec| spec.name)
        .collect()
}

pub fn matching_commands(input: &str) -> Vec<CommandSpec> {
    let prefix = input.split_whitespace().next().unwrap_or(input);
    COMMANDS
        .iter()
        .copied()
        .filter(|spec| spec.name.starts_with(prefix))
        .collect()
}
