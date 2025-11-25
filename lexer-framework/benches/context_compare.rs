use criterion::{black_box, criterion_group, criterion_main, BenchmarkGroup, Criterion, Throughput};
use lexer_framework::{
    DefaultContext, LexContext, LexToken, Lexer, LexingRule, Position, StreamingLexContext,
};

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

struct WhitespaceRule;
impl<Ctx> LexingRule<Ctx, BenchToken> for WhitespaceRule
where
    Ctx: LexContext,
{
    fn try_match(&mut self, ctx: &mut Ctx) -> Option<BenchToken> {
        let slice = ctx.consume_while(|c| c.is_whitespace());
        if slice.is_empty() {
            None
        } else {
            Some(BenchToken::Whitespace)
        }
    }

    fn priority(&self) -> i32 {
        10
    }

    fn quick_check(&self, c: Option<char>) -> Option<bool> {
        c.map(|ch| ch.is_whitespace())
    }
}

struct NumberRule;
impl<Ctx> LexingRule<Ctx, BenchToken> for NumberRule
where
    Ctx: LexContext,
{
    fn try_match(&mut self, ctx: &mut Ctx) -> Option<BenchToken> {
        let slice = ctx.consume_while(|c| c.is_ascii_digit());
        if slice.is_empty() {
            None
        } else {
            let value = slice.parse().unwrap_or(0);
            Some(BenchToken::Number(value))
        }
    }

    fn priority(&self) -> i32 {
        20
    }

    fn quick_check(&self, c: Option<char>) -> Option<bool> {
        c.map(|ch| ch.is_ascii_digit())
    }
}

struct IdentifierRule;
impl<Ctx> LexingRule<Ctx, BenchToken> for IdentifierRule
where
    Ctx: LexContext,
{
    fn try_match(&mut self, ctx: &mut Ctx) -> Option<BenchToken> {
        let is_start = |ch: char| ch.is_alphabetic() || ch == '_';
        let is_continue = |ch: char| ch.is_alphanumeric() || ch == '_';

        let Some(first) = ctx.peek() else {
            return None;
        };
        if !is_start(first) {
            return None;
        }
        let slice = ctx.consume_while(is_continue);
        if slice.is_empty() {
            None
        } else {
            Some(BenchToken::Identifier(slice.to_string()))
        }
    }

    fn priority(&self) -> i32 {
        15
    }

    fn quick_check(&self, c: Option<char>) -> Option<bool> {
        c.map(|ch| ch.is_alphabetic() || ch == '_')
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

    fn quick_check(&self, c: Option<char>) -> Option<bool> {
        c.map(|ch| "+-*/=<>!&|".contains(ch))
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

    fn quick_check(&self, _: Option<char>) -> Option<bool> {
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

fn generate_text(size_kb: usize) -> String {
    let words = [
        "function", "let", "var", "const", "if", "else", "while", "return", "import", "export",
        "class", "interface",
    ];
    let mut s = String::with_capacity(size_kb * 1024);
    while s.len() < size_kb * 1024 {
        for w in words {
            s.push_str(w);
            s.push(' ');
            s.push_str("x_Variable");
            s.push(' ');
            s.push('=');
            s.push(' ');
            s.push_str("12345");
            s.push_str(";\n");
        }
    }
    s
}

fn bench_default(group: &mut BenchmarkGroup<'_, criterion::measurement::WallTime>, name: &str, input: &str) {
    group.bench_function(name, |b| {
        b.iter(|| {
            let rules = build_rules::<DefaultContext>();
            let mut lexer = Lexer::from_str(black_box(input), rules);
            let _ = lexer.tokenize();
        })
    });
}

fn bench_streaming(group: &mut BenchmarkGroup<'_, criterion::measurement::WallTime>, name: &str, input: &str) {
    group.bench_function(name, |b| {
        b.iter(|| {
            let rules = build_rules::<StreamingLexContext>();
            let context = StreamingLexContext::from(black_box(input));
            let mut lexer = Lexer::new(context, rules);
            let _ = lexer.tokenize();
        })
    });
}

fn bench_contexts(c: &mut Criterion) {
    let mut group = c.benchmark_group("lex_context_compare");
    let size_kb = 100;
    let text = generate_text(size_kb);

    group.throughput(Throughput::Bytes(text.len() as u64));
    bench_default(&mut group, "default_english_100kb", &text);
    bench_streaming(&mut group, "streaming_english_100kb", &text);
    group.finish();
}

criterion_group!(benches, bench_contexts);
criterion_main!(benches);

