// Copyright 2026 The Atrinik Project
// SPDX-License-Identifier: MIT

#![forbid(unsafe_code)]

use atrinik_scene::{Digest256, ResourceId};
use std::{fmt, future::Future, pin::Pin};

pub const DEFAULT_MAXIMUM_RESOURCE_BYTES: usize = 16 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceRequest {
    pub id: ResourceId,
    pub revision: u64,
    pub digest: Digest256,
    pub maximum_bytes: usize,
}

impl ResourceRequest {
    /// Checks identity and allocation bounds.
    ///
    /// # Errors
    /// Returns [`Error::InvalidRequest`] for an empty identity or unsafe bound.
    pub fn validate(self) -> Result<Self, Error> {
        if self.id.0 == 0
            || self.revision == 0
            || self.digest.0.iter().all(|byte| *byte == 0)
            || self.maximum_bytes == 0
            || self.maximum_bytes > DEFAULT_MAXIMUM_RESOURCE_BYTES
        {
            return Err(Error::InvalidRequest);
        }
        Ok(self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceBlob {
    bytes: Vec<u8>,
}

impl ResourceBlob {
    /// Constructs a nonempty blob within the caller's bound.
    ///
    /// # Errors
    /// Returns [`Error::LimitExceeded`] when the byte count is zero or too large.
    pub fn new(bytes: Vec<u8>, maximum_bytes: usize) -> Result<Self, Error> {
        if bytes.is_empty() || bytes.len() > maximum_bytes {
            return Err(Error::LimitExceeded);
        }
        Ok(Self { bytes })
    }

    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

pub type ResourceFuture<'a> =
    Pin<Box<dyn Future<Output = Result<ResourceBlob, Error>> + Send + 'a>>;

/// Supplies immutable, content-addressed bytes without granting renderer code
/// ambient filesystem or network access.
pub trait ResourceProvider: Send + Sync {
    fn load(&self, request: ResourceRequest) -> ResourceFuture<'_>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    InvalidRequest,
    NotFound,
    DigestMismatch,
    LimitExceeded,
    Unavailable,
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "renderer resource error: {self:?}")
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::{Error, ResourceBlob};

    #[test]
    fn rejects_empty_and_oversized_blobs() {
        assert_eq!(ResourceBlob::new(Vec::new(), 1), Err(Error::LimitExceeded));
        assert_eq!(ResourceBlob::new(vec![1, 2], 1), Err(Error::LimitExceeded));
    }
}
