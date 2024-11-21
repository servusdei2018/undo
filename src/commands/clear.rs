use crate::cache::Cache;

use clap;

/// Creates the `clear` subcommand.
pub fn get_subcommand() -> clap::Command {
    clap::Command::new("clear")
        .about("Clear the history of tracked file modifications")
        .after_help(
            "Removes all records of file modifications that were tracked for undo.\n\
            Use this command with caution as it cannot be undone.",
        )
}

/// Handles the `clear` subcommand.
pub fn handle(c: &mut Cache) {
    match c.clear() {
        Ok(_) => {
            println!("History cleared.");
        }
        Err(e) => {
            eprintln!("Error clearing history: {}", e);
        }
    }
}
