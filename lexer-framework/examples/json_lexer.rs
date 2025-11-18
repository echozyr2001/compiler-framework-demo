//! JSON 词法分析器示例
//! 展示如何使用 lexer-framework 定义 JSON 风格的 Token 和规则

use lexer_framework::{DefaultContext, LexContext, LexToken, Lexer, LexingRule, Position};

/// JSON Token 类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonToken {
    String { value: String, position: Position },
    Number { value: String, position: Position },
    Boolean { value: bool, position: Position },
    Null { position: Position },
    LeftBrace { position: Position },
    RightBrace { position: Position },
    LeftBracket { position: Position },
    RightBracket { position: Position },
    Comma { position: Position },
    Colon { position: Position },
    Whitespace { value: String, position: Position },
    Eof { position: Position },
}

impl LexToken for JsonToken {
    fn position(&self) -> Option<Position> {
        Some(match self {
            JsonToken::String { position, .. }
            | JsonToken::Number { position, .. }
            | JsonToken::Boolean { position, .. }
            | JsonToken::Null { position }
            | JsonToken::LeftBrace { position }
            | JsonToken::RightBrace { position }
            | JsonToken::LeftBracket { position }
            | JsonToken::RightBracket { position }
            | JsonToken::Comma { position }
            | JsonToken::Colon { position }
            | JsonToken::Whitespace { position, .. }
            | JsonToken::Eof { position } => *position,
        })
    }

    fn is_eof(&self) -> bool {
        matches!(self, JsonToken::Eof { .. })
    }

    fn is_newline(&self) -> bool {
        false
    }

    fn is_whitespace(&self) -> bool {
        matches!(self, JsonToken::Whitespace { .. })
    }

    fn is_indent(&self) -> bool {
        false
    }
}

/// 匹配 JSON 字符串（支持转义字符）
pub struct StringRule;

impl<Ctx> LexingRule<Ctx, JsonToken> for StringRule
where
    Ctx: LexContext,
{
    fn try_match(&mut self, ctx: &mut Ctx) -> Option<JsonToken> {
        if ctx.peek() != Some('"') {
            return None;
        }

        let position = ctx.position();
        ctx.advance(); // consume opening quote

        let mut value = String::new();
        let mut escape = false;

        loop {
            match ctx.peek() {
                None => return None, // Unterminated string
                Some('"') if !escape => {
                    ctx.advance(); // consume closing quote
                    break;
                }
                Some('\\') if !escape => {
                    escape = true;
                    ctx.advance();
                }
                Some(ch) => {
                    if escape {
                        match ch {
                            'n' => value.push('\n'),
                            't' => value.push('\t'),
                            'r' => value.push('\r'),
                            '\\' => value.push('\\'),
                            '"' => value.push('"'),
                            _ => {
                                value.push('\\');
                                value.push(ch);
                            }
                        }
                        escape = false;
                    } else {
                        value.push(ch);
                    }
                    ctx.advance();
                }
            }
        }

        Some(JsonToken::String { value, position })
    }

    fn priority(&self) -> i32 {
        15
    }
}

/// 匹配数字
pub struct NumberRule;

impl<Ctx> LexingRule<Ctx, JsonToken> for NumberRule
where
    Ctx: LexContext,
{
    fn try_match(&mut self, ctx: &mut Ctx) -> Option<JsonToken> {
        let position = ctx.position();
        let checkpoint = ctx.checkpoint();
        let first = ctx.peek()?;

        let mut has_digit = false;
        let mut value = String::new();

        // Handle optional negative sign
        if first == '-' {
            value.push('-');
            ctx.advance();
        } else if !first.is_ascii_digit() {
            return None;
        }

        // Match integer part - must have at least one digit
        if let Some(ch) = ctx.peek() {
            if ch.is_ascii_digit() {
                has_digit = true;
                let int_part = ctx.consume_while(|c| c.is_ascii_digit());
                value.push_str(&int_part);
            } else if first == '-' {
                // If we consumed '-' but there's no digit, restore and fail
                ctx.restore(checkpoint);
                return None;
            }
        } else if first == '-' {
            // If we consumed '-' but reached EOF, restore and fail
            ctx.restore(checkpoint);
            return None;
        }

        // Must have at least one digit
        if !has_digit {
            ctx.restore(checkpoint);
            return None;
        }

        // Optionally match decimal part
        if ctx.peek() == Some('.') {
            if let Some(next) = ctx.cursor().peek_str(2).chars().nth(1) {
                if next.is_ascii_digit() {
                    value.push('.');
                    ctx.advance();
                    let decimal = ctx.consume_while(|c| c.is_ascii_digit());
                    value.push_str(&decimal);
                }
            }
        }

        // Optionally match exponent
        if ctx.peek() == Some('e') || ctx.peek() == Some('E') {
            let exp_checkpoint = ctx.checkpoint();
            value.push(ctx.advance()?);
            if let Some(sign) = ctx.peek() {
                if sign == '+' || sign == '-' {
                    value.push(ctx.advance()?);
                }
            }
            let exp = ctx.consume_while(|c| c.is_ascii_digit());
            if exp.is_empty() {
                ctx.restore(exp_checkpoint);
                // Remove 'e' or 'E' from value
                value.pop();
            } else {
                value.push_str(&exp);
            }
        }

        Some(JsonToken::Number { value, position })
    }

    fn priority(&self) -> i32 {
        14
    }
}

/// 匹配布尔值和 null
pub struct KeywordRule;

impl<Ctx> LexingRule<Ctx, JsonToken> for KeywordRule
where
    Ctx: LexContext,
{
    fn try_match(&mut self, ctx: &mut Ctx) -> Option<JsonToken> {
        let position = ctx.position();
        let remaining = ctx.cursor().remaining();

        if remaining.starts_with("true") {
            for _ in 0..4 {
                ctx.advance();
            }
            return Some(JsonToken::Boolean {
                value: true,
                position,
            });
        } else if remaining.starts_with("false") {
            for _ in 0..5 {
                ctx.advance();
            }
            return Some(JsonToken::Boolean {
                value: false,
                position,
            });
        } else if remaining.starts_with("null") {
            for _ in 0..4 {
                ctx.advance();
            }
            return Some(JsonToken::Null { position });
        }

        None
    }

    fn priority(&self) -> i32 {
        13
    }
}

/// 匹配标点符号
pub struct PunctuationRule;

impl<Ctx> LexingRule<Ctx, JsonToken> for PunctuationRule
where
    Ctx: LexContext,
{
    fn try_match(&mut self, ctx: &mut Ctx) -> Option<JsonToken> {
        use JsonToken::*;
        let ch = ctx.peek()?;
        let position = ctx.position();

        let token = match ch {
            '{' => Some(LeftBrace { position }),
            '}' => Some(RightBrace { position }),
            '[' => Some(LeftBracket { position }),
            ']' => Some(RightBracket { position }),
            ',' => Some(Comma { position }),
            ':' => Some(Colon { position }),
            _ => return None,
        };

        ctx.advance();
        token
    }

    fn priority(&self) -> i32 {
        5
    }
}

/// 匹配空白字符
pub struct WhitespaceRule;

impl<Ctx> LexingRule<Ctx, JsonToken> for WhitespaceRule
where
    Ctx: LexContext,
{
    fn try_match(&mut self, ctx: &mut Ctx) -> Option<JsonToken> {
        let position = ctx.position();
        let value = ctx.consume_while(|c| c.is_whitespace());

        if value.is_empty() {
            None
        } else {
            Some(JsonToken::Whitespace {
                value: value.to_string(),
                position,
            })
        }
    }

    fn priority(&self) -> i32 {
        1
    }
}

/// 匹配 EOF
pub struct EofRule;

impl<Ctx> LexingRule<Ctx, JsonToken> for EofRule
where
    Ctx: LexContext,
{
    fn try_match(&mut self, ctx: &mut Ctx) -> Option<JsonToken> {
        if ctx.is_eof() {
            Some(JsonToken::Eof {
                position: ctx.position(),
            })
        } else {
            None
        }
    }

    fn priority(&self) -> i32 {
        -1
    }
}

fn json_rules() -> Vec<Box<dyn LexingRule<DefaultContext, JsonToken>>> {
    vec![
        Box::new(StringRule),
        Box::new(NumberRule),
        Box::new(KeywordRule),
        Box::new(PunctuationRule),
        Box::new(WhitespaceRule),
        Box::new(EofRule),
    ]
}

fn main() {
    let json = r#"{
        "name": "Alice",
        "age": 30,
        "active": true,
        "tags": ["developer", "rust"],
        "address": null
    }"#;

    println!("Tokenizing JSON:");
    println!("{}\n", json);

    let rules = json_rules();
    let mut lexer = Lexer::from_str(json, rules);

    println!("Tokens:");
    for (i, token) in lexer.tokenize().iter().enumerate() {
        match token {
            JsonToken::Whitespace { .. } => {
                // Skip whitespace in output for readability
                continue;
            }
            _ => println!("  {}: {:?}", i, token),
        }
    }
}
