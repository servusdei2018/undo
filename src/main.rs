mod commands;

use clap::Command;
use commands::{clear, list, revert, run};

fn main() {
    let matches = Command::new("undo")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Nathanael Bracy <https://bracy.dev>")
        .about("Track file modifications from external programs and undo changes")
        .long_about(
            "The `undo` command tracks file modifications made by external programs. You can run programs like text editors,\n\
            and the tool will monitor any changes to files. Later, you can use the `revert` subcommand to undo those changes,\n\
            either for individual files or all tracked modifications.\n\n\
            This allows you to safely edit files or run commands, with the ability to roll back changes if needed."
        )
        .infer_long_args(true)
        .subcommand(clear::get_subcommand())
        .subcommand(list::get_subcommand())
        .subcommand(revert::get_subcommand())
        .subcommand(run::get_subcommand())
        .get_matches();

    match matches.subcommand() {
        Some(("clear", _)) => clear::handle(),
        Some(("list", _)) => list::handle(),
        Some(("revert", sub_m)) => revert::handle(sub_m),
        Some(("run", sub_m)) => run::handle(sub_m),
        _ => {
            eprintln!("Invalid command.");
            std::process::exit(1);
        }
    }
}
