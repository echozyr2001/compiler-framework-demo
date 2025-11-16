use common_framework::{ConsumerEvent, PipelineMessage};
use lexer_framework::streaming::TokenProducer;
use parser_framework::streaming::TokenConsumer;

/// Drives a lexer and parser in lockstep, routing messages and tokens between
/// them. This is a thin controller that can be extended with richer behavior
/// (e.g. asynchronous scheduling, buffering, backpressure).
pub struct Pipeline<L, P, Tok, Ast>
where
    L: TokenProducer<Tok>,
    P: TokenConsumer<Tok, Ast>,
{
    lexer: L,
    parser: P,
    _marker: std::marker::PhantomData<(Tok, Ast)>,
}

impl<L, P, Tok, Ast> Pipeline<L, P, Tok, Ast>
where
    L: TokenProducer<Tok>,
    P: TokenConsumer<Tok, Ast>,
{
    pub fn new(lexer: L, parser: P) -> Self {
        Self {
            lexer,
            parser,
            _marker: std::marker::PhantomData,
        }
    }

    /// Runs the pipeline until parser finishes, returning all AST nodes.
    pub fn run(mut self) -> Vec<Ast> {
        let mut results = Vec::new();

        loop {
            match self.parser.poll() {
                ConsumerEvent::Produced(mut nodes) => {
                    results.append(&mut nodes);
                }
                ConsumerEvent::NeedToken => {
                    if let Some(token) = self.lexer.poll_token() {
                        self.parser.push_token(token);
                    } else {
                        self.parser.on_message(&PipelineMessage::Finished);
                        results.extend(self.parser.finish());
                        break;
                    }
                }
                ConsumerEvent::Finished(mut nodes) => {
                    results.append(&mut nodes);
                    break;
                }
                ConsumerEvent::Blocked(reason) => {
                    self.parser.on_message(&PipelineMessage::Blocked(reason));
                    break;
                }
            }
        }

        results
    }
}
