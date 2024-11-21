use nix::libc::user_regs_struct;
use nix::sys::ptrace;
use nix::unistd::Pid;
use nix::Error;

/// Retrieves the register state of a process.
pub fn peek(pid: Pid) -> Result<user_regs_struct, Error> {
    #[cfg(target_arch = "x86_64")]
    {
        return ptrace::getregs(pid).map_err(|e| e);
    }

    #[cfg(target_arch = "aarch64")]
    {
        let regset_type = nix::sys::ptrace::RegSet::NT_PRSTATUS;
        let regs = ptrace::getregset(pid, regset_type)?;
        if let Some(regs) = regs {
            let user_regs: user_regs_struct = unsafe { mem::transmute(regs) };
            return Ok(user_regs);
        } else {
            return Err(Error::from(nix::errno::Errno::EPERM));
        }
    }
}
