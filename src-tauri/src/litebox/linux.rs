//! Linux-specific LiteBox implementations using namespaces and seccomp.
//! Note: Tauri/WebKitGTK relies on bwrap internally when the sandbox is enabled,
//! but this provides defense-in-depth resource dropping for the main process.

use std::fs;

pub fn apply_linux_sandbox() -> Result<(), String> {
    log::info!("Applying Linux filesystem/capability restrictions");
    
    #[cfg(target_os = "linux")]
    unsafe {
        // 1. Disable core dumps and ptrace attachments (unless root)
        if libc::prctl(libc::PR_SET_DUMPABLE, 0, 0, 0, 0) != 0 {
            log::warn!("LiteBox: Failed to disable dumpable flag (PR_SET_DUMPABLE)");
        } else {
            log::info!("LiteBox: Process dumpable flag disabled (anti-ptrace/core).");
        }

        // 2. Prevent the process or its children from gaining new privileges via setuid/setgid
        if libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) != 0 {
            log::warn!("LiteBox: Failed to set PR_SET_NO_NEW_PRIVS");
        } else {
            log::info!("LiteBox: PR_SET_NO_NEW_PRIVS enforced.");
        }
    }
    
    Ok(())
}

pub fn get_open_fd_count() -> usize {
    match fs::read_dir("/proc/self/fd") {
        Ok(entries) => entries.count(),
        Err(_) => 0,
    }
}
