use clap::{Command};

pub fn get_subcommand() -> clap::Command {
    Command::new("revert")
        .about("Revert the changes made to a file (or all files)")
        .long_about(
            "The `revert` subcommand allows you to undo the changes made to a file by the `run` subcommand.\n\
            You can specify a specific file or use `all` to revert all modified files."
        )
        .arg(
            clap::Arg::new("file")
                .help("The file to revert. Use 'all' to revert all modified files.")
                .required(true)
        )
        .after_help(
            "Example usage:\n\
            $ undo revert somefile.txt\n\
            This will revert the changes made to `myfile.txt` by the `run` subcommand.\n\
            $ undo revert all\n\
            This will revert all modified files."
        )
}


pub fn handle(_matches: &clap::ArgMatches) {
}