// Copyright (c) Microsoft Corporation. Licensed under the MIT license.
// Ported for security toolkit by diegoakanottheoperator.

//! # LiteBox
//!
//! > A security-focused sandboxing library OS.
//!
//! LiteBox exposes a nix/rustix-inspired interface "above" when it is provided a
//! `Platform` interface "below".
//!
//! This module ports the core platform abstraction layer, sync primitives, and
//! file descriptor table from [microsoft/litebox](https://github.com/microsoft/litebox).

pub mod fd;
pub mod blb1;
mod litebox_core;
pub mod platform;
pub mod host;
pub mod sync;

// Re-export primary types at module level.
pub use fd::{Descriptor, DescriptorKind, Descriptors, RawFd};
pub use litebox_core::LiteBox;
pub use platform::Provider;
pub use host::HostPlatform;
pub use sync::{
    Mutex, MutexGuard, RawSyncPrimitivesProvider, RwLock, RwLockReadGuard, RwLockWriteGuard,
};

use crate::traits::SecuritySandbox;

impl SecuritySandbox for LiteBox<HostPlatform> {
    fn initialize(&self) {
        log::info!("SecuritySandbox: LiteBox initialized.");
    }

    fn get_fd_count(&self) -> usize {
        self.descriptor_table().count()
    }
}

#[cfg(test)]
mod tests {
    use super::platform::*;
    use super::*;
    use std::sync::atomic::AtomicU32;
    use std::time::Duration;

    // =======================================================================
    // Mock Platform
    // =======================================================================

    /// A mock platform for testing, implementing the full Provider supertrait.
    struct MockPlatform;

    /// A mock RawMutex using parking_lot-like spin semantics.
    struct MockRawMutex {
        atomic: AtomicU32,
    }

    impl RawMutex for MockRawMutex {
        const INIT: Self = Self {
            atomic: AtomicU32::new(0),
        };

        fn underlying_atomic(&self) -> &AtomicU32 {
            &self.atomic
        }

        fn wake_many(&self, _n: usize) -> usize {
            // Mock: no actual threads to wake.
            0
        }

        fn block(&self, _val: u32) -> Result<(), ImmediatelyWokenUp> {
            // Mock: spin briefly, then return ImmediatelyWokenUp.
            std::thread::yield_now();
            Err(ImmediatelyWokenUp)
        }

        fn block_or_timeout(
            &self,
            _val: u32,
            _time: Duration,
        ) -> Result<UnblockedOrTimedOut, ImmediatelyWokenUp> {
            std::thread::yield_now();
            Ok(UnblockedOrTimedOut::TimedOut)
        }
    }

    impl RawMutexProvider for MockPlatform {
        type RawMutex = MockRawMutex;
    }

    impl IPInterfaceProvider for MockPlatform {
        fn send_ip_packet(&self, _packet: &[u8]) -> Result<(), SendError> {
            Ok(())
        }
        fn receive_ip_packet(&self, _packet: &mut [u8]) -> Result<usize, ReceiveError> {
            Err(ReceiveError::WouldBlock)
        }
    }

    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct MockInstant(u64);

    impl InstantTrait for MockInstant {
        fn checked_duration_since(&self, earlier: &Self) -> Option<Duration> {
            self.0.checked_sub(earlier.0).map(Duration::from_nanos)
        }
        fn checked_add(&self, duration: Duration) -> Option<Self> {
            self.0
                .checked_add(duration.as_nanos() as u64)
                .map(MockInstant)
        }
    }

    struct MockSystemTime(u64);

    impl SystemTimeTrait for MockSystemTime {
        const UNIX_EPOCH: Self = Self(0);
        fn duration_since(&self, earlier: &Self) -> Result<Duration, Duration> {
            if self.0 >= earlier.0 {
                Ok(Duration::from_secs(self.0 - earlier.0))
            } else {
                Err(Duration::from_secs(earlier.0 - self.0))
            }
        }
    }

    impl TimeProvider for MockPlatform {
        type Instant = MockInstant;
        type SystemTime = MockSystemTime;
        fn now(&self) -> MockInstant {
            MockInstant(0)
        }
        fn current_time(&self) -> MockSystemTime {
            MockSystemTime(0)
        }
    }

    #[derive(Debug)]
    struct MockPunchthrough;
    #[derive(Debug)]
    struct MockPunchthroughError;
    impl std::fmt::Display for MockPunchthroughError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "mock punchthrough error")
        }
    }
    impl std::error::Error for MockPunchthroughError {}

    impl Punchthrough for MockPunchthrough {
        type ReturnSuccess = ();
        type ReturnFailure = MockPunchthroughError;
    }

    impl PunchthroughProvider for MockPlatform {
        type Punchthrough = MockPunchthrough;
        fn get_punchthrough_token(
            &self,
            _punchthrough: &Self::Punchthrough,
        ) -> Option<PunchthroughToken<Self::Punchthrough>> {
            Some(PunchthroughToken::new())
        }
    }

    impl DebugLogProvider for MockPlatform {
        fn debug_log_print(&self, _msg: &str) {}
    }

    impl CrngProvider for MockPlatform {
        fn fill_bytes_crng(&self, buf: &mut [u8]) {
            // Deterministic fill for testing.
            for (i, byte) in buf.iter_mut().enumerate() {
                *byte = (i % 256) as u8;
            }
        }
    }

    impl Provider for MockPlatform {}

    // =======================================================================
    // Tests
    // =======================================================================

    #[test]
    fn test_raw_mutex_init() {
        use std::sync::atomic::Ordering;
        let mutex = MockRawMutex::INIT;
        assert_eq!(mutex.underlying_atomic().load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_descriptor_table_creation() {
        let table = Descriptors::<MockPlatform>::new_from_litebox_creation();
        assert_eq!(table.count(), 3);
        assert_eq!(table.get(RawFd::STDIN).unwrap().kind, DescriptorKind::Stdio);
        assert_eq!(
            table.get(RawFd::STDOUT).unwrap().kind,
            DescriptorKind::Stdio
        );
        assert_eq!(
            table.get(RawFd::STDERR).unwrap().kind,
            DescriptorKind::Stdio
        );
    }

    #[test]
    fn test_descriptor_table_insert_remove() {
        let mut table = Descriptors::<MockPlatform>::new_from_litebox_creation();
        let fd = table.insert(Descriptor::new(DescriptorKind::File));
        assert_eq!(fd.as_raw(), 3); // 0,1,2 are stdio

        assert_eq!(table.count(), 4);
        assert_eq!(table.get(fd).unwrap().kind, DescriptorKind::File);

        let removed = table.remove(fd);
        assert!(removed.is_some());
        assert_eq!(table.count(), 3);
        assert!(table.get(fd).is_none());

        // Next insert should reuse fd 3.
        let fd2 = table.insert(Descriptor::new(DescriptorKind::Socket));
        assert_eq!(fd2.as_raw(), 3);
    }

    #[test]
    fn test_descriptor_close_on_exec() {
        let mut table = Descriptors::<MockPlatform>::new_from_litebox_creation();
        let fd3 = table.insert(Descriptor::with_cloexec(DescriptorKind::File));
        let fd4 = table.insert(Descriptor::new(DescriptorKind::Socket));

        assert_eq!(table.count(), 5);
        table.close_on_exec();
        assert_eq!(table.count(), 4);
        assert!(table.get(fd3).is_none()); // cloexec removed
        assert!(table.get(fd4).is_some()); // non-cloexec kept
    }

    #[test]
    fn test_fd_newtype() {
        let fd = RawFd::new(42);
        assert_eq!(fd.as_raw(), 42);

        let fd_from: RawFd = 7.into();
        let val: i32 = fd_from.into();
        assert_eq!(val, 7);
    }

    #[test]
    fn test_mutex_lock_unlock() {
        let mutex = Mutex::<MockPlatform, _>::new(42u64);
        {
            let mut guard = mutex.lock();
            assert_eq!(*guard, 42);
            *guard = 99;
        }
        {
            let guard = mutex.lock();
            assert_eq!(*guard, 99);
        }
    }

    #[test]
    fn test_mutex_try_lock() {
        let mutex = Mutex::<MockPlatform, _>::new(String::from("hello"));
        let guard = mutex.try_lock();
        assert!(guard.is_some());
        assert_eq!(*guard.unwrap(), "hello");
    }

    #[test]
    fn test_rwlock_read_write() {
        let rwlock = RwLock::<MockPlatform, _>::new(vec![1, 2, 3]);
        {
            let reader = rwlock.read();
            assert_eq!(*reader, vec![1, 2, 3]);
        }
        {
            let mut writer = rwlock.write();
            writer.push(4);
        }
        {
            let reader = rwlock.read();
            assert_eq!(*reader, vec![1, 2, 3, 4]);
        }
    }

    #[test]
    fn test_punchthrough_error_display() {
        let e: PunchthroughError<MockPunchthroughError> = PunchthroughError::Unsupported;
        assert_eq!(
            e.to_string(),
            "attempted to execute unsupported punchthrough"
        );

        let e: PunchthroughError<MockPunchthroughError> =
            PunchthroughError::Unimplemented("gpu_access");
        assert_eq!(
            e.to_string(),
            "punchthrough for `gpu_access` is not implemented"
        );
    }

    #[test]
    fn test_crng_provider() {
        let platform = MockPlatform;
        let mut buf = [0u8; 16];
        platform.fill_bytes_crng(&mut buf);
        // Verify deterministic mock fill.
        for (i, &byte) in buf.iter().enumerate() {
            assert_eq!(byte, (i % 256) as u8);
        }
    }

    #[test]
    fn test_receive_error_display() {
        let e = ReceiveError::WouldBlock;
        assert_eq!(e.to_string(), "Receive operation would block");
    }

    #[test]
    fn test_time_provider() {
        let platform = MockPlatform;
        let instant = platform.now();
        let sys_time = platform.current_time();
        let dur = sys_time
            .duration_since(&MockSystemTime::UNIX_EPOCH)
            .unwrap();
        assert_eq!(dur, Duration::from_secs(0));
        let later = instant.checked_add(Duration::from_nanos(100)).unwrap();
        assert_eq!(
            later.checked_duration_since(&instant),
            Some(Duration::from_nanos(100))
        );
    }
}
