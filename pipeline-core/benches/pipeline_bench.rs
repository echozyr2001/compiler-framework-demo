use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use lexer_framework::{DefaultContext as LexDefaultContext, LexContext, Lexer, LexingRule};
use parser_framework::{
    parse_pratt, AstNode, DefaultContext, LazyContext, ParseContext, Parser, ParsingRule, Position,
    PrattConfig,
};
use pipeline_core::BatchPipeline;
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

    // 1000 lines of expressions (~25KB input)
    let lines = 1000;
    let input = generate_input(lines);

    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_function("parse_expr_1k_lines", |b| {
        b.iter(|| {
            let lexer_rules: Vec<Box<dyn LexingRule<LexDefaultContext, Token>>> = vec![
                Box::new(WhitespaceRule),
                Box::new(NumberRule),
                Box::new(OpRule),
            ];

            // BatchPipeline::run expects a Parser Builder or Rules
            // But wait, BatchPipeline::run in lib.rs uses Lexer::from_str and Parser::from_tokens internal logic?
            // Let's check signature.
            // It takes `lexer_rules` and `parser_rules`.

            let parser_rules: Vec<Box<dyn ParsingRule<DefaultContext<Token>, Token, Ast>>> =
                vec![Box::new(ExprRule { config: ExprConfig })];

            // Use the convenience method
            // Note: Whitespace filtering is usually needed.
            // Our Parser rules don't handle Whitespace tokens explicitly (Pratt parser doesn't skip them automatically unless configured).
            // We need a way to filter whitespace.
            // BatchPipeline doesn't have built-in filtering.
            // So we must use `run_custom` or modify Lexer rules to SKIP whitespace (return None).

            // In our Lexer rule above: WhitespaceRule returns Some(Token::Whitespace).
            // If we want to skip, it should return None (but consume text).
            // But Lexer expects Some(token). If try_match returns None, Lexer thinks rule didn't match.
            // Wait, Lexer logic: if try_match returns None, it restores checkpoint and tries next rule.
            // If NO rule matches, it's an error.
            // So Lexer MUST produce a token or error.
            // So we must produce Token::Whitespace.
            // Then we need to filter it before Parser.

            // Use run_custom for filtering
            let lexer = Lexer::from_str(input.as_str(), lexer_rules);

            BatchPipeline::run_custom(lexer, |tokens| {
                // Filter whitespace here
                let filtered: Vec<Token> = tokens
                    .into_iter()
                    .filter(|t| !matches!(t, Token::Whitespace))
                    .collect();
                Parser::<DefaultContext<Token>, Token, Ast>::from_tokens(filtered, parser_rules)
            });
        })
    });

    group.finish();
}

fn bench_lazy_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("pipeline_lazy");

    let lines = 1000;
    let input = generate_input(lines);

    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_function("parse_expr_1k_lines_lazy", |b| {
        b.iter(|| {
            let lexer_rules: Vec<Box<dyn LexingRule<LexDefaultContext, Token>>> = vec![
                Box::new(WhitespaceRule),
                Box::new(NumberRule),
                Box::new(OpRule),
            ];

            // Need explicit type annotation for the rules because LazyContext is complex
            // LazyContext type: LazyContext<Filter<...>, Token>
            // The iterator type is opaque (Filter), making it hard to name the type explicitly.
            // But ParsingRule is a trait object, so we can use `Box<dyn ...>`.
            // However, `dyn ParsingRule` needs a concrete Context type parameter.

            // Strategy: Use type inference where possible.
            let lexer = Lexer::from_str(input.as_str(), lexer_rules);
            let filtered_iter = lexer.filter(|t| !matches!(t, Token::Whitespace));

            // Create context
            // Window size 16 is plenty for this grammar (LL(1) basically)
            let context = LazyContext::new(filtered_iter, 16);

            // Construct parser rules for THIS context type
            // This is the tricky part: we need to construct the Vec<Box<dyn ParsingRule...>>
            // knowing the exact type of context.
            let rule = ExprRule { config: ExprConfig };
            let rules = vec![Box::new(rule) as Box<dyn ParsingRule<_, _, _>>];

            let mut parser = Parser::new(context, rules);
            parser.parse()
        })
    });

    group.finish();
}

criterion_group!(benches, bench_pipeline, bench_lazy_pipeline);
criterion_main!(benches);
