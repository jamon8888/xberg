//! Tenant-scoped sidecar persistence for the Xberg HTTP API.
//!
//! This crate never stores vectors or full document text — that stays in
//! `xberg-rag`. It owns the ID-keyed sidecar state the corpus has no place
//! for: encrypted rehydration maps today; durable jobs and an audit log in
//! future phases. See
//! `docs/superpowers/specs/2026-06-30-api-document-store-design.md`.
//!
//! ## Feature layers
//!
//! - `in-memory` (default): ephemeral [`backends::memory::InMemoryRehydrationStore`]
//!   (moka, 24h TTL). Matches the behavior shipped in `xberg::api` prior to
//!   this crate — entries are lost on process restart.
//! - `sqlite`: durable [`backends::sqlite::SqliteRehydrationStore`] (WAL-mode
//!   `rusqlite`, tenant + id primary key).

pub mod error;
pub mod rehydration;
pub mod tenant;

pub use error::{StoreError, StoreResult};
pub use rehydration::RehydrationStore;
pub use tenant::{ActorId, TenantCtx, TenantId};
pub use xberg_rag::types::DocumentId;
