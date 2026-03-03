// Copyright (c) Microsoft Corporation. Licensed under the MIT license.
// Ported for security toolkit by diegoakanottheoperator.

//! The top-level [`LiteBox`] object managing sandbox state.

use std::sync::Arc;

use super::fd::Descriptors;
use super::sync::{RawSyncPrimitivesProvider, RwLock};

/// A full LiteBox sandbox system.
///
/// Manages "global" sandbox state — the file descriptor table and platform reference.
/// For now, we assume synchronization support is a hard requirement.
pub struct LiteBox<Platform: RawSyncPrimitivesProvider> {
    inner: Arc<LiteBoxInner<Platform>>,
}

impl<Platform: RawSyncPrimitivesProvider> LiteBox<Platform> {
    /// Create a new (empty) [`LiteBox`] instance for the given `platform`.
    pub fn new(platform: &'static Platform) -> Self {
        Self {
            inner: Arc::new(LiteBoxInner {
                platform,
                descriptors: RwLock::new(Descriptors::new_from_litebox_creation()),
            }),
        }
    }

    /// Access to the underlying platform.
    pub fn platform(&self) -> &'static Platform {
        self.inner.platform
    }

    /// Access to the file descriptor table (read lock).
    pub fn descriptor_table(&self) -> impl std::ops::Deref<Target = Descriptors<Platform>> + '_ {
        self.inner.descriptors.read()
    }

    /// Mutable access to the file descriptor table (write lock).
    pub fn descriptor_table_mut(
        &self,
    ) -> impl std::ops::DerefMut<Target = Descriptors<Platform>> + '_ {
        self.inner.descriptors.write()
    }
}

impl<Platform: RawSyncPrimitivesProvider> Clone for LiteBox<Platform> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

/// The actual body of [`LiteBox`], containing shared components.
struct LiteBoxInner<Platform: RawSyncPrimitivesProvider> {
    platform: &'static Platform,
    descriptors: RwLock<Platform, Descriptors<Platform>>,
}
