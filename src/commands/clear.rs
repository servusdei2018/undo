use clap;

pub fn get_subcommand() -> clap::Command {
    clap::Command::new("clear")
        .about("Clear the history of tracked file modifications")
        .after_help(
            "Removes all records of file modifications that were tracked for undo.\n\
            Use this command with caution as it cannot be undone.",
        )
}

pub fn handle() {
    println!("History cleared.");
}
