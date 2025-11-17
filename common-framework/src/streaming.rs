/// Protocol-level signals that flow through the streaming pipeline.
///
/// This enum is inspired by message-based P2P protocols: every controller command
/// (action) and parser/lexer status report (message) is a variant here. Once the
/// pipeline adopts a channel-based architecture, components can treat these
/// signals as either inbound commands or outbound responses.
#[derive(Debug, Clone)]
pub enum StreamingSignal<Tok, Ast> {
    /// Controller asks the lexer to emit up to `n` tokens.
    RequestToken(usize),
    /// Lexer supplies a token produced from the input stream.
    SupplyToken(Tok),
    /// Controller confirms a token was delivered to the parser.
    TokenDelivered,
    /// Parser reports it produced `n` AST nodes.
    Produced(Vec<Ast>),
    /// Parser needs `n` more tokens before continuing.
    NeedToken(usize),
    /// Parser has completed and emitted remaining AST nodes.
    Finished(Vec<Ast>),
    /// Parser or lexer hit a blocking condition with a reason.
    Blocked(String),
    /// Controller signals that the input stream is finished.
    EndOfInput,
    /// Controller forces the pipeline to abort, optionally with reason.
    Abort(String),
}

/// Trait implemented by components that can **receive** streaming signals.
///
/// Similar to a P2P `Inbound` handler, the receiver decides how to react when a
/// controller or peer sends a `StreamingSignal`.
pub trait Inbound<Tok, Ast> {
    fn handle_signal(&mut self, signal: StreamingSignal<Tok, Ast>);
}

/// Trait implemented by components that can **emit** streaming signals.
///
/// Components that participate in the protocol expose an outbound channel so
/// that the controller can pull their latest status (e.g. parser produced ASTs,
/// lexer issued `Blocked`, etc.).
pub trait Outbound<Tok, Ast> {
    fn next_signal(&mut self) -> Option<StreamingSignal<Tok, Ast>>;
}
