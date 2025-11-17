use common_framework::{Inbound, Outbound, StreamingSignal};
use lexer_framework::streaming::TokenProducer;
use parser_framework::streaming::TokenConsumer;

/// Drives a lexer and parser in lockstep, routing messages and tokens between
/// them. This is a thin controller that can be extended with richer behavior
/// (e.g. asynchronous scheduling, buffering, backpressure).
pub struct Pipeline<L, P, Tok, Ast>
where
    L: TokenProducer<Tok> + Inbound<Tok, Ast> + Outbound<Tok, Ast>,
    P: TokenConsumer<Tok, Ast> + Inbound<Tok, Ast> + Outbound<Tok, Ast>,
{
    lexer: L,
    parser: P,
    _marker: std::marker::PhantomData<(Tok, Ast)>,
}

impl<L, P, Tok, Ast> Pipeline<L, P, Tok, Ast>
where
    L: TokenProducer<Tok> + Inbound<Tok, Ast> + Outbound<Tok, Ast>,
    P: TokenConsumer<Tok, Ast> + Inbound<Tok, Ast> + Outbound<Tok, Ast>,
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
            if let Some(signal) = self.parser.next_signal() {
                match signal {
                    StreamingSignal::Produced(mut nodes) => {
                        results.append(&mut nodes);
                        continue;
                    }
                    StreamingSignal::NeedToken(min_needed) => {
                        self.lexer
                            .handle_signal(StreamingSignal::RequestToken(min_needed));
                        if let Some(token_signal) = self.lexer.next_signal() {
                            match token_signal {
                                StreamingSignal::SupplyToken(token) => {
                                    self.parser
                                        .handle_signal(StreamingSignal::SupplyToken(token));
                                }
                                StreamingSignal::EndOfInput => {
                                    self.parser.handle_signal(StreamingSignal::EndOfInput);
                                    results.extend(self.parser.finish());
                                    break;
                                }
                                StreamingSignal::Blocked(reason)
                                | StreamingSignal::Abort(reason) => {
                                    self.parser
                                        .handle_signal(StreamingSignal::Abort(reason.clone()));
                                    self.lexer
                                        .handle_signal(StreamingSignal::Abort(reason.clone()));
                                    break;
                                }
                                _ => {}
                            }
                        } else {
                            self.parser.handle_signal(StreamingSignal::EndOfInput);
                            results.extend(self.parser.finish());
                            break;
                        }
                        continue;
                    }
                    StreamingSignal::Finished(mut nodes) => {
                        results.append(&mut nodes);
                        break;
                    }
                    StreamingSignal::Blocked(reason) | StreamingSignal::Abort(reason) => {
                        self.parser
                            .handle_signal(StreamingSignal::Abort(reason.clone()));
                        self.lexer
                            .handle_signal(StreamingSignal::Abort(reason.clone()));
                        break;
                    }
                    _ => {}
                }
            } else {
                break;
            }
        }

        results
    }
}
