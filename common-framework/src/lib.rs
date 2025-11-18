//! Common Framework
//!
//! 提供 lexer-framework 和 parser-framework 之间的共同组件。

pub mod checkpoint;
pub mod position;
pub mod streaming;
pub mod text_slice;

pub use checkpoint::Checkpoint;
pub use position::Position;
pub use streaming::{Inbound, Outbound, StreamingSignal};
pub use text_slice::TextSlice;
