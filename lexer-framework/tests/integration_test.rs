//! 集成测试：测试框架的完整使用场景

use lexer_framework::{DefaultContext, LexContext, LexToken, Lexer, LexingRule, Position};

type RuleSet<Tok> = Vec<Box<dyn LexingRule<DefaultContext, Tok>>>;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Keyword { value: String, position: Position },
    Ident { value: String, position: Position },
    Number { value: i64, position: Position },
    Operator { value: String, position: Position },
    Whitespace { position: Position },
    Eof { position: Position },
}

impl LexToken for Token {
    fn position(&self) -> Option<Position> {
        Some(match self {
            Token::Keyword { position, .. }
            | Token::Ident { position, .. }
            | Token::Number { position, .. }
            | Token::Operator { position, .. }
            | Token::Whitespace { position }
            | Token::Eof { position } => *position,
        })
    }

    fn is_eof(&self) -> bool {
        matches!(self, Token::Eof { .. })
    }

    fn is_newline(&self) -> bool {
        false
    }

    fn is_whitespace(&self) -> bool {
        matches!(self, Token::Whitespace { .. })
    }

    fn is_indent(&self) -> bool {
        false
    }
}

// Keyword rule (with quick_check)
struct KeywordRule {
    keywords: Vec<&'static str>,
}

impl KeywordRule {
    fn new() -> Self {
        Self {
            keywords: vec!["let", "if", "else", "fn", "return"],
        }
    }
}

impl<Ctx> LexingRule<Ctx, Token> for KeywordRule
where
    Ctx: LexContext,
{
    fn quick_check(&self, first_char: Option<char>) -> Option<bool> {
        match first_char? {
            'a'..='z' | 'A'..='Z' => Some(true), // Might be a keyword
            _ => Some(false),
        }
    }

    fn try_match(&mut self, ctx: &mut Ctx) -> Option<Token> {
        let position = ctx.position();
        let remaining = ctx.cursor().remaining();

        for keyword in &self.keywords {
            if remaining.starts_with(keyword) {
                // Check that it's not part of a longer identifier
                if let Some(next_char) = remaining.chars().nth(keyword.len()) {
                    if next_char.is_alphanumeric() || next_char == '_' {
                        continue; // It's part of an identifier, not a keyword
                    }
                }

                // Consume the keyword
                for _ in 0..keyword.len() {
                    ctx.advance();
                }

                return Some(Token::Keyword {
                    value: keyword.to_string(),
                    position,
                });
            }
        }

        None
    }

    fn priority(&self) -> i32 {
        20 // High priority
    }
}

// Identifier rule
struct IdentRule;

impl<Ctx> LexingRule<Ctx, Token> for IdentRule
where
    Ctx: LexContext,
{
    fn quick_check(&self, first_char: Option<char>) -> Option<bool> {
        match first_char? {
            'a'..='z' | 'A'..='Z' | '_' => Some(true),
            ch if ch.is_alphanumeric() => Some(true), // Unicode alphanumeric
            _ => Some(false),
        }
    }

    fn try_match(&mut self, ctx: &mut Ctx) -> Option<Token> {
        let position = ctx.position();
        let first = ctx.peek()?;

        if first.is_alphabetic() || first == '_' {
            ctx.advance();
            let rest = ctx.consume_while(|c| c.is_alphanumeric() || c == '_');

            let mut value = String::with_capacity(1 + rest.len());
            value.push(first);
            value.push_str(rest.as_ref());

            Some(Token::Ident { value, position })
        } else {
            None
        }
    }

    fn priority(&self) -> i32 {
        15
    }
}

// Number rule
struct NumberRule;

impl<Ctx> LexingRule<Ctx, Token> for NumberRule
where
    Ctx: LexContext,
{
    fn quick_check(&self, first_char: Option<char>) -> Option<bool> {
        match first_char? {
            '0'..='9' => Some(true),
            _ => Some(false),
        }
    }

    fn try_match(&mut self, ctx: &mut Ctx) -> Option<Token> {
        let position = ctx.position();
        let first = ctx.peek()?;

        if first.is_ascii_digit() {
            ctx.advance();
            let digits = ctx.consume_while(|c| c.is_ascii_digit());

            let mut value_str = String::with_capacity(1 + digits.len());
            value_str.push(first);
            value_str.push_str(digits.as_ref());

            if let Ok(value) = value_str.parse::<i64>() {
                Some(Token::Number { value, position })
            } else {
                None
            }
        } else {
            None
        }
    }

    fn priority(&self) -> i32 {
        15
    }
}

// Operator rule
struct OperatorRule;

impl<Ctx> LexingRule<Ctx, Token> for OperatorRule
where
    Ctx: LexContext,
{
    fn quick_check(&self, first_char: Option<char>) -> Option<bool> {
        match first_char? {
            '+' | '-' | '*' | '/' | '=' | '<' | '>' | '!' | '(' | ')' | '{' | '}' | ';' | '&'
            | '|' => Some(true),
            _ => Some(false),
        }
    }

    fn try_match(&mut self, ctx: &mut Ctx) -> Option<Token> {
        let position = ctx.position();
        let ch = ctx.peek()?;

        let operators = vec!["==", "!=", "<=", ">=", "&&", "||"];
        let remaining = ctx.cursor().remaining();

        // Try multi-character operators first
        for op in &operators {
            if remaining.starts_with(op) {
                for _ in 0..op.len() {
                    ctx.advance();
                }
                return Some(Token::Operator {
                    value: op.to_string(),
                    position,
                });
            }
        }

        // Single character operators
        match ch {
            '+' | '-' | '*' | '/' | '=' | '<' | '>' | '!' | '(' | ')' | '{' | '}' | ';' => {
                ctx.advance();
                Some(Token::Operator {
                    value: ch.to_string(),
                    position,
                })
            }
            _ => None,
        }
    }

    fn priority(&self) -> i32 {
        10
    }
}

// Whitespace rule
struct WhitespaceRule;

impl<Ctx> LexingRule<Ctx, Token> for WhitespaceRule
where
    Ctx: LexContext,
{
    fn quick_check(&self, first_char: Option<char>) -> Option<bool> {
        match first_char? {
            ' ' | '\t' | '\r' | '\n' => Some(true),
            _ => Some(false),
        }
    }

    fn try_match(&mut self, ctx: &mut Ctx) -> Option<Token> {
        if ctx.peek().is_some_and(|c| c.is_whitespace()) {
            let position = ctx.position();
            ctx.consume_while(|c| c.is_whitespace());
            Some(Token::Whitespace { position })
        } else {
            None
        }
    }

    fn priority(&self) -> i32 {
        1 // Low priority
    }
}

// EOF rule
struct EofRule;

impl<Ctx> LexingRule<Ctx, Token> for EofRule
where
    Ctx: LexContext,
{
    fn try_match(&mut self, ctx: &mut Ctx) -> Option<Token> {
        if ctx.is_eof() {
            Some(Token::Eof {
                position: ctx.position(),
            })
        } else {
            None
        }
    }

    fn priority(&self) -> i32 {
        -1 // Lowest priority
    }
}

fn create_lexer(input: &str) -> Lexer<DefaultContext, Token> {
    let rules: RuleSet<Token> = vec![
        Box::new(KeywordRule::new()),
        Box::new(IdentRule),
        Box::new(NumberRule),
        Box::new(OperatorRule),
        Box::new(WhitespaceRule),
        Box::new(EofRule),
    ];
    Lexer::from_str(input, rules)
}

#[test]
fn test_integration_simple_statement() {
    let input = "let x = 42";
    let mut lexer = create_lexer(input);

    let tokens: Vec<_> = lexer
        .tokenize()
        .into_iter()
        .filter(|t| !t.is_whitespace())
        .collect();

    assert_eq!(tokens.len(), 4);
    assert!(matches!(&tokens[0], Token::Keyword { value, .. } if value == "let"));
    assert!(matches!(&tokens[1], Token::Ident { value, .. } if value == "x"));
    assert!(matches!(&tokens[2], Token::Operator { value, .. } if value == "="));
    assert!(matches!(&tokens[3], Token::Number { value, .. } if *value == 42));
}

#[test]
fn test_integration_keyword_vs_identifier() {
    let input = "let let_var = 10";
    let mut lexer = create_lexer(input);

    let tokens: Vec<_> = lexer
        .tokenize()
        .into_iter()
        .filter(|t| !t.is_whitespace())
        .collect();

    // First "let" should be a keyword
    assert!(matches!(&tokens[0], Token::Keyword { value, .. } if value == "let"));
    // "let_var" should be an identifier (not keyword)
    assert!(matches!(&tokens[1], Token::Ident { value, .. } if value == "let_var"));
}

#[test]
fn test_integration_multiline() {
    let input = "let x = 10\nlet y = 20";
    let mut lexer = create_lexer(input);

    let tokens: Vec<_> = lexer
        .tokenize()
        .into_iter()
        .filter(|t| !t.is_whitespace() && !t.is_eof())
        .collect();

    assert!(tokens.len() >= 8); // Should have at least 8 tokens
    assert!(matches!(&tokens[0], Token::Keyword { value, .. } if value == "let"));
}

#[test]
fn test_integration_complex_expression() {
    let input = "if x > 10 && y < 20 { return x + y }";
    let mut lexer = create_lexer(input);

    let tokens: Vec<_> = lexer
        .tokenize()
        .into_iter()
        .filter(|t| !t.is_whitespace())
        .collect();

    // Check for multi-character operators
    let has_gt = tokens
        .iter()
        .any(|t| matches!(t, Token::Operator { value, .. } if value == ">"));
    let has_and = tokens
        .iter()
        .any(|t| matches!(t, Token::Operator { value, .. } if value == "&&"));
    let has_lt = tokens
        .iter()
        .any(|t| matches!(t, Token::Operator { value, .. } if value == "<"));

    assert!(has_gt);
    assert!(has_and);
    assert!(has_lt);
}

#[test]
fn test_integration_unicode() {
    let input = "let 变量 = 42";
    let mut lexer = create_lexer(input);

    let tokens: Vec<_> = lexer
        .tokenize()
        .into_iter()
        .filter(|t| !t.is_whitespace())
        .collect();

    // Chinese identifier should be recognized
    assert!(tokens.len() >= 3);
    assert!(matches!(&tokens[0], Token::Keyword { value, .. } if value == "let"));
    if tokens.len() > 1 {
        assert!(matches!(&tokens[1], Token::Ident { value, .. } if value == "变量"));
    }
    // Find the number token (might be at different index due to whitespace)
    let number_token = tokens
        .iter()
        .find(|t| matches!(t, Token::Number { value: 42, .. }));
    assert!(number_token.is_some());
}

#[test]
fn test_integration_empty_input() {
    let mut lexer = create_lexer("");
    let tokens = lexer.tokenize();
    // Empty input should result in no tokens or just EOF
    assert!(tokens.is_empty() || tokens.iter().all(|t| t.is_eof()));
}

#[test]
fn test_integration_only_whitespace() {
    let mut lexer = create_lexer("   \t\n  ");
    let tokens: Vec<_> = lexer
        .tokenize()
        .into_iter()
        .filter(|t| !t.is_whitespace())
        .collect();
    // Should only have whitespace tokens or EOF
    assert!(tokens.is_empty() || tokens.iter().all(|t| t.is_eof()));
}

#[test]
fn test_integration_priority_order() {
    // Test that keywords are matched before identifiers
    let input = "let";
    let mut lexer = create_lexer(input);

    let tokens: Vec<_> = lexer
        .tokenize()
        .into_iter()
        .filter(|t| !t.is_whitespace())
        .collect();

    // "let" should be matched as keyword, not identifier
    assert!(matches!(&tokens[0], Token::Keyword { value, .. } if value == "let"));
}
