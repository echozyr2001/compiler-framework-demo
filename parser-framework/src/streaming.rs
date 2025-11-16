use crate::context::{extract_position_from_token, ParseContext};
use crate::parser::Parser;
use crate::traits::AstNode;
use common_framework::{ConsumerEvent, PipelineMessage, Position};
use std::fmt::Debug;

/// Streaming-friendly parse context that can be fed tokens incrementally.
pub struct StreamingParseContext<Tok>
where
    Tok: Clone + Debug,
{
    tokens: Vec<Tok>,
    current: usize,
    finished: bool,
    position: Position,
    stream_closed: bool,
}

impl<Tok> StreamingParseContext<Tok>
where
    Tok: Clone + Debug,
{
    pub fn new() -> Self {
        Self {
            tokens: Vec::new(),
            current: 0,
            finished: false,
            position: Position::default(),
            stream_closed: false,
        }
    }

    /// Pushes a new token into the context buffer.
    pub fn push_token(&mut self, token: Tok) {
        if let Some(pos) = extract_position_from_token(&token) {
            self.position = pos;
        }
        self.tokens.push(token);
        self.finished = false;
        self.stream_closed = false;
    }

    /// Marks the context as finished, indicating no more tokens will arrive.
    pub fn mark_finished(&mut self) {
        self.finished = true;
        self.stream_closed = true;
    }

    pub fn is_stream_closed(&self) -> bool {
        self.stream_closed
    }
}

impl<Tok> ParseContext<Tok> for StreamingParseContext<Tok>
where
    Tok: Clone + Debug,
{
    fn peek(&self) -> Option<&Tok> {
        self.tokens.get(self.current)
    }

    fn peek_at(&self, offset: usize) -> Option<&Tok> {
        self.tokens.get(self.current + offset)
    }

    fn advance(&mut self) -> Option<Tok> {
        let token = self.tokens.get(self.current).cloned()?;
        if let Some(pos) = extract_position_from_token(&token) {
            self.position = pos;
        }
        self.current += 1;
        Some(token)
    }

    fn position(&self) -> Position {
        self.position
    }

    fn is_eof(&self) -> bool {
        self.finished && self.current >= self.tokens.len()
    }

    fn token_index(&self) -> usize {
        self.current
    }

    fn checkpoint(&self) -> crate::context::Checkpoint {
        crate::context::Checkpoint::new(self.current, self.position)
    }

    fn restore(&mut self, checkpoint: crate::context::Checkpoint) {
        self.current = checkpoint.token_index();
        self.position = checkpoint.position();
    }
}

/// Trait for consumers that accept tokens incrementally and emit AST nodes.
pub trait TokenConsumer<Tok, Ast> {
    /// Pushes a single token and returns any AST nodes that became available.
    fn push_token(&mut self, token: Tok) -> Vec<Ast>;

    /// Signals the end of input and drains any remaining AST nodes.
    fn finish(&mut self) -> Vec<Ast>;

    /// Receives pipeline messages from controller.
    fn on_message(&mut self, _message: &PipelineMessage) {}

    /// Attempts to advance the parser and returns the current event.
    fn poll(&mut self) -> ConsumerEvent<Ast>;
}

impl<Tok, Ast> TokenConsumer<Tok, Ast> for Parser<StreamingParseContext<Tok>, Tok, Ast>
where
    Tok: Clone + Debug,
    Ast: AstNode,
{
    fn push_token(&mut self, token: Tok) -> Vec<Ast> {
        self.context_mut().push_token(token);
        Vec::new()
    }

    fn finish(&mut self) -> Vec<Ast> {
        self.context_mut().mark_finished();
        self.parse()
    }

    fn on_message(&mut self, message: &PipelineMessage) {
        if matches!(message, PipelineMessage::NeedToken) {
            // Streaming parser always needs tokens when asked; nothing to do yet.
        }
    }

    fn poll(&mut self) -> ConsumerEvent<Ast> {
        if self.context().is_stream_closed() {
            ConsumerEvent::Finished(Vec::new())
        } else {
            ConsumerEvent::NeedToken
        }
    }
}
