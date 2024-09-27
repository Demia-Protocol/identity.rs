// Copyright 2020-2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub use demia_document::DemiaDocument;
pub use demia_document_metadata::DemiaDocumentMetadata;

mod demia_document;
mod demia_document_metadata;

#[cfg(test)]
pub(crate) mod test_utils;
