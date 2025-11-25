use crate::context::{extract_position_from_token, ParseContext};
use crate::parser::Parser;
use crate::traits::AstNode;
use common_framework::{Checkpoint, Inbound, Outbound, Position, StreamingSignal};
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
    fn peek(&mut self) -> Option<&Tok> {
        self.tokens.get(self.current)
    }

    fn peek_at(&mut self, offset: usize) -> Option<&Tok> {
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

    fn is_eof(&mut self) -> bool {
        self.finished && self.current >= self.tokens.len()
    }

    fn token_index(&self) -> usize {
        self.current
    }

    fn checkpoint(&self) -> Checkpoint {
        Checkpoint::new(self.current, self.position)
    }

    fn restore(&mut self, checkpoint: Checkpoint) {
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
        self.drain_ready_nodes()
    }

    fn finish(&mut self) -> Vec<Ast> {
        self.context_mut().mark_finished();
        self.drain_ready_nodes()
    }
}

impl<Tok, Ast> Parser<StreamingParseContext<Tok>, Tok, Ast>
where
    Tok: Clone + Debug,
    Ast: AstNode,
{
    fn drain_ready_nodes(&mut self) -> Vec<Ast> {
        let mut nodes = Vec::new();
        loop {
            let before = self.context().token_index();
            match self.next_node() {
                Some(node) => nodes.push(node),
                None => break,
            }
            if self.context().token_index() == before {
                break;
            }
        }
        nodes
    }
}

impl<Tok, Ast> Outbound<Tok, Ast> for Parser<StreamingParseContext<Tok>, Tok, Ast>
where
    Tok: Clone + Debug,
    Ast: AstNode,
{
    fn next_signal(&mut self) -> Option<StreamingSignal<Tok, Ast>> {
        let produced = self.drain_ready_nodes();
        if !produced.is_empty() {
            return Some(StreamingSignal::Produced(produced));
        }

        if self.context_mut().is_eof() {
            return Some(StreamingSignal::Finished(Vec::new()));
        }

        // Need more tokens.
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
