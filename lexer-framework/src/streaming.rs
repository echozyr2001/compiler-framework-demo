use crate::context::LexContext;
use crate::lexer::Lexer;
use common_framework::PipelineMessage;

/// Produces tokens on demand, allowing lexers to be consumed in streaming
/// pipelines.
pub trait TokenProducer<Tok> {
    /// Attempts to emit the next token from the underlying producer.
    fn poll_token(&mut self) -> Option<Tok>;

    /// Allows the controller to send control messages to the producer.
    fn on_message(&mut self, _message: &PipelineMessage) {}
}

impl<Ctx, Tok> TokenProducer<Tok> for Lexer<Ctx, Tok>
where
    Ctx: LexContext,
{
    fn poll_token(&mut self) -> Option<Tok> {
        self.next()
    }
}
