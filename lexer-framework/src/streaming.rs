use crate::context::LexContext;
use crate::lexer::Lexer;

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
use crate::context::LexContext;
use crate::lexer::Lexer;

/// Trait representing a producer that can yield tokens incrementally.
pub trait TokenProducer<Tok> {
    /// Attempts to produce the next token from the underlying source.
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
