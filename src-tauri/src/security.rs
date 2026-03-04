use std::sync::Arc;
use crate::litebox::LiteBox;
use crate::litebox::host::HostPlatform;
use crate::traits::SecuritySandbox;

pub struct SecurityModule {
    pub litebox: Arc<LiteBox<HostPlatform>>,
}

impl SecurityModule {
    pub fn new() -> Self {
        static PLATFORM: HostPlatform = HostPlatform;
        let litebox = Arc::new(LiteBox::new(&PLATFORM));
        Self { litebox }
    }
}

impl SecuritySandbox for SecurityModule {
    fn initialize(&self) {
        log::info!("SecurityModule: Initializing sandbox...");
    }

    fn get_fd_count(&self) -> usize {
        self.litebox.descriptor_table().count()
    }
}
