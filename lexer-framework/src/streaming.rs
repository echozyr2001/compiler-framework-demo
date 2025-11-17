use crate::context::LexContext;
use crate::lexer::Lexer;
use common_framework::{Inbound, Outbound, StreamingSignal};

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
