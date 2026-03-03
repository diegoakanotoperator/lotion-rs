use super::platform::*;
use std::time::{SystemTime, Instant, Duration};

pub struct HostPlatform;

pub struct HostRawMutex {
    atomic: std::sync::atomic::AtomicU32,
}

impl RawMutex for HostRawMutex {
    const INIT: Self = Self {
        atomic: std::sync::atomic::AtomicU32::new(0),
    };
    fn underlying_atomic(&self) -> &std::sync::atomic::AtomicU32 {
        &self.atomic
    }
    fn wake_many(&self, _n: usize) -> usize { 0 }
    fn block(&self, _val: u32) -> Result<(), ImmediatelyWokenUp> {
        std::thread::yield_now();
        Err(ImmediatelyWokenUp)
    }
    fn block_or_timeout(&self, _val: u32, _time: Duration) -> Result<UnblockedOrTimedOut, ImmediatelyWokenUp> {
        std::thread::yield_now();
        Ok(UnblockedOrTimedOut::TimedOut)
    }
}

impl RawMutexProvider for HostPlatform { type RawMutex = HostRawMutex; }

impl IPInterfaceProvider for HostPlatform {
    fn send_ip_packet(&self, _packet: &[u8]) -> Result<(), SendError> { Ok(()) }
    fn receive_ip_packet(&self, _packet: &mut [u8]) -> Result<usize, ReceiveError> { Err(ReceiveError::WouldBlock) }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct HostInstant(Instant);
impl InstantTrait for HostInstant {
    fn checked_duration_since(&self, earlier: &Self) -> Option<Duration> {
        self.0.checked_duration_since(earlier.0)
    }
    fn checked_add(&self, duration: Duration) -> Option<Self> {
        self.0.checked_add(duration).map(HostInstant)
    }
}

pub struct HostSystemTime(SystemTime);
impl SystemTimeTrait for HostSystemTime {
    const UNIX_EPOCH: Self = Self(SystemTime::UNIX_EPOCH);
    fn duration_since(&self, earlier: &Self) -> Result<Duration, Duration> {
        self.0.duration_since(earlier.0).map_err(|e| e.duration())
    }
}

impl TimeProvider for HostPlatform {
    type Instant = HostInstant;
    type SystemTime = HostSystemTime;
    fn now(&self) -> HostInstant { HostInstant(Instant::now()) }
    fn current_time(&self) -> HostSystemTime { HostSystemTime(SystemTime::now()) }
}

pub struct HostPunchthrough;
#[derive(Debug)]
pub struct HostPunchthroughError;
impl std::fmt::Display for HostPunchthroughError { fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f,"error") } }
impl std::error::Error for HostPunchthroughError {}
impl Punchthrough for HostPunchthrough {
    type ReturnSuccess = ();
    type ReturnFailure = HostPunchthroughError;
}
impl PunchthroughProvider for HostPlatform {
    type Punchthrough = HostPunchthrough;
    fn get_punchthrough_token(&self, _p: &Self::Punchthrough) -> Option<PunchthroughToken<Self::Punchthrough>> { Some(PunchthroughToken::new()) }
}

impl DebugLogProvider for HostPlatform {
    fn debug_log_print(&self, msg: &str) {
        log::debug!("{}", msg);
    }
}

impl CrngProvider for HostPlatform {
    fn fill_bytes_crng(&self, buf: &mut [u8]) {
        use rand::RngCore;
        rand::thread_rng().fill_bytes(buf);
    }
}

impl Provider for HostPlatform {}
