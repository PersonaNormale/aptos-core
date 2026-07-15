// Parts of the file are Copyright (c) The Diem Core Contributors
// Parts of the file are Copyright (c) The Move Contributors
// Parts of the file are Copyright (c) Aptos Foundation
// All Aptos Foundation code and content is licensed pursuant to the Innovation-Enabling Source Code License, available at https://github.com/aptos-labs/aptos-core/blob/main/LICENSE

//! Error contract shared by every stage of test plan construction.

/// A diagnostic has already been emitted to `GlobalEnv` for this failure.
pub(super) struct ErrorReported;

/// Result of a step that reports its own diagnostics on failure.
pub(super) type Checked<T> = Result<T, ErrorReported>;
