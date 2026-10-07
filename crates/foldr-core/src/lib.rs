//! Folder inspection and configuration engine for foldr.
//!
//! This crate owns folder models, platform adapters, capability descriptions,
//! change planning, presets, and recovery. Terminal presentation belongs in
//! `foldr-cli`. The implementation is tracked in the repository's Cairn backlog.

pub mod inspect;
pub mod model;
pub mod platform;
pub use inspect::inspect;
pub use model::*;
pub mod changes;
pub use changes::*;
pub mod presets;
pub use presets::*;
pub mod compare;
pub use compare::*;
