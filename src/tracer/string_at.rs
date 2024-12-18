use nix::sys::ptrace;
use nix::unistd::Pid;
use nix::Error;
use std::ffi::c_void;

/// Reads a null-terminated string from the specified memory address in the target process.
pub fn string_at(pid: Pid, addr: u64) -> Result<String, Error> {
    let mut bytes = Vec::new();
    let mut current_addr = addr;

    loop {
        let ptr = current_addr as *mut c_void;
        let word = ptrace::read(pid, ptr).map_err(|e| e)?;

        for i in 0..8 {
            let byte = (word >> (i * 8) & 0xFF) as u8;

            if byte == 0 {
                return String::from_utf8(bytes)
                    .map_err(|_| Error::from(nix::errno::Errno::EINVAL));
            }

            bytes.push(byte);
        }

        current_addr += 8;
    }
}
