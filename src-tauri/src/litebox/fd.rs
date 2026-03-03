// Copyright (c) Microsoft Corporation. Licensed under the MIT license.
// Ported for security toolkit by diegoakanottheoperator.

//! File descriptor table with sandboxing semantics.
//!
//! The [`Descriptors`] table manages file descriptors for the sandbox,
//! providing allocation, lookup, and deallocation of descriptors.

use super::platform::RawMutexProvider;

// ---------------------------------------------------------------------------
// RawFd
// ---------------------------------------------------------------------------

/// A raw file descriptor — newtype over `i32` matching the POSIX convention.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RawFd(i32);

impl RawFd {
    /// Create a new `RawFd` from an `i32`.
    pub fn new(fd: i32) -> Self {
        Self(fd)
    }

    /// Get the raw file descriptor value.
    pub fn as_raw(&self) -> i32 {
        self.0
    }

    /// Standard input.
    pub const STDIN: RawFd = RawFd(0);
    /// Standard output.
    pub const STDOUT: RawFd = RawFd(1);
    /// Standard error.
    pub const STDERR: RawFd = RawFd(2);
}

impl From<i32> for RawFd {
    fn from(fd: i32) -> Self {
        Self(fd)
    }
}

impl From<RawFd> for i32 {
    fn from(fd: RawFd) -> Self {
        fd.0
    }
}

// ---------------------------------------------------------------------------
// Descriptor
// ---------------------------------------------------------------------------

/// The type of file descriptor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DescriptorKind {
    /// A regular file.
    File,
    /// A pipe (read or write end).
    Pipe,
    /// A network socket.
    Socket,
    /// A special device (e.g., /dev/null, /dev/random).
    Special,
    /// Standard I/O (stdin, stdout, stderr).
    Stdio,
}

/// A single file descriptor entry.
#[derive(Clone, Debug)]
pub struct Descriptor {
    /// The kind of descriptor.
    pub kind: DescriptorKind,
    /// Whether this descriptor should be closed on exec.
    pub close_on_exec: bool,
}

impl Descriptor {
    /// Create a new descriptor.
    pub fn new(kind: DescriptorKind) -> Self {
        Self {
            kind,
            close_on_exec: false,
        }
    }

    /// Create a new descriptor with close-on-exec set.
    pub fn with_cloexec(kind: DescriptorKind) -> Self {
        Self {
            kind,
            close_on_exec: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Descriptors table
// ---------------------------------------------------------------------------

/// File descriptor table for the sandbox.
///
/// Manages a growable vector of optional [`Descriptor`] entries, allocating
/// the lowest available index (POSIX semantics).
pub struct Descriptors<P: RawMutexProvider> {
    table: Vec<Option<Descriptor>>,
    _platform: std::marker::PhantomData<P>,
}

impl<P: RawMutexProvider> Descriptors<P> {
    /// Create the initial descriptor table with stdin/stdout/stderr.
    pub fn new_from_litebox_creation() -> Self {
        let mut table = Vec::with_capacity(64);
        // fd 0 = stdin
        table.push(Some(Descriptor::new(DescriptorKind::Stdio)));
        // fd 1 = stdout
        table.push(Some(Descriptor::new(DescriptorKind::Stdio)));
        // fd 2 = stderr
        table.push(Some(Descriptor::new(DescriptorKind::Stdio)));
        Self {
            table,
            _platform: std::marker::PhantomData,
        }
    }

    /// Create an empty descriptor table.
    pub fn new_empty() -> Self {
        Self {
            table: Vec::with_capacity(64),
            _platform: std::marker::PhantomData,
        }
    }

    /// Allocate a new file descriptor using the lowest available index.
    ///
    /// Returns the assigned [`RawFd`].
    pub fn insert(&mut self, descriptor: Descriptor) -> RawFd {
        // Find the lowest available slot.
        for (i, slot) in self.table.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(descriptor);
                return RawFd::new(i as i32);
            }
        }
        // No free slots — push to the end.
        let fd = RawFd::new(self.table.len() as i32);
        self.table.push(Some(descriptor));
        fd
    }

    /// Look up a descriptor by its raw fd.
    pub fn get(&self, fd: RawFd) -> Option<&Descriptor> {
        let idx = fd.as_raw();
        if idx < 0 {
            return None;
        }
        self.table.get(idx as usize).and_then(|slot| slot.as_ref())
    }

    /// Remove and return a descriptor by its raw fd.
    pub fn remove(&mut self, fd: RawFd) -> Option<Descriptor> {
        let idx = fd.as_raw();
        if idx < 0 {
            return None;
        }
        let idx = idx as usize;
        if idx >= self.table.len() {
            return None;
        }
        self.table[idx].take()
    }

    /// Returns the number of open descriptors.
    pub fn count(&self) -> usize {
        self.table.iter().filter(|s| s.is_some()).count()
    }

    /// Returns the total capacity of the table.
    pub fn capacity(&self) -> usize {
        self.table.len()
    }

    /// Close all descriptors marked with close-on-exec.
    pub fn close_on_exec(&mut self) {
        for slot in &mut self.table {
            if let Some(desc) = slot {
                if desc.close_on_exec {
                    *slot = None;
                }
            }
        }
    }
}
