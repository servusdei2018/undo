use clap::{Command};

pub fn get_subcommand() -> clap::Command {
    Command::new("list")
        .about("List all modified files that can be reverted")
        .long_about(
            "The `list` subcommand displays a list of files that have been modified and are currently tracked for undo.\n\
            These files can be reverted to their previous state using the `revert` subcommand."
        )
        .after_help(
            "Examples:\n\
            $ undo list\n\
            Shows all modified files that can be reverted.\n\
            $ undo revert somefile.txt\n\
            Reverts the changes made to 'somefile.txt'."
        )
}

pub fn handle() {
}
