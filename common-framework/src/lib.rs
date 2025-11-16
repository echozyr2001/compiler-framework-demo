//! Common Framework
//!
//! 提供 lexer-framework 和 parser-framework 之间的共同组件。

pub mod position;
pub mod text_slice;

pub use position::Position;
pub use text_slice::TextSlice;
