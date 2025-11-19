// Non-streaming batch pipeline
use lexer_framework::{DefaultContext as LexDefaultContext, Lexer, LexingRule};
use parser_framework::{AstNode, DefaultContext as ParseDefaultContext, Parser, ParsingRule};

/// A batch pipeline that processes input in two stages:
/// 1. Lexer tokenizes the entire input
/// 2. Parser parses all tokens into AST nodes
///
/// This is the default (non-streaming) mode of operation.
pub struct BatchPipeline<Tok, Ast>
where
    Tok: Clone + std::fmt::Debug,
    Ast: AstNode,
{
    _marker: std::marker::PhantomData<(Tok, Ast)>,
}

impl<Tok, Ast> BatchPipeline<Tok, Ast>
where
    Tok: Clone + std::fmt::Debug,
    Ast: AstNode,
{
    /// Creates a new batch pipeline.
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }

    /// Runs the pipeline with the given input, lexer rules, and parser rules.
    ///
    /// This method:
    /// 1. Tokenizes the entire input using the lexer
    /// 2. Parses all tokens into AST nodes using the parser
    /// 3. Returns all AST nodes
    pub fn run<S: Into<String>>(
        input: S,
        lexer_rules: Vec<Box<dyn LexingRule<LexDefaultContext, Tok>>>,
        parser_rules: Vec<Box<dyn ParsingRule<ParseDefaultContext<Tok>, Tok, Ast>>>,
    ) -> Vec<Ast> {
        // Stage 1: Tokenize entire input
        let mut lexer = Lexer::from_str(input, lexer_rules);
        let tokens: Vec<Tok> = lexer.tokenize();

        // Stage 2: Parse all tokens
        let mut parser =
            Parser::<ParseDefaultContext<Tok>, Tok, Ast>::from_tokens(tokens, parser_rules);
        parser.parse()
    }

    /// Runs the pipeline with a pre-created lexer, extracting tokens and creating a parser.
    ///
    /// This method allows more control over the lexer setup,
    /// useful when you need custom contexts.
    pub fn run_with_lexer<LCtx>(
        mut lexer: Lexer<LCtx, Tok>,
        parser_rules: Vec<Box<dyn ParsingRule<ParseDefaultContext<Tok>, Tok, Ast>>>,
    ) -> Vec<Ast>
    where
        LCtx: lexer_framework::LexContext,
    {
        // Stage 1: Tokenize entire input
        let tokens: Vec<Tok> = lexer.tokenize();

        // Stage 2: Create parser from tokens and parse
        let mut parser =
            Parser::<ParseDefaultContext<Tok>, Tok, Ast>::from_tokens(tokens, parser_rules);
        parser.parse()
    }
}

impl<Tok, Ast> Default for BatchPipeline<Tok, Ast>
where
    Tok: Clone + std::fmt::Debug,
    Ast: AstNode,
{
    fn default() -> Self {
        Self::new()
    }
}

// Streaming pipeline (only available with streaming feature)
#[cfg(feature = "streaming")]
use common_framework::{Inbound, Outbound, StreamingSignal};
#[cfg(feature = "streaming")]
use lexer_framework::streaming::TokenProducer;
#[cfg(feature = "streaming")]
use parser_framework::streaming::TokenConsumer;

/// Drives a lexer and parser in lockstep, routing messages and tokens between
/// them. This is a thin controller that can be extended with richer behavior
/// (e.g. asynchronous scheduling, buffering, backpressure).
///
/// This struct is only available when the `streaming` feature is enabled.
#[cfg(feature = "streaming")]
pub struct StreamingPipeline<L, P, Tok, Ast>
where
    L: TokenProducer<Tok> + Inbound<Tok, Ast> + Outbound<Tok, Ast>,
    P: TokenConsumer<Tok, Ast> + Inbound<Tok, Ast> + Outbound<Tok, Ast>,
{
    lexer: L,
    parser: P,
    _marker: std::marker::PhantomData<(Tok, Ast)>,
}

#[cfg(feature = "streaming")]
impl<L, P, Tok, Ast> StreamingPipeline<L, P, Tok, Ast>
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

        while let Some(signal) = self.parser.next_signal() {
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
        }

        results
    }
}

/// Type alias for backward compatibility.
/// Use `StreamingPipeline` for new code.
#[cfg(feature = "streaming")]
pub type Pipeline<L, P, Tok, Ast> = StreamingPipeline<L, P, Tok, Ast>;
