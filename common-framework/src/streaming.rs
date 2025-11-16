/// Message emitted by a component (lexer, parser, controller) during streaming
/// orchestration. Right now it encodes a simple request/response flow between
/// parser and pipeline, but it can be extended for richer collaboration (e.g.
/// backpressure, lookahead hints).
#[derive(Debug, Clone)]
pub enum PipelineMessage {
    /// Parser needs the next token.
    NeedToken,
    /// Parser finished and produced the given AST count.
    ProducedAstCount(usize),
    /// Parser cannot make progress; contains context string.
    Blocked(String),
    /// Parser has finished and there are no more nodes to emit.
    Finished,
}

/// Actions controller can trigger on downstream components.
#[derive(Debug, Clone)]
pub enum PipelineAction<Tok> {
    /// Provide the next token to the parser.
    ProvideToken(Tok),
    /// Signal end-of-input.
    EndOfInput,
}

/// Events emitted by a token consumer (parser) after a poll.
#[derive(Debug)]
pub enum ConsumerEvent<Ast> {
    /// Parser consumed available tokens and produced AST nodes.
    Produced(Vec<Ast>),
    /// Parser requires another token to continue.
    NeedToken,
    /// Parser finished; contains any trailing AST nodes.
    Finished(Vec<Ast>),
    /// Parser is blocked and cannot proceed.
    Blocked(String),
}
