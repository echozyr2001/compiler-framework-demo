use crate::context::{extract_position_from_token, ParseContext};
use crate::parser::Parser;
use crate::traits::AstNode;
use common_framework::{Inbound, Outbound, Position, StreamingSignal};
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
}

impl<Tok> Default for StreamingParseContext<Tok>
where
    Tok: Clone + Debug,
{
    fn default() -> Self {
        Self::new()
    }
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
        }
    }

    /// Pushes a new token into the context buffer.
    pub fn push_token(&mut self, token: Tok) {
        if let Some(pos) = extract_position_from_token(&token) {
            self.position = pos;
        }
        self.tokens.push(token);
        self.finished = false;
    }

    /// Marks the context as finished, indicating no more tokens will arrive.
    pub fn mark_finished(&mut self) {
        self.finished = true;
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
}

impl<Tok, Ast> Outbound<Tok, Ast> for Parser<StreamingParseContext<Tok>, Tok, Ast>
where
    Tok: Clone + Debug,
    Ast: AstNode,
{
    fn next_signal(&mut self) -> Option<StreamingSignal<Tok, Ast>> {
        // In streaming mode, we only try to parse when we have EOF,
        // otherwise we just request more tokens
        if self.context().is_eof() {
            // When EOF is reached, parse all remaining tokens
            let remaining = self.parse();
            if !remaining.is_empty() {
                return Some(StreamingSignal::Produced(remaining));
            }
            return Some(StreamingSignal::Finished(Vec::new()));
        }

        // Before EOF, we always need more tokens
        // Don't try to parse incrementally as it may fail for complex expressions
        Some(StreamingSignal::NeedToken(1))
    }
}

impl<Tok, Ast> Inbound<Tok, Ast> for Parser<StreamingParseContext<Tok>, Tok, Ast>
where
    Tok: Clone + Debug,
    Ast: AstNode,
{
    fn handle_signal(&mut self, signal: StreamingSignal<Tok, Ast>) {
        match signal {
            StreamingSignal::SupplyToken(token) => {
                self.push_token(token);
            }
            StreamingSignal::EndOfInput => {
                self.context_mut().mark_finished();
            }
            StreamingSignal::Abort(_) => {
                self.context_mut().mark_finished();
            }
            _ => {}
        }
    }
}
