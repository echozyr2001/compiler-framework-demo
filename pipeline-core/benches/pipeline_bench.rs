use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use lexer_framework::{DefaultContext as LexDefaultContext, LexContext, Lexer, LexingRule};
use parser_framework::{
    parse_pratt, AstNode, DefaultContext, LazyContext, ParseContext, Parser, ParsingRule, Position,
    PrattConfig,
};
use pipeline_core::BatchPipeline;
use std::sync::Arc;
// use common_framework::TextSlice; // Unused

// --- Shared Types ---
#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(i64),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
    Whitespace,
}

impl lexer_framework::LexToken for Token {
    fn position(&self) -> Option<Position> {
        None
    }
    fn is_eof(&self) -> bool {
        false
    }
    fn is_newline(&self) -> bool {
        false
    }
    fn is_whitespace(&self) -> bool {
        matches!(self, Token::Whitespace)
    }
    fn is_indent(&self) -> bool {
        false
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum Ast {
    Number(i64),
    Binary(Box<Ast>, char, Box<Ast>),
}

impl AstNode for Ast {
    fn position(&self) -> Option<Position> {
        None
    }
}

// --- Lexer Rules ---
struct WhitespaceRule;
impl LexingRule<LexDefaultContext, Token> for WhitespaceRule {
    fn try_match(&mut self, ctx: &mut LexDefaultContext) -> Option<Token> {
        let s = ctx.consume_while(|c| c.is_whitespace());
        if !s.is_empty() {
            Some(Token::Whitespace)
        } else {
            None
        }
    }
    fn quick_check(&self, c: Option<char>) -> Option<bool> {
        c.map(|ch| ch.is_whitespace())
    }
}

struct NumberRule;
impl LexingRule<LexDefaultContext, Token> for NumberRule {
    fn try_match(&mut self, ctx: &mut LexDefaultContext) -> Option<Token> {
        let s = ctx.consume_while(|c| c.is_ascii_digit());
        if !s.is_empty() {
            let n = s.parse().unwrap_or(0);
            Some(Token::Number(n))
        } else {
            None
        }
    }
    fn quick_check(&self, c: Option<char>) -> Option<bool> {
        c.map(|ch| ch.is_ascii_digit())
    }
}

struct OpRule;
impl LexingRule<LexDefaultContext, Token> for OpRule {
    fn try_match(&mut self, ctx: &mut LexDefaultContext) -> Option<Token> {
        let ch = ctx.peek()?;
        let tok = match ch {
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Star,
            '/' => Token::Slash,
            '(' => Token::LParen,
            ')' => Token::RParen,
            _ => return None,
        };
        ctx.advance();
        Some(tok)
    }
    fn quick_check(&self, c: Option<char>) -> Option<bool> {
        c.map(|ch| "+-*/()".contains(ch))
    }
}

// --- Parser Rules (Pratt) ---
struct ExprConfig;
impl<Ctx> PrattConfig<Ctx, Token, Ast> for ExprConfig
where
    Ctx: ParseContext<Token>,
{
    fn prefix_op(&self, token: &Token) -> Option<((), u8)> {
        match token {
            Token::Number(_) | Token::LParen => Some(((), 0)),
            _ => None,
        }
    }
    fn infix_op(&self, token: &Token) -> Option<(u8, u8)> {
        match token {
            Token::Plus | Token::Minus => Some((10, 11)),
            Token::Star | Token::Slash => Some((20, 21)),
            _ => None,
        }
    }
    fn parse_prefix<F>(&self, token: Token, ctx: &mut Ctx, parser: &F) -> Option<Ast>
    where
        F: Fn(&mut Ctx, u8) -> Option<Ast>,
    {
        match token {
            Token::Number(n) => Some(Ast::Number(n)),
            Token::LParen => {
                let expr = parser(ctx, 0)?;
                if matches!(ctx.peek(), Some(Token::RParen)) {
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
        left: Ast,
        token: Token,
        r_bp: u8,
        ctx: &mut Ctx,
        parser: &F,
    ) -> Option<Ast>
    where
        F: Fn(&mut Ctx, u8) -> Option<Ast>,
    {
        let op = match token {
            Token::Plus => '+',
            Token::Minus => '-',
            Token::Star => '*',
            Token::Slash => '/',
            _ => return None,
        };
        let right = parser(ctx, r_bp)?;
        Some(Ast::Binary(Box::new(left), op, Box::new(right)))
    }
}

struct ExprRule {
    config: ExprConfig,
}
impl<Ctx> ParsingRule<Ctx, Token, Ast> for ExprRule
where
    Ctx: ParseContext<Token>,
{
    fn try_parse(&mut self, ctx: &mut Ctx) -> Option<Ast> {
        parse_pratt(ctx, &self.config, 0)
    }
    fn quick_check(&self, token: Option<&Token>) -> Option<bool> {
        matches!(token, Some(Token::Number(_)) | Some(Token::LParen)).into()
    }
}

// --- Generator ---
fn generate_input(lines: usize) -> String {
    // "123 + 456 * ( 789 - 10 )" repeated
    let expr = "123 + 456 * ( 789 - 10 ) \n";
    expr.repeat(lines)
}

// --- Bench ---
fn bench_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("pipeline_batch");
    for &lines in &[1_000usize, 10_000, 50_000] {
        let input = Arc::new(generate_input(lines));
        let len = input.len();
        group.throughput(Throughput::Bytes(len as u64));
        let bench_name = format!("parse_expr_{}k_lines", lines / 1000);
        group.bench_function(bench_name, |b| {
            let input = input.clone();
            b.iter(move || {
                let lexer_rules: Vec<Box<dyn LexingRule<LexDefaultContext, Token>>> = vec![
                    Box::new(WhitespaceRule),
                    Box::new(NumberRule),
                    Box::new(OpRule),
                ];
                let parser_rules: Vec<Box<dyn ParsingRule<DefaultContext<Token>, Token, Ast>>> =
                    vec![Box::new(ExprRule { config: ExprConfig })];
                let lexer = Lexer::from_str(input.as_ref(), lexer_rules);
                BatchPipeline::run_custom(lexer, |tokens| {
                    let filtered: Vec<Token> = tokens
                        .into_iter()
                        .filter(|t| !matches!(t, Token::Whitespace))
                        .collect();
                    Parser::<DefaultContext<Token>, Token, Ast>::from_tokens(filtered, parser_rules)
                })
            })
        });
    }
    group.finish();
}

fn bench_lazy_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("pipeline_lazy");
    for &lines in &[1_000usize, 10_000, 50_000] {
        let input = Arc::new(generate_input(lines));
        let len = input.len();
        group.throughput(Throughput::Bytes(len as u64));
        let bench_name = format!("parse_expr_{}k_lines_lazy", lines / 1000);
        group.bench_function(bench_name, |b| {
            let input = input.clone();
            b.iter(move || {
                let lexer_rules: Vec<Box<dyn LexingRule<LexDefaultContext, Token>>> = vec![
                    Box::new(WhitespaceRule),
                    Box::new(NumberRule),
                    Box::new(OpRule),
                ];
                let lexer = Lexer::from_str(input.as_ref(), lexer_rules);
                let filtered_iter = lexer.filter(|t| !matches!(t, Token::Whitespace));
                let context = LazyContext::new(filtered_iter, 16);
                let parser_rules = vec![Box::new(ExprRule { config: ExprConfig }) as Box<_>];
                let mut parser = Parser::new(context, parser_rules);
                parser.parse()
            })
        });
    }
    group.finish();
}

criterion_group!(benches, bench_pipeline, bench_lazy_pipeline);
criterion_main!(benches);
