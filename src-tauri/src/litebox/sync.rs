// Copyright (c) Microsoft Corporation. Licensed under the MIT license.
// Ported for security toolkit by diegoakanottheoperator.

//! Synchronization primitives built on top of the platform's [`RawMutex`].
//!
//! Provides [`Mutex`] and [`RwLock`] types parameterized over the platform.

use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::Ordering;

use super::platform::{RawMutex, RawMutexProvider};

/// Marker supertrait collecting all sync-related requirements.
pub trait RawSyncPrimitivesProvider: RawMutexProvider + Send + Sync + 'static {}
impl<T: RawMutexProvider + Send + Sync + 'static> RawSyncPrimitivesProvider for T {}

// ===========================================================================
// Mutex
// ===========================================================================

const MUTEX_UNLOCKED: u32 = 0;
const MUTEX_LOCKED: u32 = 1;
const MUTEX_LOCKED_CONTENDED: u32 = 2;

/// A mutual exclusion lock backed by the platform's [`RawMutex`].
pub struct Mutex<P: RawMutexProvider, T: ?Sized> {
    raw: P::RawMutex,
    data: UnsafeCell<T>,
}

// Safety: Mutex provides exclusive access; T only needs Send.
unsafe impl<P: RawMutexProvider, T: ?Sized + Send> Send for Mutex<P, T> {}
unsafe impl<P: RawMutexProvider, T: ?Sized + Send> Sync for Mutex<P, T> {}

impl<P: RawMutexProvider, T> Mutex<P, T> {
    /// Creates a new mutex wrapping `value`.
    pub fn new(value: T) -> Self {
        Self {
            raw: P::RawMutex::INIT,
            data: UnsafeCell::new(value),
        }
    }

    /// Consumes the mutex and returns the inner value.
    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }
}

impl<P: RawMutexProvider, T: ?Sized> Mutex<P, T> {
    /// Acquire the mutex, blocking until it is available.
    pub fn lock(&self) -> MutexGuard<'_, P, T> {
        // Fast path: try to acquire the lock.
        let atomic = self.raw.underlying_atomic();
        if atomic
            .compare_exchange(
                MUTEX_UNLOCKED,
                MUTEX_LOCKED,
                Ordering::Acquire,
                Ordering::Relaxed,
            )
            .is_ok()
        {
            return MutexGuard { mutex: self };
        }

        // Slow path: mark as contended and block.
        self.lock_slow();
        MutexGuard { mutex: self }
    }

    #[cold]
    fn lock_slow(&self) {
        let atomic = self.raw.underlying_atomic();
        loop {
            // Try to transition to locked state.
            let prev = atomic.swap(MUTEX_LOCKED_CONTENDED, Ordering::Acquire);
            if prev == MUTEX_UNLOCKED {
                return;
            }

            // Block until woken. Ignore ImmediatelyWokenUp — just retry.
            let _ = self.raw.block(MUTEX_LOCKED_CONTENDED);
        }
    }

    /// Try to acquire the mutex without blocking.
    ///
    /// Returns `None` if the lock is already held.
    pub fn try_lock(&self) -> Option<MutexGuard<'_, P, T>> {
        let atomic = self.raw.underlying_atomic();
        if atomic
            .compare_exchange(
                MUTEX_UNLOCKED,
                MUTEX_LOCKED,
                Ordering::Acquire,
                Ordering::Relaxed,
            )
            .is_ok()
        {
            Some(MutexGuard { mutex: self })
        } else {
            None
        }
    }
}

/// RAII guard for [`Mutex`].
pub struct MutexGuard<'a, P: RawMutexProvider, T: ?Sized> {
    mutex: &'a Mutex<P, T>,
}

impl<P: RawMutexProvider, T: ?Sized> Deref for MutexGuard<'_, P, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<P: RawMutexProvider, T: ?Sized> DerefMut for MutexGuard<'_, P, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<P: RawMutexProvider, T: ?Sized> Drop for MutexGuard<'_, P, T> {
    fn drop(&mut self) {
        let atomic = self.mutex.raw.underlying_atomic();
        // If we were the only locker, just unlock. If contended, wake one waiter.
        if atomic.swap(MUTEX_UNLOCKED, Ordering::Release) == MUTEX_LOCKED_CONTENDED {
            self.mutex.raw.wake_one();
        }
    }
}

// ===========================================================================
// RwLock
// ===========================================================================

// State encoding: bits 0..30 = reader count, bit 31 = writer flag
const RWLOCK_WRITER_BIT: u32 = 1 << 31;

/// A reader-writer lock backed by the platform's [`RawMutex`].
pub struct RwLock<P: RawMutexProvider, T: ?Sized> {
    raw: P::RawMutex,
    data: UnsafeCell<T>,
}

unsafe impl<P: RawMutexProvider, T: ?Sized + Send> Send for RwLock<P, T> {}
unsafe impl<P: RawMutexProvider, T: ?Sized + Send + Sync> Sync for RwLock<P, T> {}

impl<P: RawMutexProvider, T> RwLock<P, T> {
    /// Creates a new reader-writer lock wrapping `value`.
    pub fn new(value: T) -> Self {
        Self {
            raw: P::RawMutex::INIT,
            data: UnsafeCell::new(value),
        }
    }

    /// Consumes the lock and returns the inner value.
    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }
}

impl<P: RawMutexProvider, T: ?Sized> RwLock<P, T> {
    /// Acquire a read lock.
    pub fn read(&self) -> RwLockReadGuard<'_, P, T> {
        let atomic = self.raw.underlying_atomic();
        loop {
            let state = atomic.load(Ordering::Relaxed);
            // If no writer is holding/requesting the lock, try to increment reader count.
            if state & RWLOCK_WRITER_BIT == 0 {
                if atomic
                    .compare_exchange_weak(state, state + 1, Ordering::Acquire, Ordering::Relaxed)
                    .is_ok()
                {
                    return RwLockReadGuard { rwlock: self };
                }
            } else {
                // A writer is active; block until woken.
                let _ = self.raw.block(state);
            }
        }
    }

    /// Acquire a write lock.
    pub fn write(&self) -> RwLockWriteGuard<'_, P, T> {
        let atomic = self.raw.underlying_atomic();
        loop {
            let state = atomic.load(Ordering::Relaxed);
            // If entirely free (no readers, no writer), acquire.
            if state == 0 {
                if atomic
                    .compare_exchange_weak(
                        0,
                        RWLOCK_WRITER_BIT,
                        Ordering::Acquire,
                        Ordering::Relaxed,
                    )
                    .is_ok()
                {
                    return RwLockWriteGuard { rwlock: self };
                }
            } else {
                // Readers or writer active; block.
                let _ = self.raw.block(state);
            }
        }
    }
}

/// RAII guard for read access on [`RwLock`].
pub struct RwLockReadGuard<'a, P: RawMutexProvider, T: ?Sized> {
    rwlock: &'a RwLock<P, T>,
}

impl<P: RawMutexProvider, T: ?Sized> Deref for RwLockReadGuard<'_, P, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.rwlock.data.get() }
    }
}

impl<P: RawMutexProvider, T: ?Sized> Drop for RwLockReadGuard<'_, P, T> {
    fn drop(&mut self) {
        let atomic = self.rwlock.raw.underlying_atomic();
        let prev = atomic.fetch_sub(1, Ordering::Release);
        // If we were the last reader, wake any waiting writers.
        if prev == 1 {
            self.rwlock.raw.wake_one();
        }
    }
}

/// RAII guard for write access on [`RwLock`].
pub struct RwLockWriteGuard<'a, P: RawMutexProvider, T: ?Sized> {
    rwlock: &'a RwLock<P, T>,
}

impl<P: RawMutexProvider, T: ?Sized> Deref for RwLockWriteGuard<'_, P, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.rwlock.data.get() }
    }
}

impl<P: RawMutexProvider, T: ?Sized> DerefMut for RwLockWriteGuard<'_, P, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.rwlock.data.get() }
    }
}

impl<P: RawMutexProvider, T: ?Sized> Drop for RwLockWriteGuard<'_, P, T> {
    fn drop(&mut self) {
        let atomic = self.rwlock.raw.underlying_atomic();
        atomic.store(0, Ordering::Release);
        // Wake all waiters so readers and writers can re-check state.
        self.rwlock.raw.wake_all();
    }
}
