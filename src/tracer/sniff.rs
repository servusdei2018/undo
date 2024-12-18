use crate::tracer::peek;
use crate::tracer::string_at;

use nix::errno::Errno;
use nix::libc;
use nix::unistd::Pid;
use std::ffi::c_ulonglong;
use std::fmt;
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Change {
    Created(String),
    Deleted(String),
    Changed(String),
}

impl fmt::Display for Change {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Change::Created(path) => write!(f, "File created: {}", path),
            Change::Deleted(path) => write!(f, "File deleted: {}", path),
            Change::Changed(path) => write!(f, "File changed: {}", path),
        }
    }
}

pub fn sniff(pid: Pid) -> Result<Change, nix::Error> {
    let regs = peek(pid).unwrap();
    let syscall: c_ulonglong;
    #[cfg(target_arch = "x86_64")]
    {
        syscall = regs.orig_rax;
    }
    #[cfg(target_arch = "aarch64")]
    {
        syscall = regs.regs[8];
    }

    let change: Change = match syscall {
        2 => {
            // open
            #[cfg(target_arch = "x86_64")]
            {
                Change::Changed(string_at(pid, regs.rdi)?)
            }
            #[cfg(target_arch = "aarch64")]
            {
                Change::Changed(string_at(pid, regs.regs[0])?)
            }
        }
        76 | 77 => return Ok(Change::Changed(syscall.to_string())), // truncate, ftruncate
        80 | 81 => return Ok(Change::Changed(syscall.to_string())), // chdir, fchdir
        83 | 84 => return Ok(Change::Changed(syscall.to_string())), // mkdir, rmdir
        85 => {
            // creat
            #[cfg(target_arch = "x86_64")]
            {
                Change::Created(string_at(pid, regs.rdi)?)
            }
            #[cfg(target_arch = "aarch64")]
            {
                Change::Created(string_at(pid, regs.regs[0])?)
            }
        }
        86 => return Ok(Change::Changed(syscall.to_string())), // link
        87 => {
            // unlink
            #[cfg(target_arch = "x86_64")]
            {
                Change::Deleted(string_at(pid, regs.rdi)?)
            }
            #[cfg(target_arch = "aarch64")]
            {
                Change::Deleted(string_at(pid, regs.regs[0])?)
            }
        }
        88 => return Ok(Change::Changed(syscall.to_string())), // symlink
        90..95 => return Ok(Change::Changed(syscall.to_string())), // chmod, fchmod, chown, fchown, lchown, umask
        133 => return Ok(Change::Changed(syscall.to_string())), // mknod
        161 => return Ok(Change::Changed(syscall.to_string())), // chroot
        188..199 => return Ok(Change::Changed(syscall.to_string())), // xattr stuff
        257 => {
            // openat
            #[cfg(target_arch = "x86_64")]
            {
                let dirfd = regs.rdi;
                let pathname = string_at(pid, regs.rsi)?;

                let full_path = if dirfd == libc::AT_FDCWD as u64 {
                    // If dirfd is AT_FDCWD, treat pathname as relative to the current working directory
                    let cwd = resolve_cwd(pid)?;
                    cwd.join(pathname)
                } else {
                    // If dirfd is not AT_FDCWD, use it to resolve the path
                    let dir_path = resolve_dirfd_to_path(pid, dirfd)?;
                    dir_path.join(pathname)
                };

                Change::Changed(full_path.to_str().unwrap_or("").to_string())
            }
            #[cfg(target_arch = "aarch64")]
            {
                let dirfd = regs.regs[0];
                let pathname = string_at(pid, regs.regs[1])?;

                let full_path = if dirfd == libc::AT_FDCWD as u64 {
                    // If dirfd is AT_FDCWD, treat pathname as relative to the current working directory
                    let cwd = resolve_cwd(pid)?;
                    cwd.join(pathname)
                } else {
                    // If dirfd is not AT_FDCWD, use it to resolve the path
                    let dir_path = resolve_dirfd_to_path(pid, dirfd)?;
                    dir_path.join(pathname)
                };

                Change::Changed(full_path.to_str().unwrap_or("").to_string())
            }
        }
        258..261 | 263..269 => return Ok(Change::Changed(syscall.to_string())), // mkdirat and friends
        437 => return Ok(Change::Changed(syscall.to_string())), // openat2
        _ => return Err(nix::Error::from(nix::errno::Errno::EINVAL)),
    };

    Ok(change)
}

/// Resolves the current working directory of a traced process.
fn resolve_cwd(pid: Pid) -> Result<PathBuf, nix::Error> {
    let cwd_path = format!("/proc/{}/cwd", pid);
    match fs::read_link(cwd_path) {
        Ok(path) => Ok(path),
        Err(err) => Err(Errno::from_raw(err.raw_os_error().unwrap_or(libc::EINVAL))),
    }
}

/// Resolves a directory file descriptor to an actual directory path.
fn resolve_dirfd_to_path(pid: Pid, dirfd: c_ulonglong) -> Result<PathBuf, nix::Error> {
    let dirfd_path = format!("/proc/{}/fd/{}", pid, dirfd);
    match fs::read_link(&dirfd_path) {
        Ok(path) => Ok(path),
        Err(err) => Err(nix::Error::from_raw(err.raw_os_error().unwrap_or(libc::EINVAL))),
    }
}