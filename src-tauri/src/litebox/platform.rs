// Copyright (c) Microsoft Corporation. Licensed under the MIT license.
// Ported for security toolkit by diegoakanottheoperator.

//! The underlying platform upon which LiteBox resides.
//!
//! The top-level trait that denotes something is a valid LiteBox platform is [`Provider`].
//! This trait is a composition of subtraits that can be implemented independently.

use core::fmt;
use core::time::Duration;

// ---------------------------------------------------------------------------
// Provider supertrait
// ---------------------------------------------------------------------------

/// A provider of a platform upon which LiteBox can execute.
///
/// Ideally, a `Provider` is zero-sized and only exists to provide access to functionality.
/// Most APIs act upon `&self` to allow storage of useful "globals" within it.
pub trait Provider:
    RawMutexProvider
    + IPInterfaceProvider
    + TimeProvider
    + PunchthroughProvider
    + DebugLogProvider
    + CrngProvider
    + Send
    + Sync
    + 'static
{
}

// ---------------------------------------------------------------------------
// Raw Mutex Provider
// ---------------------------------------------------------------------------

/// A provider of raw mutexes (futex-like blocking primitive).
pub trait RawMutexProvider {
    /// The raw mutex type for this platform.
    type RawMutex: RawMutex;
}

/// Zero-sized struct indicating that the block was immediately unblocked
/// (due to non-matching value).
pub struct ImmediatelyWokenUp;

/// Named-boolean to indicate whether [`RawMutex::block_or_timeout`] was woken up or timed out.
#[must_use]
pub enum UnblockedOrTimedOut {
    /// Unblocked by a wake call.
    Unblocked,
    /// Sufficient time elapsed without a wake call.
    TimedOut,
}

/// A raw mutex/lock API; expected to roughly match (or even be implemented using) a Linux futex.
pub trait RawMutex: Send + Sync + 'static {
    /// The initial value for a raw mutex, with an underlying atomic value of zero.
    const INIT: Self;

    /// Returns a reference to the underlying atomic value.
    fn underlying_atomic(&self) -> &core::sync::atomic::AtomicU32;

    /// Wake up `n` threads blocked on this raw mutex.
    ///
    /// Returns the number of waiters that were woken up.
    fn wake_many(&self, n: usize) -> usize;

    /// Wake up one thread blocked on this raw mutex.
    ///
    /// Returns true if this actually woke up such a thread.
    fn wake_one(&self) -> bool {
        self.wake_many(1) > 0
    }

    /// Wake up all threads that are blocked on this raw mutex.
    ///
    /// Returns the number of waiters that were woken up.
    fn wake_all(&self) -> usize {
        self.wake_many(i32::MAX as usize)
    }

    /// If the underlying value is `val`, block until a wake operation wakes us up.
    fn block(&self, val: u32) -> Result<(), ImmediatelyWokenUp>;

    /// If the underlying value is `val`, block until a wake operation wakes us up,
    /// or some `time` has passed without a wake operation having occurred.
    fn block_or_timeout(
        &self,
        val: u32,
        time: Duration,
    ) -> Result<UnblockedOrTimedOut, ImmediatelyWokenUp>;
}

// ---------------------------------------------------------------------------
// IP Interface Provider
// ---------------------------------------------------------------------------

/// An IP packet interface to the outside world (e.g., via a TUN device).
pub trait IPInterfaceProvider {
    /// Send the IP packet.
    fn send_ip_packet(&self, packet: &[u8]) -> Result<(), SendError>;

    /// Receive an IP packet into `packet`.
    ///
    /// Returns size of packet received.
    fn receive_ip_packet(&self, packet: &mut [u8]) -> Result<usize, ReceiveError>;
}

/// Errors for [`IPInterfaceProvider::send_ip_packet`].
#[derive(Debug)]
#[non_exhaustive]
pub enum SendError {}

impl fmt::Display for SendError {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // No variants currently
        Ok(())
    }
}

impl std::error::Error for SendError {}

/// Errors for [`IPInterfaceProvider::receive_ip_packet`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ReceiveError {
    /// Receive operation would block.
    WouldBlock,
}

impl fmt::Display for ReceiveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReceiveError::WouldBlock => write!(f, "Receive operation would block"),
        }
    }
}

impl std::error::Error for ReceiveError {}

// ---------------------------------------------------------------------------
// Time Provider
// ---------------------------------------------------------------------------

/// An interface to understanding time.
pub trait TimeProvider {
    /// The monotonic instant type.
    type Instant: InstantTrait;
    /// The wall-clock system time type.
    type SystemTime: SystemTimeTrait;

    /// Returns an instant corresponding to "now".
    fn now(&self) -> Self::Instant;

    /// Returns the current system time.
    fn current_time(&self) -> Self::SystemTime;
}

/// An opaque measurement of a monotonically nondecreasing clock.
///
/// Distinct from [`SystemTimeTrait`] — this is monotonic and need not relate to "real" time.
pub trait InstantTrait: Copy + Clone + PartialEq + Eq + PartialOrd + Ord + Send + Sync {
    /// Returns the duration elapsed from `earlier` to `self`, or `None` if `earlier` is later.
    fn checked_duration_since(&self, earlier: &Self) -> Option<Duration>;

    /// Returns the duration elapsed from `earlier` to `self`, or zero if `earlier` is later.
    fn duration_since(&self, earlier: &Self) -> Duration {
        self.checked_duration_since(earlier)
            .unwrap_or(Duration::from_secs(0))
    }

    /// Returns a new `Instant` that is the sum of this instant and the provided duration.
    fn checked_add(&self, duration: Duration) -> Option<Self>;
}

/// A measurement of the system clock (wall-clock time).
///
/// Distinct from [`InstantTrait`] — may not be monotonic, but represents "real" time.
pub trait SystemTimeTrait: Send + Sync {
    /// An anchor in time corresponding to "1970-01-01 00:00:00 UTC".
    const UNIX_EPOCH: Self;

    /// Returns the duration since `earlier`, or `Err(abs_duration)` if the clock adjusted backwards.
    fn duration_since(&self, earlier: &Self) -> Result<Duration, Duration>;
}

// ---------------------------------------------------------------------------
// Punchthrough Provider (Zero-Trust Auditability)
// ---------------------------------------------------------------------------

/// Punch through any functionality for a particular platform that is not explicitly part
/// of the common shared platform interface.
///
/// Improves auditability: all host invocations passing through the punchthrough are
/// auditable; anything circumventing it is suspicious.
pub trait PunchthroughProvider {
    /// The punchthrough type for this platform.
    type Punchthrough: Punchthrough;

    /// Request permission token to invoke a punchthrough.
    fn get_punchthrough_token(
        &self,
        punchthrough: &Self::Punchthrough,
    ) -> Option<PunchthroughToken<Self::Punchthrough>>;
}

/// Punchthrough support allowing access to functionality not captured by [`Provider`].
pub trait Punchthrough {
    /// The success return type.
    type ReturnSuccess;
    /// The failure return type.
    type ReturnFailure: std::error::Error;
}

/// A token that demonstrates the platform is allowing access for a particular punchthrough.
pub struct PunchthroughToken<P: Punchthrough + ?Sized> {
    _marker: std::marker::PhantomData<P>,
}

impl<P: Punchthrough> PunchthroughToken<P> {
    /// Create a new punchthrough token (platform internal use).
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

/// Possible errors for a [`Punchthrough`].
#[derive(Debug)]
pub enum PunchthroughError<E: std::error::Error> {
    /// Attempted to execute unsupported punchthrough.
    Unsupported,
    /// Punchthrough for a feature is not implemented.
    Unimplemented(&'static str),
    /// Underlying failure.
    Failure(E),
}

impl<E: std::error::Error> fmt::Display for PunchthroughError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PunchthroughError::Unsupported => {
                write!(f, "attempted to execute unsupported punchthrough")
            }
            PunchthroughError::Unimplemented(name) => {
                write!(f, "punchthrough for `{name}` is not implemented")
            }
            PunchthroughError::Failure(e) => write!(f, "{e}"),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for PunchthroughError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PunchthroughError::Failure(e) => Some(e),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Debug Log Provider
// ---------------------------------------------------------------------------

/// An interface to dumping debug output for tracing / audit purposes.
pub trait DebugLogProvider {
    /// Print `msg` to the debug log.
    ///
    /// Newlines are *not* automatically appended.
    fn debug_log_print(&self, msg: &str);
}

// ---------------------------------------------------------------------------
// CRNG Provider
// ---------------------------------------------------------------------------

/// A provider of cryptographically-secure random data.
///
/// This must be infallible — no possibility of failure, blocking, or returning
/// low-quality randomness. The implementation must ensure the CRNG is appropriately
/// initialized and seeded.
pub trait CrngProvider {
    /// Fill `buf` with cryptographically secure random bytes.
    ///
    /// # Panics
    ///
    /// Panics if unable to fill the buffer. This is considered a fatal error.
    fn fill_bytes_crng(&self, buf: &mut [u8]);
}

// ---------------------------------------------------------------------------
// Stdio Provider
// ---------------------------------------------------------------------------

/// Possible standard output/error streams.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StdioOutStream {
    /// Standard output.
    Stdout,
    /// Standard error.
    Stderr,
}

/// Possible standard input/output streams.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StdioStream {
    /// Standard input.
    Stdin = 0,
    /// Standard output.
    Stdout = 1,
    /// Standard error.
    Stderr = 2,
}

/// Errors for [`StdioProvider::read_from_stdin`].
#[derive(Debug)]
#[non_exhaustive]
pub enum StdioReadError {
    /// Input stream has been closed.
    Closed,
}

impl fmt::Display for StdioReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StdioReadError::Closed => write!(f, "input stream has been closed"),
        }
    }
}

impl std::error::Error for StdioReadError {}

/// Errors for [`StdioProvider::write_to`].
#[derive(Debug)]
#[non_exhaustive]
pub enum StdioWriteError {
    /// Output stream has been closed.
    Closed,
}

impl fmt::Display for StdioWriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StdioWriteError::Closed => write!(f, "output stream has been closed"),
        }
    }
}

impl std::error::Error for StdioWriteError {}

/// A provider of standard input/output functionality.
pub trait StdioProvider {
    /// Read from standard input. Returns number of bytes read.
    fn read_from_stdin(&self, buf: &mut [u8]) -> Result<usize, StdioReadError>;

    /// Write to stdout/stderr. Returns number of bytes written.
    fn write_to(&self, stream: StdioOutStream, buf: &[u8]) -> Result<usize, StdioWriteError>;

    /// Check if a stream is connected to a TTY.
    fn is_a_tty(&self, stream: StdioStream) -> bool;
}
