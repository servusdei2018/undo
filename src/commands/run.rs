use crate::cache::Cache;
use crate::tracer;

use clap;
use nix::sys::ptrace;
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::Pid;
use std::process;

/// Creates the `run` subcommand.
pub fn get_subcommand() -> clap::Command {
    clap::Command::new("run")
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

/// Handles the `run` subcommand.
pub fn handle(_c: &Cache, matches: &clap::ArgMatches) {
    match process::Command::new(matches.get_one::<String>("program").unwrap())
        .args(
            matches
                .get_many::<String>("args")
                .map(|s| s.collect::<Vec<_>>())
                .unwrap_or_default(),
        )
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn()
    {
        Ok(child_process) => {
            let child_pid = Pid::from_raw(child_process.id() as i32);
            ptrace::attach(child_pid).unwrap();
            loop {
                match waitpid(child_pid, None).unwrap() {
                    WaitStatus::Stopped(pid, _) => {
                        if let Ok(path) = tracer::sniff(pid) {
                            println!("Path: {}", path);
                        }
                        ptrace::syscall(pid, None).unwrap();
                    }
                    WaitStatus::Exited(_, _) => {
                        break;
                    }
                    _ => {}
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to spawn command: {}", e);
            process::exit(1);
        }
    }
}
