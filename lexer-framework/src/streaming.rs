use crate::context::LexContext;
use crate::cursor::Cursor;
use crate::lexer::Lexer;
use common_framework::{Inbound, Outbound, Position, StreamingSignal};
use std::sync::Arc;

/// Streaming-friendly lex context that can be fed characters incrementally.
/// This is similar to `StreamingParseContext` but for lexing operations.
pub struct StreamingLexContext {
    buffer: String,
    current: usize,
    finished: bool,
    position: Position,
}

impl StreamingLexContext {
    /// Creates a new empty streaming lex context.
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            current: 0,
            finished: false,
            position: Position::default(),
        }
    }

    /// Pushes a new character into the context buffer.
    pub fn push_char(&mut self, ch: char) {
        self.buffer.push(ch);
        self.finished = false;
    }

    /// Pushes a string slice into the context buffer.
    pub fn push_str(&mut self, s: &str) {
        self.buffer.push_str(s);
        self.finished = false;
    }

    /// Marks the context as finished, indicating no more characters will arrive.
    pub fn mark_finished(&mut self) {
        self.finished = true;
    }
}

impl Default for StreamingLexContext {
    fn default() -> Self {
        Self::new()
    }
}

impl From<String> for StreamingLexContext {
    fn from(value: String) -> Self {
        Self {
            buffer: value,
            current: 0,
            finished: true,
            position: Position::default(),
        }
    }
}

impl From<&str> for StreamingLexContext {
    fn from(value: &str) -> Self {
        Self {
            buffer: value.to_string(),
            current: 0,
            finished: true,
            position: Position::default(),
        }
    }
}

impl LexContext for StreamingLexContext {
    fn cursor(&self) -> &Cursor {
        // We need to create a cursor from the buffer
        // Since Cursor requires Arc<str>, we'll need to handle this differently
        // For now, we'll use a workaround by creating a temporary cursor
        // This is not ideal but works for the streaming use case
        panic!("StreamingLexContext::cursor() should not be called directly. Use the LexContext trait methods instead.");
    }

    fn cursor_mut(&mut self) -> &mut Cursor {
        panic!("StreamingLexContext::cursor_mut() should not be called directly. Use the LexContext trait methods instead.");
    }

    fn peek(&self) -> Option<char> {
        if self.current >= self.buffer.len() {
            return None;
        }
        self.buffer[self.current..].chars().next()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        // Advance by the character's byte length
        self.current += ch.len_utf8();
        // Update position
        if ch == '\n' {
            self.position.line += 1;
            self.position.column = 1;
        } else {
            self.position.column += 1;
        }
        self.position.offset = self.current;
        Some(ch)
    }

    fn position(&self) -> Position {
        self.position
    }

    fn is_eof(&self) -> bool {
        self.finished && self.current >= self.buffer.len()
    }

    fn consume_while<F>(&mut self, mut predicate: F) -> common_framework::TextSlice
    where
        F: FnMut(char) -> bool,
    {
        let start = self.current;
        while let Some(ch) = self.peek() {
            if !predicate(ch) {
                break;
            }
            self.advance();
        }
        let end = self.current;
        let buffer_arc = Arc::<str>::from(self.buffer.as_str());
        common_framework::TextSlice::new(buffer_arc, start, end)
    }

    fn checkpoint(&self) -> crate::cursor::Checkpoint {
        crate::cursor::Checkpoint::new(self.current, self.position)
    }

    fn restore(&mut self, checkpoint: crate::cursor::Checkpoint) {
        self.current = checkpoint.current();
        self.position = checkpoint.position();
    }

    fn offset(&self) -> usize {
        self.current
    }
}

/// Produces tokens on demand, allowing lexers to be consumed in streaming
/// pipelines.
pub trait TokenProducer<Tok> {
    /// Attempts to emit the next token from the underlying producer.
    fn poll_token(&mut self) -> Option<Tok>;
}

impl<Ctx, Tok> TokenProducer<Tok> for Lexer<Ctx, Tok>
where
    Ctx: LexContext,
{
    fn poll_token(&mut self) -> Option<Tok> {
        self.next()
    }
}

impl<Ctx, Tok, Ast> Outbound<Tok, Ast> for Lexer<Ctx, Tok>
where
    Ctx: LexContext,
{
    fn next_signal(&mut self) -> Option<StreamingSignal<Tok, Ast>> {
        if let Some(token) = self.poll_token() {
            return Some(StreamingSignal::SupplyToken(token));
        }

        if self.context().is_eof() {
            return Some(StreamingSignal::EndOfInput);
        }

        None
    }
}

impl<Ctx, Tok, Ast> Inbound<Tok, Ast> for Lexer<Ctx, Tok>
where
    Ctx: LexContext,
{
    fn handle_signal(&mut self, signal: StreamingSignal<Tok, Ast>) {
        if let StreamingSignal::Abort(reason) = signal {
            eprintln!("Lexer received abort: {}", reason);
        }
    }
}
