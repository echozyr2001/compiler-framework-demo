use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use lexer_framework::{DefaultContext, LexContext, LexToken, Lexer, LexingRule, Position};

// --- Token Definition ---
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

// --- Rules ---

// 1. Whitespace Rule
struct WhitespaceRule;
impl LexingRule<DefaultContext, BenchToken> for WhitespaceRule {
    fn try_match(&mut self, ctx: &mut DefaultContext) -> Option<BenchToken> {
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

// 2. Number Rule (Simple Digits)
struct NumberRule;
impl LexingRule<DefaultContext, BenchToken> for NumberRule {
    fn try_match(&mut self, ctx: &mut DefaultContext) -> Option<BenchToken> {
        let slice = ctx.consume_while(|ch| ch.is_ascii_digit());
        if !slice.is_empty() {
            // We just want to benchmark lexing speed, so unwrap is fine for bench data
            let num: i64 = slice.parse().unwrap_or(0);
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

// 3. Identifier Rule (Unicode aware)
// Matches [Alpha | _ | Chinese | Emoji] [AlphaNumeric | _ | Chinese | Emoji]*
struct IdentifierRule;
impl LexingRule<DefaultContext, BenchToken> for IdentifierRule {
    fn try_match(&mut self, ctx: &mut DefaultContext) -> Option<BenchToken> {
        // Helper for start char
        let is_ident_start = |ch: char| ch.is_alphabetic() || ch == '_'; // is_alphabetic includes Chinese
                                                                         // Helper for continue char
        let is_ident_continue = |ch: char| ch.is_alphanumeric() || ch == '_';

        // Check first char
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
        // Note: quick_check optimization relies on this being fast.
        // For non-ASCII, Lexer logic might skip lookup table and call this.
        first_char.map(|c| c.is_alphabetic() || c == '_')
    }
}

// 4. Operator Rule
struct OperatorRule;
impl LexingRule<DefaultContext, BenchToken> for OperatorRule {
    fn try_match(&mut self, ctx: &mut DefaultContext) -> Option<BenchToken> {
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

// 5. Fallback Rule (consume 1 char to prevent infinite loop)
struct UnknownRule;
impl LexingRule<DefaultContext, BenchToken> for UnknownRule {
    fn try_match(&mut self, ctx: &mut DefaultContext) -> Option<BenchToken> {
        ctx.advance().map(BenchToken::Unknown)
    }
    fn priority(&self) -> i32 {
        0
    }
    fn quick_check(&self, _first_char: Option<char>) -> Option<bool> {
        Some(true)
    }
}

// --- Data Generation ---

fn generate_english(size_kb: usize) -> String {
    let words = [
        "function",
        "let",
        "var",
        "const",
        "if",
        "else",
        "while",
        "return",
        "import",
        "export",
        "class",
        "interface",
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

fn generate_chinese(size_kb: usize) -> String {
    let words = [
        "定义", "变量", "如果", "否则", "循环", "返回", "类", "接口", "模块", "导入",
    ];
    let mut s = String::with_capacity(size_kb * 1024);
    while s.len() < size_kb * 1024 {
        for w in words {
            s.push_str(w);
            s.push(' ');
            s.push_str("变量名_甲");
            s.push(' ');
            s.push('=');
            s.push(' ');
            s.push_str("67890");
            s.push_str(";\n");
        }
    }
    s
}

fn generate_emoji(size_kb: usize) -> String {
    // Emoji variable names are valid in some languages, let's treat them as Identifiers here if is_alphabetic allows (it usually doesn't for most emojis, but let's mix them as "Unknown" or "Ident" depending on rule)
    // Rust's is_alphabetic() usually is false for Emoji. So they might fall to UnknownRule.
    // Let's check: '😀'.is_alphabetic() is false.
    // So Emojis will hit UnknownRule in our set, which is fine, it still tests Lexer throughput.
    let emojis = ["😀", "🚀", "🦀", "💻", "🔥", "✨", "🎉", "📦"];
    let mut s = String::with_capacity(size_kb * 1024);
    while s.len() < size_kb * 1024 {
        for e in emojis {
            s.push_str(e);
            s.push_str(" + ");
        }
        s.push('\n');
    }
    s
}

fn generate_mixed(size_kb: usize) -> String {
    let mut s = String::with_capacity(size_kb * 1024);
    let eng = generate_english(1);
    let cn = generate_chinese(1);
    let emo = generate_emoji(1);

    while s.len() < size_kb * 1024 {
        s.push_str(&eng);
        s.push_str(&cn);
        s.push_str(&emo);
    }
    s
}

// --- Benchmarks ---

fn bench_lexer(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer_batch");

    // We target ~100KB for each test to get stable throughput
    let size_kb = 100;

    let english_text = generate_english(size_kb);
    let chinese_text = generate_chinese(size_kb);
    let emoji_text = generate_emoji(size_kb);
    let mixed_text = generate_mixed(size_kb);

    // English
    group.throughput(Throughput::Bytes(english_text.len() as u64));
    group.bench_function("english_100kb", |b| {
        b.iter(|| {
            let rules: Vec<Box<dyn LexingRule<DefaultContext, BenchToken>>> = vec![
                Box::new(WhitespaceRule),
                Box::new(NumberRule),
                Box::new(IdentifierRule),
                Box::new(OperatorRule),
                Box::new(UnknownRule),
            ];
            let mut lexer = Lexer::from_str(english_text.as_str(), rules);
            let _tokens: Vec<_> = lexer.tokenize();
        })
    });

    // Chinese
    group.throughput(Throughput::Bytes(chinese_text.len() as u64));
    group.bench_function("chinese_100kb", |b| {
        b.iter(|| {
            let rules: Vec<Box<dyn LexingRule<DefaultContext, BenchToken>>> = vec![
                Box::new(WhitespaceRule),
                Box::new(NumberRule),
                Box::new(IdentifierRule),
                Box::new(OperatorRule),
                Box::new(UnknownRule),
            ];
            let mut lexer = Lexer::from_str(chinese_text.as_str(), rules);
            let _tokens: Vec<_> = lexer.tokenize();
        })
    });

    // Emoji
    group.throughput(Throughput::Bytes(emoji_text.len() as u64));
    group.bench_function("emoji_100kb", |b| {
        b.iter(|| {
            let rules: Vec<Box<dyn LexingRule<DefaultContext, BenchToken>>> = vec![
                Box::new(WhitespaceRule),
                Box::new(NumberRule),
                Box::new(IdentifierRule),
                Box::new(OperatorRule),
                Box::new(UnknownRule),
            ];
            let mut lexer = Lexer::from_str(emoji_text.as_str(), rules);
            let _tokens: Vec<_> = lexer.tokenize();
        })
    });

    // Mixed
    group.throughput(Throughput::Bytes(mixed_text.len() as u64));
    group.bench_function("mixed_100kb", |b| {
        b.iter(|| {
            let rules: Vec<Box<dyn LexingRule<DefaultContext, BenchToken>>> = vec![
                Box::new(WhitespaceRule),
                Box::new(NumberRule),
                Box::new(IdentifierRule),
                Box::new(OperatorRule),
                Box::new(UnknownRule),
            ];
            let mut lexer = Lexer::from_str(mixed_text.as_str(), rules);
            let _tokens: Vec<_> = lexer.tokenize();
        })
    });

    group.finish();
}

criterion_group!(benches, bench_lexer);
criterion_main!(benches);
