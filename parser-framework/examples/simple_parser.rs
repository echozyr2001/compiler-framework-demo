//! Minimal parser example.
//!
//! Demonstrates the parser-framework workflow:
//! 1. Define tokens.
//! 2. Define AST nodes.
//! 3. Implement parsing rules.
//! 4. Run the parser over a token stream.
//!
//! The example parses simple arithmetic expressions with two numbers and one operator.

use parser_framework::{AstNode, DefaultContext, ParseContext, Parser, ParsingRule, Position};

type SimpleParserRules =
    Vec<Box<dyn ParsingRule<DefaultContext<SimpleToken>, SimpleToken, SimpleExpr>>>;

// ============================================================================
// Token definition (simplified: we start from a ready-made token stream)
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum SimpleToken {
    Number { value: i32, position: Position },
    Plus { position: Position },
    Minus { position: Position },
    Eof { position: Position },
}

// ============================================================================
// AST definition
// ============================================================================

/// Simple expression nodes.
#[derive(Debug, Clone, PartialEq)]
pub enum SimpleExpr {
    /// Numeric literal.
    Number { value: i32, position: Position },
    /// Binary operation (lhs, operator, rhs).
    Binary {
        op: Op,
        left: Box<SimpleExpr>,
        right: Box<SimpleExpr>,
        position: Position,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Add,
    Subtract,
}

impl AstNode for SimpleExpr {
    fn position(&self) -> Option<Position> {
        Some(match self {
            SimpleExpr::Number { position, .. } => *position,
            SimpleExpr::Binary { position, .. } => *position,
        })
    }
}

// ============================================================================
// Parser rules
// ============================================================================

/// Parses numeric literals.
struct NumberRule;

impl<Ctx> ParsingRule<Ctx, SimpleToken, SimpleExpr> for NumberRule
where
    Ctx: ParseContext<SimpleToken>,
{
    fn try_parse(&mut self, ctx: &mut Ctx) -> Option<SimpleExpr> {
        let token = ctx.peek()?.clone();
        if let SimpleToken::Number { value, position } = token {
            ctx.advance();
            Some(SimpleExpr::Number { value, position })
        } else {
            None
        }
    }

    fn priority(&self) -> i32 {
        10 // Higher priority: numbers are the base case.
    }

    fn quick_check(&self, current_token: Option<&SimpleToken>) -> Option<bool> {
        Some(matches!(current_token?, SimpleToken::Number { .. }))
    }
}

/// Parses binary expressions of the shape `number op number`.
struct BinaryRule {
    op_token: SimpleToken,
    op: Op,
}

impl BinaryRule {
    fn new(op_token: SimpleToken, op: Op) -> Self {
        Self { op_token, op }
    }
}

impl<Ctx> ParsingRule<Ctx, SimpleToken, SimpleExpr> for BinaryRule
where
    Ctx: ParseContext<SimpleToken>,
{
    fn try_parse(&mut self, ctx: &mut Ctx) -> Option<SimpleExpr> {
        // Parse the left operand (must be a number).
        let left_token = ctx.peek()?.clone();
        let left = if let SimpleToken::Number { value, position } = left_token {
            ctx.advance();
            SimpleExpr::Number { value, position }
        } else {
            return None;
        };

        // Inspect and consume the operator.
        let op_token = ctx.peek()?.clone();
        let op_position = match &op_token {
            SimpleToken::Plus { position } | SimpleToken::Minus { position } => *position,
            _ => return None,
        };

        // Ensure the operator matches the rule we were configured with.
        let matches_op = matches!(
            (&self.op_token, &op_token),
            (SimpleToken::Plus { .. }, SimpleToken::Plus { .. })
                | (SimpleToken::Minus { .. }, SimpleToken::Minus { .. })
        );

        if !matches_op {
            return None;
        }

        ctx.advance(); // consume the operator

        // Parse the right operand (must be a number).
        let right_token = ctx.peek()?.clone();
        let right = if let SimpleToken::Number { value, position } = right_token {
            ctx.advance();
            SimpleExpr::Number { value, position }
        } else {
            return None;
        };

        Some(SimpleExpr::Binary {
            op: self.op,
            left: Box::new(left),
            right: Box::new(right),
            position: op_position,
        })
    }

    fn priority(&self) -> i32 {
        15 // Higher priority: needs to match the entire expression.
    }

    fn quick_check(&self, current_token: Option<&SimpleToken>) -> Option<bool> {
        // Binary expressions require the first token to be a number.
        Some(matches!(current_token?, SimpleToken::Number { .. }))
    }
}

// ============================================================================
// Example program
// ============================================================================

fn main() {
    println!("=== Simple Parser Example ===\n");
    println!("Demonstrates how parser-framework parses basic arithmetic expressions.\n");

    // Example 1: single number
    println!("[Example 1] Parsing a single number:");
    println!("{}", "=".repeat(50));

    let tokens1 = vec![SimpleToken::Number {
        value: 42,
        position: Position::new(),
    }];

    let rules1: SimpleParserRules = vec![Box::new(NumberRule)];

    let context1 = DefaultContext::from_token_iter(tokens1);
    let mut parser1 = Parser::new(context1, rules1);

    if let Some(ast) = parser1.parse_one() {
        println!("Input: 42");
        println!("AST: {:?}", ast);
        println!("Successfully parsed: {:?}", ast);
    } else {
        println!("Parse failed");
    }

    println!("\n");

    // Example 2: addition
    println!("[Example 2] Parsing addition:");
    println!("{}", "=".repeat(50));

    let tokens2 = vec![
        SimpleToken::Number {
            value: 3,
            position: Position {
                line: 1,
                column: 1,
                offset: 0,
            },
        },
        SimpleToken::Plus {
            position: Position {
                line: 1,
                column: 3,
                offset: 2,
            },
        },
        SimpleToken::Number {
            value: 4,
            position: Position {
                line: 1,
                column: 5,
                offset: 4,
            },
        },
    ];

    let rules2: SimpleParserRules = vec![
        Box::new(BinaryRule::new(
            SimpleToken::Plus {
                position: Position::new(),
            },
            Op::Add,
        )),
        Box::new(NumberRule), // fallback to parse standalone numbers
    ];

    let context2 = DefaultContext::from_token_iter(tokens2);
    let mut parser2 = Parser::new(context2, rules2);

    if let Some(ast) = parser2.parse_one() {
        println!("Input: 3 + 4");
        println!("AST: {:#?}", ast);
        match ast {
            SimpleExpr::Binary {
                op, left, right, ..
            } => {
                println!(
                    "Result: {:?} {} {:?}",
                    left,
                    match op {
                        Op::Add => "+",
                        Op::Subtract => "-",
                    },
                    right
                );
            }
            _ => println!("Parsed AST: {:?}", ast),
        }
    } else {
        println!("Parse failed");
    }

    println!("\n");

    // Example 3: subtraction
    println!("[Example 3] Parsing subtraction:");
    println!("{}", "=".repeat(50));

    let tokens3 = vec![
        SimpleToken::Number {
            value: 10,
            position: Position {
                line: 1,
                column: 1,
                offset: 0,
            },
        },
        SimpleToken::Minus {
            position: Position {
                line: 1,
                column: 4,
                offset: 3,
            },
        },
        SimpleToken::Number {
            value: 3,
            position: Position {
                line: 1,
                column: 6,
                offset: 5,
            },
        },
    ];

    let rules3: SimpleParserRules = vec![
        Box::new(BinaryRule::new(
            SimpleToken::Minus {
                position: Position::new(),
            },
            Op::Subtract,
        )),
        Box::new(NumberRule),
    ];

    let context3 = DefaultContext::from_token_iter(tokens3);
    let mut parser3 = Parser::new(context3, rules3);

    if let Some(ast) = parser3.parse_one() {
        println!("Input: 10 - 3");
        println!("AST: {:#?}", ast);
        match ast {
            SimpleExpr::Binary {
                op, left, right, ..
            } => {
                println!(
                    "Result: {:?} {} {:?}",
                    left,
                    match op {
                        Op::Add => "+",
                        Op::Subtract => "-",
                    },
                    right
                );
            }
            _ => println!("Parsed AST: {:?}", ast),
        }
    } else {
        println!("Parse failed");
    }
}
