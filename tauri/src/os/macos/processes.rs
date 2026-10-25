//! The Mac's processes (`platform::processes`) with `libproc`.

use std::path::PathBuf;

use crate::platform::processes::Proc;

/// Every process with its program path (those the user can query).
pub fn all() -> Vec<Proc> {
    // SAFETY: a null buffer asks only for the count.
    let n = unsafe { libc::proc_listallpids(std::ptr::null_mut(), 0) };
    if n <= 0 {
        return Vec::new();
    }
    // Room for processes started in between.
    let mut pids = vec![0 as libc::pid_t; n as usize + 64];
    let bytes = (pids.len() * size_of::<libc::pid_t>()) as libc::c_int;
    // SAFETY: the buffer and its size in bytes describe the same memory.
    let n = unsafe { libc::proc_listallpids(pids.as_mut_ptr().cast(), bytes) };
    pids.truncate(n.max(0) as usize);
    pids.into_iter()
        .filter(|&pid| pid > 0)
        .filter_map(|pid| {
            let pid = pid as u32;
            let path = image(pid)?;
            let mut info: libc::proc_bsdinfo = unsafe { std::mem::zeroed() };
            let size = size_of::<libc::proc_bsdinfo>() as libc::c_int;
            // SAFETY: `info` is the structure PROC_PIDTBSDINFO fills, of `size` bytes.
            let read = unsafe {
                libc::proc_pidinfo(
                    pid as libc::c_int,
                    libc::PROC_PIDTBSDINFO,
                    0,
                    std::ptr::from_mut(&mut info).cast(),
                    size,
                )
            };
            Some(Proc {
                pid,
                parent: if read == size { info.pbi_ppid } else { 0 },
                name: path.file_name()?.to_string_lossy().into_owned(),
                path,
            })
        })
        .collect()
}

/// The program of process `pid`, if the user can query it.
pub fn image(pid: u32) -> Option<PathBuf> {
    use std::os::unix::ffi::OsStrExt;
    let mut buf = vec![0u8; libc::PROC_PIDPATHINFO_MAXSIZE as usize];
    // SAFETY: the buffer and its size describe the same memory.
    let n = unsafe {
        libc::proc_pidpath(
            pid as libc::c_int,
            buf.as_mut_ptr().cast(),
            buf.len() as u32,
        )
    };
    (n > 0).then(|| PathBuf::from(std::ffi::OsStr::from_bytes(&buf[..n as usize])))
}
