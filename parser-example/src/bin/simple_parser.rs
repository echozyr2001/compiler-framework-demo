//! 简单的解析器示例
//!
//! 本示例展示了如何使用 parser-framework 的基本功能：
//! 1. 定义 Token 类型
//! 2. 定义 AST 节点类型
//! 3. 实现解析规则
//! 4. 使用 Parser 解析 token 流
//!
//! 这个示例解析简单的算术表达式，只支持两个数字和一个运算符。

use parser_framework::{AstNode, DefaultContext, ParseContext, Parser, ParsingRule, Position};

type SimpleParserRules =
    Vec<Box<dyn ParsingRule<DefaultContext<SimpleToken>, SimpleToken, SimpleExpr>>>;

// ============================================================================
// Token 定义（简化版，直接从 token 流开始）
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum SimpleToken {
    Number { value: i32, position: Position },
    Plus { position: Position },
    Minus { position: Position },
    Eof { position: Position },
}

// ============================================================================
// AST 节点定义
// ============================================================================

/// 简单的表达式 AST 节点
#[derive(Debug, Clone, PartialEq)]
pub enum SimpleExpr {
    /// 数字字面量
    Number { value: i32, position: Position },
    /// 二元运算：左操作数 运算符 右操作数
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
// Parser 规则
// ============================================================================

/// 解析数字字面量
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
        10 // 高优先级，数字是最基本的表达式
    }

    fn quick_check(&self, current_token: Option<&SimpleToken>) -> Option<bool> {
        Some(matches!(current_token?, SimpleToken::Number { .. }))
    }
}

/// 解析二元运算表达式：数字 运算符 数字
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
        // 解析左操作数（必须是数字）
        let left_token = ctx.peek()?.clone();
        let left = if let SimpleToken::Number { value, position } = left_token {
            ctx.advance();
            SimpleExpr::Number { value, position }
        } else {
            return None;
        };

        // 检查并消费运算符
        let op_token = ctx.peek()?.clone();
        let op_position = match &op_token {
            SimpleToken::Plus { position } | SimpleToken::Minus { position } => *position,
            _ => return None,
        };

        // 检查运算符是否匹配
        let matches_op = matches!(
            (&self.op_token, &op_token),
            (SimpleToken::Plus { .. }, SimpleToken::Plus { .. })
                | (SimpleToken::Minus { .. }, SimpleToken::Minus { .. })
        );

        if !matches_op {
            return None;
        }

        ctx.advance(); // 消费运算符

        // 解析右操作数（必须是数字）
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
        15 // 较高优先级，二元运算应该先尝试（因为它需要匹配完整的表达式）
    }

    fn quick_check(&self, current_token: Option<&SimpleToken>) -> Option<bool> {
        // 二元运算需要检查第一个 token 是否为数字
        Some(matches!(current_token?, SimpleToken::Number { .. }))
    }
}

// ============================================================================
// 主程序
// ============================================================================

fn main() {
    println!("=== 简单解析器示例 ===\n");
    println!("本示例展示如何使用 parser-framework 解析简单的算术表达式\n");

    // 示例 1: 解析单个数字
    println!("【示例 1】解析单个数字:");
    println!("{}", "=".repeat(50));

    let tokens1 = vec![SimpleToken::Number {
        value: 42,
        position: Position::new(),
    }];

    let rules1: SimpleParserRules = vec![Box::new(NumberRule)];

    let context1 = DefaultContext::from_token_iter(tokens1);
    let mut parser1 = Parser::new(context1, rules1);

    if let Some(ast) = parser1.parse_one() {
        println!("输入: 42");
        println!("AST: {:?}", ast);
        println!("成功解析为: {:?}", ast);
    } else {
        println!("解析失败");
    }

    println!("\n");

    // 示例 2: 解析加法表达式
    println!("【示例 2】解析加法表达式:");
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
        Box::new(NumberRule), // NumberRule 作为后备，用于解析单个数字
    ];

    let context2 = DefaultContext::from_token_iter(tokens2);
    let mut parser2 = Parser::new(context2, rules2);

    if let Some(ast) = parser2.parse_one() {
        println!("输入: 3 + 4");
        println!("AST: {:#?}", ast);
        match ast {
            SimpleExpr::Binary {
                op, left, right, ..
            } => {
                println!(
                    "成功解析为: {:?} {} {:?}",
                    left,
                    match op {
                        Op::Add => "+",
                        Op::Subtract => "-",
                    },
                    right
                );
            }
            _ => println!("解析结果: {:?}", ast),
        }
    } else {
        println!("解析失败");
    }

    println!("\n");

    // 示例 3: 解析减法表达式
    println!("【示例 3】解析减法表达式:");
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
        println!("输入: 10 - 3");
        println!("AST: {:#?}", ast);
        match ast {
            SimpleExpr::Binary {
                op, left, right, ..
            } => {
                println!(
                    "成功解析为: {:?} {} {:?}",
                    left,
                    match op {
                        Op::Add => "+",
                        Op::Subtract => "-",
                    },
                    right
                );
            }
            _ => println!("解析结果: {:?}", ast),
        }
    } else {
        println!("解析失败");
    }

    println!("\n");
    println!("=== 示例说明 ===");
    println!("1. 定义 Token 类型: SimpleToken");
    println!("2. 定义 AST 节点类型: SimpleExpr (实现 AstNode trait)");
    println!("3. 实现解析规则: NumberRule, BinaryRule (实现 ParsingRule trait)");
    println!("4. 使用 Parser::new() 创建解析器");
    println!("5. 调用 parser.parse_one() 解析 AST");
    println!("\n关键点:");
    println!("- 规则按优先级排序，高优先级规则先尝试");
    println!("- quick_check() 可以快速跳过不匹配的规则");
    println!("- 规则失败时会自动恢复检查点（checkpoint）");
    println!("- Context 提供 peek(), advance(), checkpoint(), restore() 等方法");
}
