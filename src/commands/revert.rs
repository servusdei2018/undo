use crate::cache::Cache;

use clap;
use std::env;
use std::path::Path;

/// Creates the `revert` subcommand.
pub fn get_subcommand() -> clap::Command {
    clap::Command::new("revert")
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

/// Handles the `revert` subcommand.
pub fn handle(c: &mut Cache, matches: &clap::ArgMatches) {
    let file = matches.get_one::<String>("file").unwrap();

    if file == "all" {
        match c.list() {
            Ok(files) => {
                for file in files {
                    match c.restore(&file) {
                        Ok(_) => println!("Reverted file: {}", file.display()),
                        Err(e) => eprintln!("Error reverting file '{}': {}", file.display(), e),
                    }
                }
            }
            Err(e) => eprintln!("Error retrieving cached changes: {}", e),
        }
    } else {
        let file_path = if Path::new(file).is_absolute() {
            Path::new(file).to_path_buf()
        } else {
            let current_dir = env::current_dir().unwrap();
            current_dir.join(file)
        };

        match c.restore(&file_path) {
            Ok(_) => println!("Reverted file: {}", file_path.display()),
            Err(e) => eprintln!("Error reverting file '{}': {}", file_path.display(), e),
        }
    }
}
