use crate::lexer::context::LexContext;
use crate::lexer::traits::LexingRule;
use crate::position::Position;
use std::cmp::Reverse;

/// A simple token type for demonstration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimpleToken {
    Whitespace { value: String, position: Position },
    Newline { position: Position },
    Ident { value: String, position: Position },
    Number { value: String, position: Position },
    Operator { value: String, position: Position },
    Eof { position: Position },
}

impl crate::lexer::traits::LexToken for SimpleToken {
    fn position(&self) -> Option<Position> {
        match self {
            SimpleToken::Whitespace { position, .. }
            | SimpleToken::Newline { position, .. }
            | SimpleToken::Ident { position, .. }
            | SimpleToken::Number { position, .. }
            | SimpleToken::Operator { position, .. }
            | SimpleToken::Eof { position, .. } => Some(*position),
        }
    }

    fn is_eof(&self) -> bool {
        matches!(self, SimpleToken::Eof { .. })
    }

    fn is_newline(&self) -> bool {
        matches!(self, SimpleToken::Newline { .. })
    }

    fn is_whitespace(&self) -> bool {
        matches!(self, SimpleToken::Whitespace { .. })
    }

    fn is_indent(&self) -> bool {
        if let SimpleToken::Whitespace { value, .. } = self {
            value.starts_with('\t') || value.chars().all(|c| c == ' ')
        } else {
            false
        }
    }
}

/// A rule that matches whitespace (excluding newlines).
pub struct WhitespaceRule;

impl<'input, Ctx> LexingRule<'input, Ctx, SimpleToken> for WhitespaceRule
where
    Ctx: LexContext<'input>,
{
    fn try_match(&mut self, ctx: &mut Ctx) -> Option<SimpleToken> {
        let position = ctx.position();
        let value = ctx.consume_while(|c| c.is_whitespace() && c != '\n');

        if value.is_empty() {
            None
        } else {
            Some(SimpleToken::Whitespace {
                value: value.to_string(),
                position,
            })
        }
    }

    fn priority(&self) -> i32 {
        1 // Lower priority than keywords/identifiers
    }
}

/// A rule that matches newlines.
pub struct NewlineRule;

impl<'input, Ctx> LexingRule<'input, Ctx, SimpleToken> for NewlineRule
where
    Ctx: LexContext<'input>,
{
    fn try_match(&mut self, ctx: &mut Ctx) -> Option<SimpleToken> {
        if ctx.peek() == Some('\n') {
            let position = ctx.position();
            ctx.advance();
            Some(SimpleToken::Newline { position })
        } else {
            None
        }
    }

    fn priority(&self) -> i32 {
        2
    }
}

/// A rule that matches identifiers.
pub struct IdentRule;

impl<'input, Ctx> LexingRule<'input, Ctx, SimpleToken> for IdentRule
where
    Ctx: LexContext<'input>,
{
    fn try_match(&mut self, ctx: &mut Ctx) -> Option<SimpleToken> {
        let position = ctx.position();
        let first = ctx.peek()?;

        if first.is_alphabetic() || first == '_' {
            ctx.advance();
            let rest = ctx.consume_while(|c| c.is_alphanumeric() || c == '_');

            let mut value = String::with_capacity(1 + rest.len());
            value.push(first);
            value.push_str(rest);

            Some(SimpleToken::Ident { value, position })
        } else {
            None
        }
    }

    fn priority(&self) -> i32 {
        10 // Higher priority - identifiers should be matched before operators
    }
}

/// A rule that matches numbers.
pub struct NumberRule;

impl<'input, Ctx> LexingRule<'input, Ctx, SimpleToken> for NumberRule
where
    Ctx: LexContext<'input>,
{
    fn try_match(&mut self, ctx: &mut Ctx) -> Option<SimpleToken> {
        let position = ctx.position();
        let first = ctx.peek()?;

        if first.is_ascii_digit() {
            let mut value = String::new();
            value.push(first);
            ctx.advance();

            // Match remaining digits
            let rest = ctx.consume_while(|c| c.is_ascii_digit());
            value.push_str(rest);

            // Optionally match decimal part
            if ctx.peek() == Some('.')
                && ctx
                    .cursor()
                    .peek_str(2)
                    .chars()
                    .nth(1)
                    .is_some_and(|c| c.is_ascii_digit())
            {
                value.push('.');
                ctx.advance();
                let decimal = ctx.consume_while(|c| c.is_ascii_digit());
                value.push_str(decimal);
            }

            Some(SimpleToken::Number { value, position })
        } else {
            None
        }
    }

    fn priority(&self) -> i32 {
        10 // Same priority as identifiers
    }
}

/// A rule that matches operators and punctuation.
pub struct OperatorRule {
    operators: Vec<String>,
}

impl OperatorRule {
    pub fn new(operators: Vec<&str>) -> Self {
        Self {
            operators: operators.into_iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl Default for OperatorRule {
    fn default() -> Self {
        Self::new(vec![
            "==", "!=", "<=", ">=", "&&", "||", "+", "-", "*", "/", "%", "=", "<", ">", "!", "(",
            ")", "{", "}", "[", "]", ".", ",", ";", ":",
        ])
    }
}

impl<'input, Ctx> LexingRule<'input, Ctx, SimpleToken> for OperatorRule
where
    Ctx: LexContext<'input>,
{
    fn try_match(&mut self, ctx: &mut Ctx) -> Option<SimpleToken> {
        // Sort operators by length (longest first) to match multi-character operators
        let mut sorted_ops: Vec<_> = self.operators.iter().collect();
        sorted_ops.sort_by_key(|op| Reverse(op.len()));

        for op in sorted_ops {
            let peeked = ctx.cursor().peek_str(op.len());
            if peeked == op {
                let position = ctx.position();
                for _ in 0..op.len() {
                    ctx.advance();
                }
                return Some(SimpleToken::Operator {
                    value: op.clone(),
                    position,
                });
            }
        }

        None
    }

    fn priority(&self) -> i32 {
        5 // Medium priority
    }
}

/// A rule that matches EOF.
pub struct EofRule;

impl<'input, Ctx> LexingRule<'input, Ctx, SimpleToken> for EofRule
where
    Ctx: LexContext<'input>,
{
    fn try_match(&mut self, ctx: &mut Ctx) -> Option<SimpleToken> {
        if ctx.is_eof() {
            Some(SimpleToken::Eof {
                position: ctx.position(),
            })
        } else {
            None
        }
    }

    fn priority(&self) -> i32 {
        -1 // Lowest priority - only match when nothing else matches
    }
}

// Helper function to create default rules
pub fn default_rules<'input>() -> Vec<
    Box<
        dyn LexingRule<'input, crate::lexer::context::DefaultContext<'input>, SimpleToken> + 'input,
    >,
> {
    vec![
        Box::new(NewlineRule),
        Box::new(WhitespaceRule),
        Box::new(IdentRule),
        Box::new(NumberRule),
        Box::new(OperatorRule::default()),
        Box::new(EofRule),
    ]
}
