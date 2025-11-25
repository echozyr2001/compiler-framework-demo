use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use lexer_framework::streaming::StreamingLexContext;
use lexer_framework::{DefaultContext, LexContext, LexToken, Lexer, LexingRule, Position};

#[derive(Debug, Clone, PartialEq)]
enum BenchToken {
    Number(i64),
    Identifier(String),
    Operator(char),
    Whitespace,
    Unknown(char),
}

impl LexToken for BenchToken {
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
        matches!(self, BenchToken::Whitespace)
    }

    fn is_indent(&self) -> bool {
        false
    }
}

// ----- Rules (generic over LexContext) -----

struct WhitespaceRule;
impl<Ctx> LexingRule<Ctx, BenchToken> for WhitespaceRule
where
    Ctx: LexContext,
{
    fn try_match(&mut self, ctx: &mut Ctx) -> Option<BenchToken> {
        let slice = ctx.consume_while(|ch| ch.is_whitespace());
        if !slice.is_empty() {
            Some(BenchToken::Whitespace)
        } else {
            None
        }
    }

    fn priority(&self) -> i32 {
        10
    }

    fn quick_check(&self, first_char: Option<char>) -> Option<bool> {
        first_char.map(|c| c.is_whitespace())
    }
}

struct NumberRule;
impl<Ctx> LexingRule<Ctx, BenchToken> for NumberRule
where
    Ctx: LexContext,
{
    fn try_match(&mut self, ctx: &mut Ctx) -> Option<BenchToken> {
        let slice = ctx.consume_while(|ch| ch.is_ascii_digit());
        if !slice.is_empty() {
            let num = slice.parse().unwrap_or(0);
            Some(BenchToken::Number(num))
        } else {
            None
        }
    }

    fn priority(&self) -> i32 {
        20
    }

    fn quick_check(&self, first_char: Option<char>) -> Option<bool> {
        first_char.map(|c| c.is_ascii_digit())
    }
}

struct IdentifierRule;
impl<Ctx> LexingRule<Ctx, BenchToken> for IdentifierRule
where
    Ctx: LexContext,
{
    fn try_match(&mut self, ctx: &mut Ctx) -> Option<BenchToken> {
        let is_ident_start = |ch: char| ch.is_alphabetic() || ch == '_';
        let is_ident_continue = |ch: char| ch.is_alphanumeric() || ch == '_';

        if let Some(ch) = ctx.peek() {
            if !is_ident_start(ch) {
                return None;
            }
        } else {
            return None;
        }

        let slice = ctx.consume_while(is_ident_continue);
        if !slice.is_empty() {
            Some(BenchToken::Identifier(slice.to_string()))
        } else {
            None
        }
    }

    fn priority(&self) -> i32 {
        15
    }

    fn quick_check(&self, first_char: Option<char>) -> Option<bool> {
        first_char.map(|c| c.is_alphabetic() || c == '_')
    }
}

struct OperatorRule;
impl<Ctx> LexingRule<Ctx, BenchToken> for OperatorRule
where
    Ctx: LexContext,
{
    fn try_match(&mut self, ctx: &mut Ctx) -> Option<BenchToken> {
        let ch = ctx.peek()?;
        if "+-*/=<>!&|".contains(ch) {
            ctx.advance();
            Some(BenchToken::Operator(ch))
        } else {
            None
        }
    }

    fn priority(&self) -> i32 {
        20
    }

    fn quick_check(&self, first_char: Option<char>) -> Option<bool> {
        first_char.map(|c| "+-*/=<>!&|".contains(c))
    }
}

struct UnknownRule;
impl<Ctx> LexingRule<Ctx, BenchToken> for UnknownRule
where
    Ctx: LexContext,
{
    fn try_match(&mut self, ctx: &mut Ctx) -> Option<BenchToken> {
        ctx.advance().map(BenchToken::Unknown)
    }

    fn priority(&self) -> i32 {
        0
    }

    fn quick_check(&self, _first_char: Option<char>) -> Option<bool> {
        Some(true)
    }
}

fn build_rules<Ctx>() -> Vec<Box<dyn LexingRule<Ctx, BenchToken>>>
where
    Ctx: LexContext + 'static,
{
    vec![
        Box::new(WhitespaceRule),
        Box::new(NumberRule),
        Box::new(IdentifierRule),
        Box::new(OperatorRule),
        Box::new(UnknownRule),
    ]
}

// ----- Data generation -----

fn generate_mixed(size_kb: usize) -> String {
    let mut s = String::with_capacity(size_kb * 1024);
    let words = [
        "function", "let", "var", "const", "if", "else", "while", "return", "import", "class",
    ];
    while s.len() < size_kb * 1024 {
        for w in &words {
            s.push_str(w);
            s.push(' ');
            s.push_str("变量");
            s.push(' ');
            s.push('=');
            s.push(' ');
            s.push_str("12345");
            s.push(';');
            s.push('\n');
        }
    }
    s
}

// ----- Benchmarks -----

fn bench_streaming_vs_default(c: &mut Criterion) {
    let size_kb = 100;
    let text = generate_mixed(size_kb);

    let mut group = c.benchmark_group("lexer_stream_vs_default");
    group.throughput(Throughput::Bytes(text.len() as u64));

    group.bench_function("default_context", |b| {
        b.iter(|| {
            let mut lexer = Lexer::from_str(text.as_str(), build_rules::<DefaultContext>());
            black_box(lexer.tokenize());
        })
    });

    group.bench_function("streaming_context", |b| {
        b.iter(|| {
            let ctx = StreamingLexContext::from(text.as_str());
            let mut lexer = Lexer::new(ctx, build_rules::<StreamingLexContext>());
            black_box(lexer.tokenize());
        })
    });

    group.finish();
}

criterion_group!(benches, bench_streaming_vs_default);
criterion_main!(benches);

