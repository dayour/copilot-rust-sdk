// Copyright (c) 2026 Elias Bachaalany
// SPDX-License-Identifier: MIT

//! Generated-surface RPC bindings.
//!
//! Each submodule owns one namespace group from `schemas/api.schema.json`
//! and attaches its methods to [`crate::Session`] or [`crate::Client`] via
//! its own inherent `impl` block.

pub mod canvas;
pub mod client_mcp;
pub mod history;
pub mod mcp;
pub mod metadata;
pub mod permissions;
pub mod sessions;
pub mod skills;
pub mod tasks;
