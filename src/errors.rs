// Copyright (c) The nextest Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use quick_xml::encoding::EncodingError;
use std::io;
use thiserror::Error;

/// An error that occurs while serializing a [`Report`](crate::Report).
///
/// Returned by [`Report::serialize`](crate::Report::serialize) and
/// [`Report::to_string`](crate::Report::to_string).
#[derive(Debug, Error)]
#[error("error serializing JUnit report")]
pub struct SerializeError {
    #[from]
    inner: quick_xml::Error,
}

impl From<EncodingError> for SerializeError {
    fn from(inner: EncodingError) -> Self {
        Self {
            inner: quick_xml::Error::Encoding(inner),
        }
    }
}

impl From<io::Error> for SerializeError {
    fn from(inner: io::Error) -> Self {
        Self {
            inner: quick_xml::Error::from(inner),
        }
    }
}
