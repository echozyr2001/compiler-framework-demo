use common_framework::Position;
use lexer_framework::streaming::TokenProducer;
use lexer_framework::{
    DefaultContext as LexContext, LexContext as _, LexToken, Lexer, LexingRule,
    Position as LexPosition,
};
use parser_framework::{AstNode, ParseContext, Parser, ParsingRule, StreamingParseContext};
use pipeline_core::Pipeline;

use common_framework::{Inbound, Outbound, StreamingSignal};

fn main() {
    let input = "3 + 4 * (2 - 1) / 5";
    println!("Input: {input}");

    let lexer_rules = build_lexer_rules();
    let parser_rules = build_parser_rules();

    let lexer = FilteringProducer::new(Lexer::from_str(input.to_owned(), lexer_rules));
    let parser = Parser::new(StreamingParseContext::new(), parser_rules);

    let pipeline = Pipeline::new(lexer, parser);
    let asts = pipeline.run();

    println!("ASTs produced by the streaming pipeline:");
    for (idx, ast) in asts.iter().enumerate() {
        println!("  AST {idx}: {ast:?}");
    }
}

// --- Lexer setup ----------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
enum CalcToken {
    Number { value: f64, position: LexPosition },
    Plus { position: LexPosition },
    Minus { position: LexPosition },
    Multiply { position: LexPosition },
    Divide { position: LexPosition },
    LeftParen { position: LexPosition },
    RightParen { position: LexPosition },
    Whitespace { position: LexPosition },
    Eof { position: LexPosition },
}

impl LexToken for CalcToken {
    fn position(&self) -> Option<Position> {
        Some(match self {
            CalcToken::Number { position, .. }
            | CalcToken::Plus { position }
            | CalcToken::Minus { position }
            | CalcToken::Multiply { position }
            | CalcToken::Divide { position }
            | CalcToken::LeftParen { position }
            | CalcToken::RightParen { position }
            | CalcToken::Whitespace { position }
            | CalcToken::Eof { position } => *position,
        })
    }

    fn is_eof(&self) -> bool {
        matches!(self, CalcToken::Eof { .. })
    }

    fn is_newline(&self) -> bool {
        matches!(self, CalcToken::Whitespace { .. })
    }

    fn is_whitespace(&self) -> bool {
        matches!(self, CalcToken::Whitespace { .. })
    }

    fn is_indent(&self) -> bool {
        false
    }
}

fn build_lexer_rules() -> Vec<Box<dyn LexingRule<LexContext, CalcToken>>> {
    vec![
        Box::new(NumberRule),
        Box::new(OperatorRule),
        Box::new(WhitespaceRule),
        Box::new(EofRule),
    ]
}

struct NumberRule;

impl LexingRule<LexContext, CalcToken> for NumberRule {
    fn try_match(&mut self, ctx: &mut LexContext) -> Option<CalcToken> {
        let position = ctx.position();
        let first = ctx.peek()?;
        if !first.is_ascii_digit() {
            return None;
        }

        let mut literal = String::new();
        literal.push(first);
        ctx.advance();
        literal.push_str(ctx.consume_while(|ch| ch.is_ascii_digit()).as_ref());
        if ctx.peek() == Some('.') {
            literal.push('.');
            ctx.advance();
            literal.push_str(ctx.consume_while(|ch| ch.is_ascii_digit()).as_ref());
        }

        literal
            .parse::<f64>()
            .ok()
            .map(|value| CalcToken::Number { value, position })
    }

    fn priority(&self) -> i32 {
        10
    }
}

struct OperatorRule;

impl LexingRule<LexContext, CalcToken> for OperatorRule {
    fn quick_check(&self, first_char: Option<char>) -> Option<bool> {
        match first_char? {
            '+' | '-' | '*' | '/' | '(' | ')' => Some(true),
            _ => Some(false),
        }
    }

    fn try_match(&mut self, ctx: &mut LexContext) -> Option<CalcToken> {
        let position = ctx.position();
        let ch = ctx.peek()?;
        let token = match ch {
            '+' => CalcToken::Plus { position },
            '-' => CalcToken::Minus { position },
            '*' => CalcToken::Multiply { position },
            '/' => CalcToken::Divide { position },
            '(' => CalcToken::LeftParen { position },
            ')' => CalcToken::RightParen { position },
            _ => return None,
        };
        ctx.advance();
        Some(token)
    }
}

struct WhitespaceRule;

impl LexingRule<LexContext, CalcToken> for WhitespaceRule {
    fn try_match(&mut self, ctx: &mut LexContext) -> Option<CalcToken> {
        if ctx.peek().is_some_and(|ch| ch.is_whitespace()) {
            let position = ctx.position();
            ctx.consume_while(|ch| ch.is_whitespace());
            Some(CalcToken::Whitespace { position })
        } else {
            None
        }
    }

    fn priority(&self) -> i32 {
        -1
    }
}

struct EofRule;

impl LexingRule<LexContext, CalcToken> for EofRule {
    fn try_match(&mut self, ctx: &mut LexContext) -> Option<CalcToken> {
        if ctx.is_eof() {
            Some(CalcToken::Eof {
                position: ctx.position(),
            })
        } else {
            None
        }
    }

    fn priority(&self) -> i32 {
        -10
    }
}

// --- Parser setup ---------------------------------------------------------------------

#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields are used in Debug output
enum Expr {
    Number {
        value: f64,
        position: Position,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
        position: Position,
    },
}

#[derive(Debug, Clone, Copy)]
enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl BinaryOp {
    fn precedence(self) -> i32 {
        match self {
            BinaryOp::Add | BinaryOp::Subtract => 10,
            BinaryOp::Multiply | BinaryOp::Divide => 20,
        }
    }
}

impl AstNode for Expr {
    fn position(&self) -> Option<Position> {
        Some(match self {
            Expr::Number { position, .. } | Expr::Binary { position, .. } => *position,
        })
    }
}

struct ExpressionRule;

impl ExpressionRule {
    fn new() -> Self {
        Self
    }
}

impl<Ctx> ParsingRule<Ctx, CalcToken, Expr> for ExpressionRule
where
    Ctx: ParseContext<CalcToken>,
{
    fn try_parse(&mut self, ctx: &mut Ctx) -> Option<Expr> {
        let checkpoint = ctx.checkpoint();
        match parse_expression(ctx, 0) {
            Some(expr) => Some(expr),
            None => {
                ctx.restore(checkpoint);
                None
            }
        }
    }

    fn priority(&self) -> i32 {
        100
    }
}

fn build_parser_rules(
) -> Vec<Box<dyn ParsingRule<StreamingParseContext<CalcToken>, CalcToken, Expr>>> {
    vec![Box::new(ExpressionRule::new())]
}

fn parse_expression<Ctx>(ctx: &mut Ctx, min_precedence: i32) -> Option<Expr>
where
    Ctx: ParseContext<CalcToken>,
{
    let mut left = parse_primary(ctx)?;
    loop {
        let op = match ctx.peek().and_then(binary_op_from_token) {
            Some(op) if op.precedence() >= min_precedence => op,
            _ => break,
        };
        ctx.advance();
        let right = parse_expression(ctx, op.precedence() + 1)?;
        let position = left
            .position()
            .or_else(|| right.position())
            .unwrap_or_default();
        left = Expr::Binary {
            op,
            left: Box::new(left),
            right: Box::new(right),
            position,
        };
    }
    Some(left)
}

fn parse_primary<Ctx>(ctx: &mut Ctx) -> Option<Expr>
where
    Ctx: ParseContext<CalcToken>,
{
    match ctx.peek()?.clone() {
        CalcToken::Number { value, position } => {
            ctx.advance();
            Some(Expr::Number { value, position })
        }
        CalcToken::LeftParen { .. } => {
            ctx.advance();
            let expr = parse_expression(ctx, 0)?;
            if matches!(ctx.peek(), Some(CalcToken::RightParen { .. })) {
                ctx.advance();
                Some(expr)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn binary_op_from_token(token: &CalcToken) -> Option<BinaryOp> {
    match token {
        CalcToken::Plus { .. } => Some(BinaryOp::Add),
        CalcToken::Minus { .. } => Some(BinaryOp::Subtract),
        CalcToken::Multiply { .. } => Some(BinaryOp::Multiply),
        CalcToken::Divide { .. } => Some(BinaryOp::Divide),
        _ => None,
    }
}

// --- Filtering producer ---------------------------------------------------------------

struct FilteringProducer<L> {
    inner: L,
}

impl<L> FilteringProducer<L> {
    fn new(inner: L) -> Self {
        Self { inner }
    }
}

impl<L> TokenProducer<CalcToken> for FilteringProducer<L>
where
    L: TokenProducer<CalcToken>,
{
    fn poll_token(&mut self) -> Option<CalcToken> {
        while let Some(token) = self.inner.poll_token() {
            if let Some(filtered) = filter_token(token) {
                return Some(filtered);
            }
        }
        None
    }
}

impl<L, Ast> Outbound<CalcToken, Ast> for FilteringProducer<L>
where
    L: Outbound<CalcToken, Ast>,
{
    fn next_signal(&mut self) -> Option<StreamingSignal<CalcToken, Ast>> {
        while let Some(signal) = self.inner.next_signal() {
            match signal {
                StreamingSignal::SupplyToken(token) => {
                    if let Some(filtered) = filter_token(token) {
                        return Some(StreamingSignal::SupplyToken(filtered));
                    }
                }
                other => return Some(other),
            }
        }
        None
    }
}

impl<L, Ast> Inbound<CalcToken, Ast> for FilteringProducer<L>
where
    L: Inbound<CalcToken, Ast>,
{
    fn handle_signal(&mut self, signal: StreamingSignal<CalcToken, Ast>) {
        self.inner.handle_signal(signal);
    }
}

fn filter_token(token: CalcToken) -> Option<CalcToken> {
    match token {
        CalcToken::Whitespace { .. } => None,
        other => Some(other),
    }
}
