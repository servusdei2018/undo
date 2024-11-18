use clap;
use nix::sys::ptrace;
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::Pid;
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
    let program = matches
        .get_one::<String>("program")
        .unwrap();
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
        Ok(child_process) => {
            let child_pid = Pid::from_raw(child_process.id() as i32);
            ptrace::attach(child_pid).unwrap();
            loop {
                let status = waitpid(child_pid, None).unwrap();
                if let WaitStatus::Stopped(pid, _) = status {
                        let regs = ptrace::getregs(pid).unwrap();
                        let _syscall_number = regs.orig_rax;
                        ptrace::syscall(child_pid, None).unwrap();
                }
                if let WaitStatus::Exited(_, _) = status {
                    break;
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to spawn command: {}", e);
            process::exit(1);
        }
    }
}
