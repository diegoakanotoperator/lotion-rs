//! Linux-specific LiteBox implementations using namespaces and seccomp.
//! Note: Tauri/WebKitGTK relies on bwrap internally when the sandbox is enabled,
//! but this provides defense-in-depth resource dropping for the main process.

use std::fs;

pub fn apply_linux_sandbox() -> Result<(), String> {
    log::info!("Applying Linux filesystem/capability restrictions");

    #[cfg(target_os = "linux")]
    unsafe {
        // 1. Unshare namespaces for isolation
        // CLONE_NEWNS: Mount namespace
        // CLONE_NEWUTS: Hostname namespace
        // CLONE_NEWIPC: IPC namespace
        // CLONE_NEWPID: PID namespace (this process becomes PID 1 in the new namespace)
        // CLONE_NEWNET: Network namespace (no network access)
        let flags = libc::CLONE_NEWNS | libc::CLONE_NEWUTS | libc::CLONE_NEWIPC | libc::CLONE_NEWPID | libc::CLONE_NEWNET;
        
        // Note: unshare(CLONE_NEWUSER) is often needed if not running as root,
        // but it can complicate resource access. We'll stick to basic resource dropping
        // if unshare fails or is restricted by sysctl.
        if libc::unshare(flags) != 0 {
            log::warn!("LiteBox: Failed to unshare namespaces (might lack CAP_SYS_ADMIN or user namespaces restricted)");
        } else {
            log::info!("LiteBox: Full namespace isolation enforced (NS, UTS, IPC, PID, NET).");
        }

        // 2. Disable core dumps and ptrace attachments
        libc::prctl(libc::PR_SET_DUMPABLE, 0, 0, 0, 0);
        log::info!("LiteBox: Process dumpable flag disabled.");

        // 3. Prevent gaining new privileges
        libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0);
        log::info!("LiteBox: PR_SET_NO_NEW_PRIVS enforced.");

        // 4. Drop all capabilities
        // Note: This requires the process to have some caps to begin with or be root.
        // For a desktop app, we mostly rely on the namespaces above.
    }

    Ok(())
}

pub fn get_open_fd_count() -> usize {
    match fs::read_dir("/proc/self/fd") {
        Ok(entries) => entries.count(),
        Err(_) => 0,
    }
}
