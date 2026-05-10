use routis::tui::command::{parse_slash_command, SlashCommand, COMMANDS};

#[test]
fn trace_commands_are_registered() {
    assert!(COMMANDS.iter().any(|command| command.name == "/trace"));
    assert!(COMMANDS.iter().any(|command| command.name == "/traces"));
    assert_eq!(parse_slash_command("/trace").unwrap(), SlashCommand::Trace);
    assert_eq!(parse_slash_command("/traces").unwrap(), SlashCommand::Traces);
}
