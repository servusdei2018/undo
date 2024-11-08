use clap::{Command};

pub fn get_subcommand() -> clap::Command {
    Command::new("run")
        .about("Run a command while tracking file modifications")
        .long_about(
            "Executes a command with specified arguments.\n\
            Files changed by the command are tracked, and changes can be reverted using the `revert` subcommand."
        )
        .arg(
            clap::arg!([program] "Command to run")
                .required(true)
                .help("Command to execute, e.g., nano, vim, etc.")
        )
        .arg(
            clap::arg!([args]... "Arguments to passed to the command")
                .help("Arguments passed to the specified command.")
        )
}


pub fn handle(_matches: &clap::ArgMatches) {
}