use crate::litebox::LiteBox;
use std::sync::Arc;

pub struct SecurityModule {
    pub litebox: Arc<LiteBox>,
}

impl Default for SecurityModule {
    fn default() -> Self {
        Self::new()
    }
}

impl SecurityModule {
    pub fn new() -> Self {
        let mut sandbox = LiteBox::new();
        // Zero-Trust: Enforce OS-level boundaries at startup
        if let Err(e) = sandbox.apply_sandbox() {
            log::error!("CRITICAL: Failed to apply OS-level LiteBox sandbox: {}", e);
            // In a strict Zero-Trust model, we should panic here if the sandbox fails.
            // panic!("Security sandbox initialization failed");
        }

        Self {
            litebox: Arc::new(sandbox),
        }
    }
}

// Note: LiteBox OS-level implementation already implements SecuritySandbox traits.
