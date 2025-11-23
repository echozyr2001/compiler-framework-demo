//! Calculator lexer example.
//! Demonstrates how to define calculator-style tokens and rules with lexer-framework.

use lexer_framework::{DefaultContext, LexContext, LexToken, Lexer, LexingRule, Position};

/// Token definition used by the calculator example (kept intentionally small).
#[derive(Debug, Clone, PartialEq)]
pub enum CalcToken {
    Number { value: f64, position: Position },
    Plus { position: Position },
    Minus { position: Position },
    Multiply { position: Position },
    Divide { position: Position },
    Power { position: Position },
    LeftParen { position: Position },
    RightParen { position: Position },
    Whitespace { position: Position },
    Eof { position: Position },
}

impl LexToken for CalcToken {
    fn position(&self) -> Option<Position> {
        Some(match self {
            CalcToken::Number { position, .. }
            | CalcToken::Plus { position }
            | CalcToken::Minus { position }
            | CalcToken::Multiply { position }
            | CalcToken::Divide { position }
            | CalcToken::Power { position }
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
        false
    }

    fn is_whitespace(&self) -> bool {
        matches!(self, CalcToken::Whitespace { .. })
    }

    fn is_indent(&self) -> bool {
        false
    }
}

/// Matches floating-point numbers.
pub struct NumberRule;

impl<Ctx> LexingRule<Ctx, CalcToken> for NumberRule
where
    Ctx: LexContext,
{
    fn try_match(&mut self, ctx: &mut Ctx) -> Option<CalcToken> {
        let position = ctx.position();
        let first = ctx.peek()?;

        if !first.is_ascii_digit() && first != '.' {
            return None;
        }

        let mut value_str = String::new();
        let mut has_digit = false;

        // Parse the integer part.
        if first.is_ascii_digit() {
            value_str.push(first);
            has_digit = true;
            ctx.advance();
            let int_part = ctx.consume_while(|c| c.is_ascii_digit());
            value_str.push_str(int_part.as_ref());
        }

        // Parse the fractional part.
        if ctx.peek() == Some('.') {
            value_str.push('.');
            ctx.advance();
            let decimal = ctx.consume_while(|c| c.is_ascii_digit());
            if !decimal.is_empty() {
                has_digit = true;
            }
            value_str.push_str(decimal.as_ref());
        }

        // Require at least one digit overall.
        if !has_digit {
            return None;
        }

        // Attempt to parse the collected literal as f64.
        if let Ok(value) = value_str.parse::<f64>() {
            Some(CalcToken::Number { value, position })
        } else {
            None
        }
    }

    fn priority(&self) -> i32 {
        15
    }
}

/// Matches operators.
///
/// Two approaches are highlighted:
/// 1. `match` statements — ideal for a small fixed operator set (fast, compile-time optimization).
/// 2. Lookup tables — handy when operators are user-configurable at runtime (more flexible, slightly slower).
///
/// This example sticks with `match` because calculator operators are fixed.
/// For a dynamic configuration, see `json_lexer.rs` and its `PunctuationRule`.
pub struct OperatorRule;

impl<Ctx> LexingRule<Ctx, CalcToken> for OperatorRule
where
    Ctx: LexContext,
{
    /// Quick check: only operator characters should reach `try_match`.
    ///
    /// Optimization: skip this rule for letters/digits/whitespace to avoid unnecessary checkpoints.
    fn quick_check(&self, first_char: Option<char>) -> Option<bool> {
        match first_char? {
            '+' | '-' | '*' | '/' | '^' | '(' | ')' => Some(true), // could be an operator
            _ => Some(false), // letters/digits/whitespace are never operators
        }
    }

    fn try_match(&mut self, ctx: &mut Ctx) -> Option<CalcToken> {
        use CalcToken::*;
        let ch = ctx.peek()?;
        let position = ctx.position();

        // Approach 1: `match` (current implementation)
        // Pros: compile-time optimization, good performance, concise code.
        // Cons: operator set fixed at compile time.
        let token = match ch {
            '+' => Some(Plus { position }),
            '-' => Some(Minus { position }),
            '*' => Some(Multiply { position }),
            '/' => Some(Divide { position }),
            '^' => Some(Power { position }),
            '(' => Some(LeftParen { position }),
            ')' => Some(RightParen { position }),
            _ => return None,
        };

        ctx.advance();
        token

        // Approach 2: vector-based mapping (optional)
        // pub struct OperatorRule {
        //     mappings: Vec<(char, fn(Position) -> CalcToken)>,
        // }
        // Iterate `mappings` in `try_match`.
        // Pros: runtime-configurable operator set.
        // Cons: requires a lookup per match, slightly slower.
    }

    fn priority(&self) -> i32 {
        10
    }
}

/// Matches whitespace.
pub struct WhitespaceRule;

impl<Ctx> LexingRule<Ctx, CalcToken> for WhitespaceRule
where
    Ctx: LexContext,
{
    fn try_match(&mut self, ctx: &mut Ctx) -> Option<CalcToken> {
        if ctx.peek().is_some_and(|c| c.is_whitespace()) {
            let position = ctx.position();
            ctx.consume_while(|c| c.is_whitespace());
            Some(CalcToken::Whitespace { position })
        } else {
            None
        }
    }

    fn priority(&self) -> i32 {
        1
    }
}

/// Matches EOF.
pub struct EofRule;

impl<Ctx> LexingRule<Ctx, CalcToken> for EofRule
where
    Ctx: LexContext,
{
    fn try_match(&mut self, ctx: &mut Ctx) -> Option<CalcToken> {
        if ctx.is_eof() {
            Some(CalcToken::Eof {
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

fn calc_rules() -> Vec<Box<dyn LexingRule<DefaultContext, CalcToken>>> {
    vec![
        Box::new(NumberRule),
        Box::new(OperatorRule),
        Box::new(WhitespaceRule),
        Box::new(EofRule),
    ]
}

fn main() {
    let expressions = vec!["3 + 4", "2 * 3.14", "(1 + 2) * 3", "2 ^ 8", "10 / 2.5"];

    for expr in expressions {
        println!("Expression: {}", expr);

        let rules = calc_rules();
        let mut lexer = Lexer::from_str(expr, rules);

        println!("Tokens:");
        for token in lexer.tokenize() {
            match token {
                CalcToken::Whitespace { .. } => {
                    // Skip whitespace
                    continue;
                }
                _ => println!("  {:?}", token),
            }
        }
        println!();
    }
}
