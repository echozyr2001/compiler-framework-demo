//! Common Framework
//!
//! Shared building blocks for the lexer and parser frameworks:
//!  - [`Position`]: consistent line/column/offset markers.
//!  - [`Checkpoint`]: progress snapshots used for backtracking and lazy evaluation.
//!  - [`TextSlice`]: reference-counted immutable string slices.
//!  - [`StreamingSignal`] / [`Inbound`] / [`Outbound`]: protocol primitives for real-time/incremental pipelines.
//!
//! These types are lightweight and do not depend on concrete lexer/parser implementations,
//! so they can be reused in custom projects as well.

pub mod checkpoint;
pub mod position;
pub mod streaming;
pub mod text_slice;

pub use checkpoint::Checkpoint;
pub use position::Position;
pub use streaming::{Inbound, Outbound, StreamingSignal};
pub use text_slice::TextSlice;
