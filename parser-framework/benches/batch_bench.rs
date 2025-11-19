use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use parser_framework::{
    parse_pratt, AstNode, DefaultContext, ParseContext, Parser, ParsingRule, Position, PrattConfig,
};

// --- Types ---
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
enum BenchToken {
    Number(i64),
    Identifier(String),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
    Eof,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum BenchAst {
    Number(i64),
    Var(String),
    Binary(Box<BenchAst>, char, Box<BenchAst>),
    Call(String, Vec<BenchAst>), // Function call
}

impl AstNode for BenchAst {
    fn position(&self) -> Option<Position> {
        None
    }
}

// --- Pratt Configuration ---
struct ExprParserConfig;

impl PrattConfig<DefaultContext<BenchToken>, BenchToken, BenchAst> for ExprParserConfig {
    fn prefix_op(&self, token: &BenchToken) -> Option<((), u8)> {
        match token {
            BenchToken::Number(_) | BenchToken::Identifier(_) | BenchToken::LParen => Some(((), 0)),
            _ => None,
        }
    }

    fn infix_op(&self, token: &BenchToken) -> Option<(u8, u8)> {
        match token {
            BenchToken::Plus | BenchToken::Minus => Some((10, 11)), // Left associative
            BenchToken::Star | BenchToken::Slash => Some((20, 21)), // Left associative
            _ => None,
        }
    }

    fn parse_prefix<F>(
        &self,
        token: BenchToken,
        ctx: &mut DefaultContext<BenchToken>,
        parser: &F,
    ) -> Option<BenchAst>
    where
        F: Fn(&mut DefaultContext<BenchToken>, u8) -> Option<BenchAst>,
    {
        match token {
            BenchToken::Number(n) => Some(BenchAst::Number(n)),
            BenchToken::Identifier(s) => {
                // Check for function call: ident ( ... )
                if let Some(BenchToken::LParen) = ctx.peek() {
                    ctx.advance(); // eat '('
                    let mut args = Vec::new();
                    // Simple arg parsing: just one expr for bench simplicity, or loop
                    if let Some(arg) = parser(ctx, 0) {
                        args.push(arg);
                    }
                    // Expect ')'
                    if let Some(BenchToken::RParen) = ctx.peek() {
                        ctx.advance();
                        Some(BenchAst::Call(s, args))
                    } else {
                        None // Error in real parser
                    }
                } else {
                    Some(BenchAst::Var(s))
                }
            }
            BenchToken::LParen => {
                let expr = parser(ctx, 0)?;
                if let Some(BenchToken::RParen) = ctx.peek() {
                    ctx.advance();
                    Some(expr)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn parse_infix<F>(
        &self,
        left: BenchAst,
        token: BenchToken,
        r_bp: u8,
        ctx: &mut DefaultContext<BenchToken>,
        parser: &F,
    ) -> Option<BenchAst>
    where
        F: Fn(&mut DefaultContext<BenchToken>, u8) -> Option<BenchAst>,
    {
        let op = match token {
            BenchToken::Plus => '+',
            BenchToken::Minus => '-',
            BenchToken::Star => '*',
            BenchToken::Slash => '/',
            _ => return None,
        };

        let right = parser(ctx, r_bp)?;
        Some(BenchAst::Binary(Box::new(left), op, Box::new(right)))
    }
}

// --- Rules ---
// A generic Expression Rule that uses Pratt Parser
struct ExpressionRule {
    config: ExprParserConfig,
}

impl ExpressionRule {
    fn new() -> Self {
        Self {
            config: ExprParserConfig,
        }
    }
}

impl ParsingRule<DefaultContext<BenchToken>, BenchToken, BenchAst> for ExpressionRule {
    fn try_parse(&mut self, ctx: &mut DefaultContext<BenchToken>) -> Option<BenchAst> {
        parse_pratt(ctx, &self.config, 0)
    }

    fn quick_check(&self, token: Option<&BenchToken>) -> Option<bool> {
        // Can start with Number, Ident, or '('
        match token {
            Some(BenchToken::Number(_))
            | Some(BenchToken::Identifier(_))
            | Some(BenchToken::LParen) => Some(true),
            _ => Some(false),
        }
    }
}

// --- Data Generation ---
// Generates a mix of expressions: "a + b * c", "foo(10) - 5", etc.
fn generate_expression_tokens(count: usize) -> Vec<BenchToken> {
    let mut tokens = Vec::with_capacity(count);
    // Pattern: Var + Var * Var - Call(Num) ...
    // Repeated pattern length: ~10 tokens
    // "a" "+" "b" "*" "c" "-" "func" "(" "1" ")"
    for _ in 0..(count / 10) {
        tokens.push(BenchToken::Identifier("a".to_string()));
        tokens.push(BenchToken::Plus);
        tokens.push(BenchToken::Identifier("b".to_string()));
        tokens.push(BenchToken::Star);
        tokens.push(BenchToken::Identifier("c".to_string()));
        tokens.push(BenchToken::Minus);
        tokens.push(BenchToken::Identifier("func".to_string()));
        tokens.push(BenchToken::LParen);
        tokens.push(BenchToken::Number(1));
        tokens.push(BenchToken::RParen);

        // Add a connector occasionally or let the parser loop handle separate expressions?
        // Standard parser usually parses a LIST of expressions or statements.
        // Our ExpressionRule parses ONE expression.
        // So we need a way to chain them, OR the bench loop creates a new parser for each?
        // Creating parser is cheap. But let's simulate a "Program" rule that parses a list of exprs.
    }
    // Ensure we don't end with partial expr if count is small, but for bench it's fine.
    tokens
}

fn bench_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_complex");

    // 10k tokens representing complex expressions
    let size = 10_000;
    let tokens = generate_expression_tokens(size);

    group.throughput(Throughput::Elements(size as u64));
    group.bench_function("parse_expr_mixed", |b| {
        b.iter(|| {
            let rules: Vec<Box<dyn ParsingRule<DefaultContext<BenchToken>, BenchToken, BenchAst>>> =
                vec![Box::new(ExpressionRule::new())];
            // Clone tokens to simulate fresh input
            let input_tokens = tokens.clone();
            let mut parser =
                Parser::<DefaultContext<BenchToken>, BenchToken, BenchAst>::from_tokens(
                    input_tokens,
                    rules,
                );
            // Parser::parse() loops until EOF, calling try_parse repeatedly.
            // Since our input is "Expr Expr Expr...", ExpressionRule will match repeatedly.
            let _nodes = parser.parse();
        })
    });

    group.finish();
}

criterion_group!(benches, bench_parser);
criterion_main!(benches);
