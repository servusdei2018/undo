use clap;
use nix::sys::ptrace;
use nix::sys::signal::{raise, Signal};
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::fork;
use nix::unistd::ForkResult::*;
use std::process;

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

pub fn handle(matches: &clap::ArgMatches) {
    match unsafe { fork() }.expect("Error: Fork Failed") {
        Child => {
            ptrace::traceme().unwrap();
            // As recommended by ptrace(2), raise SIGTRAP to pause the child until the parent is ready to continue
            raise(Signal::SIGTRAP).unwrap();

            let program = matches
                .get_one::<String>("program")
                .expect("Program is required");
            let args = matches
                .get_many::<String>("args")
                .map(|s| s.collect::<Vec<_>>())
                .unwrap_or_default();

            let mut cmd = process::Command::new(program);
            cmd.args(args);
            cmd.stdin(std::process::Stdio::inherit())
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit());

            match cmd.spawn() {
                Ok(mut child) => {
                    let exit_status = child.wait().unwrap();
                    process::exit(exit_status.code().unwrap_or(1));
                }
                Err(e) => {
                    eprintln!("Failed to run command: {}", e);
                    process::exit(1);
                }
            }
        }
        Parent { child } => {
            assert_eq!(
                waitpid(child, None),
                Ok(WaitStatus::Stopped(child, Signal::SIGTRAP))
            );
            ptrace::cont(child, None).unwrap();
            assert_eq!(
                waitpid(child, None),
                Ok(WaitStatus::Stopped(child, Signal::SIGTRAP))
            );
            ptrace::cont(child, Some(Signal::SIGKILL)).unwrap();
            match waitpid(child, None) {
                Ok(WaitStatus::Exited(pid, exit_code)) if pid == child => {
                    process::exit(exit_code);
                }
                Ok(WaitStatus::Signaled(pid, signal, _)) if pid == child => {
                    eprintln!("Child process {} was killed by signal: {:?}", child, signal);
                    process::exit(1);
                }
                Ok(status) => {
                    eprintln!("Unexpected wait status: {:?}", status);
                    process::exit(1);
                }
                Err(e) => {
                    eprintln!("Error waiting for child process: {}", e);
                    process::exit(1);
                }
            }
        }
    }
}
