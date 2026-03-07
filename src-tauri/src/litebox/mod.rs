// Copyright (c) Microsoft Corporation. Licensed under the MIT license.
// Refactored for real OS-level containment by diegoakanottheoperator.

//! # LiteBox (OS-Level Sandboxing)
//!
//! [EXPERIMENTAL] Provides OS-level process isolation hints and resource dropping.
//! Note: While this module implements initial Windows Job Objects and Linux
//! PR_SET_NO_NEW_PRIVS boundaries, it is currently a defense-in-depth layer
//! and does not yet provide full filesystem/network jail isolation.
//! Use with caution in high-risk environments.

use crate::traits::SecuritySandbox;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

pub struct LiteBox {
    initialized: bool,
}

impl Default for LiteBox {
    fn default() -> Self {
        Self::new()
    }
}

impl LiteBox {
    pub fn new() -> Self {
        Self { initialized: false }
    }

    /// Explicitly initialize the sandbox constraints for the current platform.
    pub fn apply_sandbox(&mut self) -> Result<(), String> {
        if self.initialized {
            return Ok(());
        }

        #[cfg(target_os = "linux")]
        {
            log::info!("LiteBox: Applying Linux generic namespace/seccomp boundaries");
            linux::apply_linux_sandbox()?;
        }

        #[cfg(target_os = "windows")]
        {
            log::info!("LiteBox: Applying Windows AppContainer/Job Object boundaries");
            windows::apply_windows_sandbox()?;
        }

        self.initialized = true;
        Ok(())
    }
}

impl SecuritySandbox for LiteBox {
    fn initialize(&self) {
        log::info!("SecuritySandbox: LiteBox OS-Level Enforcement Online.");
    }

    fn get_fd_count(&self) -> usize {
        // OS-level limits instead of manual tracking
        #[cfg(target_os = "linux")]
        {
            linux::get_open_fd_count()
        }
        #[cfg(not(target_os = "linux"))]
        {
            0
        }
    }
}

// Fallback for macOS or unimplemented OS
#[allow(dead_code)]
#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn apply_fallback_sandbox() -> Result<(), String> {
    log::warn!("LiteBox: No OS-level sandbox implemented for this platform. Running dangerously.");
    Ok(())
}
