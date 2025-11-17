//! 计算器解析器示例
//!
//! 本示例展示了如何使用 parser-framework 和 lexer-framework 配合工作，
//! 实现一个计算器表达式解析器。
//!
//! 流程：
//! 1. 使用 lexer-framework 将输入文本转换为 token 流
//! 2. 使用 parser-framework 将 token 流解析为 AST
//! 3. 展示 AST 结构
//!
//! 支持的语法：
//! - 数字（整数和浮点数）
//! - 四则运算：+、-、*、/
//! - 幂运算：^
//! - 括号：()
//! - 一元运算符：负号（-5）
//!
//! 当前实现状态：
//! ✅ 基本二元运算：`3 + 4`、`2 * 3.14`、`10 / 2.5`、`2 ^ 8`
//! ✅ 一元运算符：`-5`
//! ⚠️  运算符优先级：`3 + 4 * 5` 无法正确解析（右操作数无法递归解析更高优先级运算符）
//! ⚠️  括号内完整表达式：`(1 + 2) * 3` 无法正确解析（GroupRule 无法递归解析括号内的二元运算）
//!
//! 限制说明：
//! 当前的 BinaryRule 和 GroupRule 实现使用 `parse_expression` 辅助函数来解析子表达式，
//! 但 `parse_expression` 只能解析基本表达式（数字、括号、一元运算符），无法递归解析
//! 完整的二元运算表达式。要完全支持运算符优先级和括号内的完整表达式，需要：
//! 1. 让 BinaryRule 的右操作数能够递归使用规则系统解析更高优先级的运算符
//! 2. 让 GroupRule 能够递归使用规则系统解析括号内的完整表达式
//!    这需要改进架构，让规则能够访问规则列表并递归创建 Parser。

#[cfg(feature = "streaming")]
use common_framework::{Inbound, Outbound, StreamingSignal};
#[cfg(feature = "streaming")]
use lexer_framework::TokenProducer;
use lexer_framework::{
    DefaultContext as LexContext, LexContext as LexContextTrait, LexToken, Lexer, LexingRule,
    Position as LexPosition,
};
#[cfg(feature = "streaming")]
use parser_framework::StreamingParseContext;
use parser_framework::{AstNode, DefaultContext, ParseContext, Parser, ParsingRule, Position};
#[cfg(feature = "streaming")]
use pipeline_core::Pipeline;

type CalcLexerRules = Vec<Box<dyn LexingRule<LexContext, CalcToken>>>;
type CalcParserRules = Vec<Box<dyn ParsingRule<DefaultContext<CalcToken>, CalcToken, Expr>>>;

fn build_parser_rules() -> CalcParserRules {
    build_parser_rules_for_ctx::<DefaultContext<CalcToken>>()
}

fn build_parser_rules_for_ctx<Ctx>() -> Vec<Box<dyn ParsingRule<Ctx, CalcToken, Expr>>>
where
    Ctx: ParseContext<CalcToken> + 'static,
{
    vec![
        Box::new(GroupRule),
        Box::new(UnaryRule),
        Box::new(BinaryRule::new(
            CalcToken::Power {
                position: LexPosition::new(),
            },
            BinaryOp::Power,
            12,
        )),
        Box::new(BinaryRule::new(
            CalcToken::Multiply {
                position: LexPosition::new(),
            },
            BinaryOp::Multiply,
            10,
        )),
        Box::new(BinaryRule::new(
            CalcToken::Divide {
                position: LexPosition::new(),
            },
            BinaryOp::Divide,
            10,
        )),
        Box::new(BinaryRule::new(
            CalcToken::Plus {
                position: LexPosition::new(),
            },
            BinaryOp::Add,
            5,
        )),
        Box::new(BinaryRule::new(
            CalcToken::Minus {
                position: LexPosition::new(),
            },
            BinaryOp::Subtract,
            5,
        )),
        Box::new(NumberParserRule),
    ]
}

// ============================================================================
// Token 定义（从 lexer-example 复制，但为了完整性包含在这里）
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum CalcToken {
    Number { value: f64, position: LexPosition },
    Plus { position: LexPosition },
    Minus { position: LexPosition },
    Multiply { position: LexPosition },
    Divide { position: LexPosition },
    Power { position: LexPosition },
    LeftParen { position: LexPosition },
    RightParen { position: LexPosition },
    Whitespace { position: LexPosition },
    Eof { position: LexPosition },
}

impl LexToken for CalcToken {
    fn position(&self) -> Option<LexPosition> {
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

// ============================================================================
// Lexer 规则（简化版，仅用于演示）
// ============================================================================

struct NumberRule;

impl<Ctx> LexingRule<Ctx, CalcToken> for NumberRule
where
    Ctx: LexContextTrait,
{
    fn try_match(&mut self, ctx: &mut Ctx) -> Option<CalcToken> {
        let position = ctx.position();
        let first = ctx.peek()?;

        if !first.is_ascii_digit() && first != '.' {
            return None;
        }

        let mut value_str = String::new();
        let mut has_digit = false;

        if first.is_ascii_digit() {
            value_str.push(first);
            has_digit = true;
            ctx.advance();
            let int_part = ctx.consume_while(|c| c.is_ascii_digit());
            value_str.push_str(int_part.as_ref());
        }

        if ctx.peek() == Some('.') {
            value_str.push('.');
            ctx.advance();
            let decimal = ctx.consume_while(|c| c.is_ascii_digit());
            if !decimal.is_empty() {
                has_digit = true;
            }
            value_str.push_str(decimal.as_ref());
        }

        if !has_digit {
            return None;
        }

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

struct OperatorRule;

impl<Ctx> LexingRule<Ctx, CalcToken> for OperatorRule
where
    Ctx: LexContextTrait,
{
    fn quick_check(&self, first_char: Option<char>) -> Option<bool> {
        match first_char? {
            '+' | '-' | '*' | '/' | '^' | '(' | ')' => Some(true),
            _ => Some(false),
        }
    }

    fn try_match(&mut self, ctx: &mut Ctx) -> Option<CalcToken> {
        let ch = ctx.peek()?;
        let position = ctx.position();

        let token = match ch {
            '+' => Some(CalcToken::Plus { position }),
            '-' => Some(CalcToken::Minus { position }),
            '*' => Some(CalcToken::Multiply { position }),
            '/' => Some(CalcToken::Divide { position }),
            '^' => Some(CalcToken::Power { position }),
            '(' => Some(CalcToken::LeftParen { position }),
            ')' => Some(CalcToken::RightParen { position }),
            _ => return None,
        };

        ctx.advance();
        token
    }

    fn priority(&self) -> i32 {
        10
    }
}

struct WhitespaceRule;

impl<Ctx> LexingRule<Ctx, CalcToken> for WhitespaceRule
where
    Ctx: LexContextTrait,
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

struct EofRule;

impl<Ctx> LexingRule<Ctx, CalcToken> for EofRule
where
    Ctx: LexContextTrait,
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

// ============================================================================
// AST 节点定义
// ============================================================================

/// 表达式 AST 节点
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// 数字字面量
    Number { value: f64, position: Position },
    /// 二元运算：左操作数 运算符 右操作数
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
        position: Position,
    },
    /// 一元运算：运算符 操作数（用于负号）
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
        position: Position,
    },
    /// 括号表达式
    Group { expr: Box<Expr>, position: Position },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Negate, // 负号
}

impl AstNode for Expr {
    fn position(&self) -> Option<Position> {
        Some(match self {
            Expr::Number { position, .. }
            | Expr::Binary { position, .. }
            | Expr::Unary { position, .. }
            | Expr::Group { position, .. } => *position,
        })
    }
}

// ============================================================================
// Parser 规则
// ============================================================================

/// 解析数字字面量（Parser 规则）
struct NumberParserRule;

impl<Ctx> ParsingRule<Ctx, CalcToken, Expr> for NumberParserRule
where
    Ctx: ParseContext<CalcToken>,
{
    fn try_parse(&mut self, ctx: &mut Ctx) -> Option<Expr> {
        let token = ctx.peek()?.clone();
        if let CalcToken::Number {
            value,
            position: lex_pos,
        } = token
        {
            // 转换 lexer Position 到 parser Position
            let position = Position {
                line: lex_pos.line,
                column: lex_pos.column,
                offset: lex_pos.offset,
            };
            ctx.advance();
            Some(Expr::Number { value, position })
        } else {
            None
        }
    }

    fn priority(&self) -> i32 {
        1 // 低优先级，数字作为原子表达式，应该在其他规则都失败后才尝试
    }

    fn quick_check(&self, current_token: Option<&CalcToken>) -> Option<bool> {
        Some(matches!(current_token?, CalcToken::Number { .. }))
    }
}

/// 解析括号表达式
struct GroupRule;

impl<Ctx> ParsingRule<Ctx, CalcToken, Expr> for GroupRule
where
    Ctx: ParseContext<CalcToken>,
{
    fn try_parse(&mut self, ctx: &mut Ctx) -> Option<Expr> {
        // 检查左括号
        let start_pos = if let Some(CalcToken::LeftParen { position: lex_pos }) = ctx.peek() {
            Position {
                line: lex_pos.line,
                column: lex_pos.column,
                offset: lex_pos.offset,
            }
        } else {
            return None;
        };

        ctx.advance(); // 消费左括号

        // 跳过空白
        while matches!(ctx.peek(), Some(CalcToken::Whitespace { .. })) {
            ctx.advance();
        }

        // 解析内部表达式（递归）
        // 为了支持括号内的完整表达式（如 "(1 + 2) * 3" 中的 "1 + 2"），
        // 我们需要递归解析括号内的完整表达式，包括二元运算
        //
        // 策略：先尝试解析基本表达式，然后检查后面是否有运算符需要处理
        // 如果有，递归解析完整的表达式

        let checkpoint_before_expr = ctx.checkpoint();

        // 先尝试解析基本表达式（数字、括号、一元运算符）
        let mut expr = if let Some(inner_expr) = parse_expression(ctx) {
            inner_expr
        } else {
            ctx.restore(checkpoint_before_expr);
            return None; // 括号内必须有表达式
        };

        // 跳过空白
        while matches!(ctx.peek(), Some(CalcToken::Whitespace { .. })) {
            ctx.advance();
        }

        // 检查后面是否有运算符需要处理（支持括号内的完整表达式）
        // 尝试解析所有可能的二元运算符（按优先级从高到低）
        let all_binary_ops = vec![
            (
                CalcToken::Power {
                    position: LexPosition::new(),
                },
                BinaryOp::Power,
                12,
            ),
            (
                CalcToken::Multiply {
                    position: LexPosition::new(),
                },
                BinaryOp::Multiply,
                10,
            ),
            (
                CalcToken::Divide {
                    position: LexPosition::new(),
                },
                BinaryOp::Divide,
                10,
            ),
            (
                CalcToken::Plus {
                    position: LexPosition::new(),
                },
                BinaryOp::Add,
                5,
            ),
            (
                CalcToken::Minus {
                    position: LexPosition::new(),
                },
                BinaryOp::Subtract,
                5,
            ),
        ];

        // 尝试递归解析括号内的完整表达式（包括二元运算）
        // 循环直到遇到右括号或无法继续解析
        loop {
            // 跳过空白
            while matches!(ctx.peek(), Some(CalcToken::Whitespace { .. })) {
                ctx.advance();
            }

            // 检查是否遇到右括号（应该停止解析）
            if matches!(ctx.peek(), Some(CalcToken::RightParen { .. })) {
                break;
            }

            // 检查是否有运算符（按优先级从高到低尝试）
            let expr_checkpoint = ctx.checkpoint();
            let mut found_op = false;

            for (op_token, op, op_priority) in &all_binary_ops {
                if let Some(token) = ctx.peek() {
                    if token_matches(op_token, token) {
                        // 找到了运算符，递归解析
                        ctx.restore(expr_checkpoint);
                        let mut binary_rule = BinaryRule::new(op_token.clone(), *op, *op_priority);
                        if let Some(binary_expr) = binary_rule.try_parse(ctx) {
                            expr = binary_expr;
                            found_op = true;
                            break;
                        }
                        // 如果失败，恢复检查点
                        ctx.restore(expr_checkpoint);
                        break;
                    }
                }
            }

            // 如果没有找到运算符，退出循环（可能是遇到了右括号或其他 token）
            if !found_op {
                break;
            }
        }

        // 跳过空白
        while matches!(ctx.peek(), Some(CalcToken::Whitespace { .. })) {
            ctx.advance();
        }

        // 检查右括号
        if !matches!(ctx.peek(), Some(CalcToken::RightParen { .. })) {
            return None;
        }
        ctx.advance(); // 消费右括号

        Some(Expr::Group {
            expr: Box::new(expr),
            position: start_pos,
        })
    }

    fn priority(&self) -> i32 {
        25 // 最高优先级，括号优先级最高
    }

    fn quick_check(&self, current_token: Option<&CalcToken>) -> Option<bool> {
        Some(matches!(current_token?, CalcToken::LeftParen { .. }))
    }
}

/// 解析一元运算符（负号）
struct UnaryRule;

impl<Ctx> ParsingRule<Ctx, CalcToken, Expr> for UnaryRule
where
    Ctx: ParseContext<CalcToken>,
{
    fn try_parse(&mut self, ctx: &mut Ctx) -> Option<Expr> {
        let position = if let Some(CalcToken::Minus { position: lex_pos }) = ctx.peek() {
            Position {
                line: lex_pos.line,
                column: lex_pos.column,
                offset: lex_pos.offset,
            }
        } else {
            return None;
        };

        ctx.advance(); // 消费负号

        // 跳过空白
        while matches!(ctx.peek(), Some(CalcToken::Whitespace { .. })) {
            ctx.advance();
        }

        // 解析操作数（递归）
        parse_expression(ctx).map(|operand| Expr::Unary {
            op: UnaryOp::Negate,
            operand: Box::new(operand),
            position,
        })
    }

    fn priority(&self) -> i32 {
        15
    }

    fn quick_check(&self, current_token: Option<&CalcToken>) -> Option<bool> {
        Some(matches!(current_token?, CalcToken::Minus { .. }))
    }
}

/// 解析二元运算符表达式
/// 支持运算符优先级：右操作数会递归解析更高优先级的运算符
struct BinaryRule {
    op_token: CalcToken,
    op: BinaryOp,
    priority: i32,
}

impl BinaryRule {
    fn new(op_token: CalcToken, op: BinaryOp, priority: i32) -> Self {
        Self {
            op_token,
            op,
            priority,
        }
    }
}

impl<Ctx> ParsingRule<Ctx, CalcToken, Expr> for BinaryRule
where
    Ctx: ParseContext<CalcToken>,
{
    fn try_parse(&mut self, ctx: &mut Ctx) -> Option<Expr> {
        // 创建检查点，用于失败时回溯
        let checkpoint = ctx.checkpoint();

        // 跳过空白
        while matches!(ctx.peek(), Some(CalcToken::Whitespace { .. })) {
            ctx.advance();
        }

        // 尝试解析左操作数
        // 左操作数可以是：数字、括号表达式、一元表达式，或者已经解析好的表达式（如 Group）
        // 由于我们无法直接识别已解析的表达式，我们使用 parse_expression 来解析基本表达式
        // 但这样无法处理 "Group * Number" 的情况
        //
        // 解决方案：让 BinaryRule 能够识别 Group 作为左操作数
        // 或者改进架构，让规则能够递归使用规则系统

        // 先尝试解析一个基本的表达式（数字、括号、一元运算符）
        let left = if let Some(expr) = parse_expression(ctx) {
            expr
        } else {
            ctx.restore(checkpoint);
            return None;
        };

        // 跳过空白
        while matches!(ctx.peek(), Some(CalcToken::Whitespace { .. })) {
            ctx.advance();
        }

        // 检查运算符是否匹配
        let position = match ctx.peek() {
            Some(token) if token_matches(&self.op_token, token) => {
                if let Some(lex_pos) = token.position() {
                    Position {
                        line: lex_pos.line,
                        column: lex_pos.column,
                        offset: lex_pos.offset,
                    }
                } else {
                    ctx.restore(checkpoint);
                    return None;
                }
            }
            _ => {
                ctx.restore(checkpoint);
                return None;
            }
        };

        ctx.advance(); // 消费运算符

        // 跳过空白
        while matches!(ctx.peek(), Some(CalcToken::Whitespace { .. })) {
            ctx.advance();
        }

        // 解析右操作数（需要递归解析，可能包含更高优先级的运算符）
        // 例如：在 "3 + 4 * 5" 中，+ 的左操作数是 3，右操作数应该是 "4 * 5"（Binary(*, 4, 5)）
        //
        // 策略：先解析基本表达式，然后检查后面是否有更高优先级的运算符
        // 如果有，递归解析更高优先级的表达式

        let right_checkpoint = ctx.checkpoint();

        // 先解析基本表达式（数字、括号、一元运算符）
        let mut right = if let Some(expr) = parse_expression(ctx) {
            expr
        } else {
            ctx.restore(checkpoint);
            return None;
        };

        // 跳过空白
        while matches!(ctx.peek(), Some(CalcToken::Whitespace { .. })) {
            ctx.advance();
        }

        // 检查后面是否有更高优先级的运算符需要处理
        // 根据当前运算符的优先级，只尝试解析优先级 >= 当前优先级的运算符
        let higher_priority_ops: Vec<(CalcToken, BinaryOp, i32)> = match self.priority {
            5 => {
                // 加减运算符（优先级5）：右操作数应该尝试解析 *、/、^
                vec![
                    (
                        CalcToken::Power {
                            position: LexPosition::new(),
                        },
                        BinaryOp::Power,
                        12,
                    ),
                    (
                        CalcToken::Multiply {
                            position: LexPosition::new(),
                        },
                        BinaryOp::Multiply,
                        10,
                    ),
                    (
                        CalcToken::Divide {
                            position: LexPosition::new(),
                        },
                        BinaryOp::Divide,
                        10,
                    ),
                ]
            }
            10 => {
                // 乘除运算符（优先级10）：右操作数应该尝试解析 ^
                vec![(
                    CalcToken::Power {
                        position: LexPosition::new(),
                    },
                    BinaryOp::Power,
                    12,
                )]
            }
            _ => vec![], // 其他优先级（如 ^，优先级12）不需要处理更高优先级
        };

        // 尝试解析更高优先级的运算符
        for (op_token, op, _op_priority) in higher_priority_ops {
            // 检查当前 token 是否匹配更高优先级的运算符
            if let Some(token) = ctx.peek() {
                if token_matches(&op_token, token) {
                    // 找到了更高优先级的运算符，需要递归解析
                    // 恢复右操作数的检查点，然后重新解析（包括更高优先级的运算符）
                    ctx.restore(right_checkpoint);

                    // 创建更高优先级的 BinaryRule 来递归解析
                    let mut higher_rule = BinaryRule::new(op_token, op, _op_priority);
                    if let Some(higher_expr) = higher_rule.try_parse(ctx) {
                        right = higher_expr;
                        break; // 成功解析，退出循环
                    }
                    // 如果失败，恢复并使用之前解析的基本表达式
                    ctx.restore(right_checkpoint);
                    right = parse_expression(ctx)?;
                    break;
                }
            }
        }

        Some(Expr::Binary {
            op: self.op,
            left: Box::new(left),
            right: Box::new(right),
            position,
        })
    }

    fn priority(&self) -> i32 {
        self.priority
    }

    fn quick_check(&self, current_token: Option<&CalcToken>) -> Option<bool> {
        // 二元运算符需要第一个 token 是数字或括号，这样才有左操作数
        current_token.map(|t| {
            matches!(
                t,
                CalcToken::Number { .. } | CalcToken::LeftParen { .. } | CalcToken::Minus { .. } // 可能是一元负号
            )
        })
    }
}

// 辅助函数：检查两个 token 是否匹配（忽略位置）
fn token_matches(t1: &CalcToken, t2: &CalcToken) -> bool {
    matches!(
        (t1, t2),
        (CalcToken::Plus { .. }, CalcToken::Plus { .. })
            | (CalcToken::Minus { .. }, CalcToken::Minus { .. })
            | (CalcToken::Multiply { .. }, CalcToken::Multiply { .. })
            | (CalcToken::Divide { .. }, CalcToken::Divide { .. })
            | (CalcToken::Power { .. }, CalcToken::Power { .. })
    )
}

// 辅助函数：解析表达式（简化版，用于 BinaryRule 和 GroupRule 内部）
// 这个函数只能解析基本的表达式（数字、括号），不能递归解析完整的二元运算
// 因为二元运算需要根据优先级来决定如何解析
fn parse_expression<Ctx>(ctx: &mut Ctx) -> Option<Expr>
where
    Ctx: ParseContext<CalcToken>,
{
    // 跳过空白
    while matches!(ctx.peek(), Some(CalcToken::Whitespace { .. })) {
        ctx.advance();
    }

    // 尝试解析数字
    let token = ctx.peek()?.clone();
    if let CalcToken::Number {
        value,
        position: lex_pos,
    } = token
    {
        let position = Position {
            line: lex_pos.line,
            column: lex_pos.column,
            offset: lex_pos.offset,
        };
        ctx.advance();
        return Some(Expr::Number { value, position });
    }

    // 尝试解析括号
    if matches!(ctx.peek(), Some(CalcToken::LeftParen { .. })) {
        let checkpoint = ctx.checkpoint();
        let mut group_rule = GroupRule;
        if let Some(expr) = group_rule.try_parse(ctx) {
            return Some(expr);
        }
        ctx.restore(checkpoint);
    }

    // 尝试解析一元运算符
    if matches!(ctx.peek(), Some(CalcToken::Minus { .. })) {
        let checkpoint = ctx.checkpoint();
        let mut unary_rule = UnaryRule;
        if let Some(expr) = unary_rule.try_parse(ctx) {
            return Some(expr);
        }
        ctx.restore(checkpoint);
    }

    None
}

// ============================================================================
// 主程序
// ============================================================================

fn main() {
    println!("=== 计算器解析器示例 ===\n");

    let expressions = vec![
        "3 + 4",
        "2 * 3.14",
        "(1 + 2) * 3",
        "2 ^ 8",
        "10 / 2.5",
        "-5",
        "3 + 4 * 5",
    ];

    for expr in expressions {
        println!("表达式: {}", expr);
        println!("{}", "=".repeat(50));

        // 步骤 1: 词法分析
        println!("\n[步骤 1] 词法分析:");
        let lexer_rules: CalcLexerRules = vec![
            Box::new(NumberRule),
            Box::new(OperatorRule),
            Box::new(WhitespaceRule),
            Box::new(EofRule),
        ];

        let mut lexer = Lexer::from_str(expr, lexer_rules);
        let tokens: Vec<CalcToken> = lexer
            .tokenize()
            .into_iter()
            .filter(|t| !t.is_whitespace() && !t.is_eof())
            .collect();

        for (i, token) in tokens.iter().enumerate() {
            println!("  Token {}: {:?}", i, token);
        }

        // 步骤 2: 语法分析
        println!("\n[步骤 2] 语法分析:");

        // 创建解析规则
        // 注意：规则按优先级从高到低排序
        // 优先级高的规则先尝试，这样可以先匹配复杂表达式（如二元运算、括号）
        // 数字规则优先级最低，作为后备规则
        let parser_rules = build_parser_rules();

        let context = DefaultContext::from_token_iter(tokens);
        let mut parser = Parser::new(context, parser_rules);

        // 解析 AST
        if let Some(ast) = parser.parse_one() {
            println!("  AST: {:#?}", ast);

            // 显示 AST 的树形结构
            println!("\n  AST 树形结构:");
            print_ast_tree(&ast, 0);
        } else {
            println!("  解析失败：无法生成 AST");
        }

        println!("\n{}", "=".repeat(50));
        println!();
    }

    #[cfg(feature = "streaming")]
    {
        println!("=== 流式解析管道示例 ===");
        let controller = CalcPipeline::new();
        let inputs = ["3 + 4 * 2", "(1 + 2) * 3", "10 / 2 + 5"];
        for expr in inputs {
            println!("Pipeline 输入: {}", expr);
            let asts = controller.run(expr);
            for (idx, ast) in asts.iter().enumerate() {
                println!("  AST {}: {:#?}", idx + 1, ast);
            }
            println!("{}", "-".repeat(50));
        }
    }
}

#[cfg(feature = "streaming")]
struct CalcPipeline;

#[cfg(feature = "streaming")]
impl CalcPipeline {
    fn new() -> Self {
        Self
    }

    fn run(&self, input: &str) -> Vec<Expr> {
        let lexer_rules = build_lexer_rules();
        let parser_rules = build_parser_rules_for_ctx::<StreamingParseContext<CalcToken>>();

        let lexer = FilteringProducer::new(Lexer::from_str(input.to_owned(), lexer_rules));
        let parser = Parser::new(StreamingParseContext::new(), parser_rules);

        Pipeline::new(lexer, parser).run()
    }
}

#[cfg(feature = "streaming")]
fn build_lexer_rules() -> CalcLexerRules {
    vec![
        Box::new(NumberRule),
        Box::new(OperatorRule),
        Box::new(WhitespaceRule),
        Box::new(EofRule),
    ]
}

#[cfg(feature = "streaming")]
fn filter_streaming_token(token: CalcToken) -> Option<CalcToken> {
    match token {
        CalcToken::Whitespace { .. } | CalcToken::Eof { .. } => None,
        other => Some(other),
    }
}

#[cfg(feature = "streaming")]
struct FilteringProducer<L> {
    inner: L,
}

#[cfg(feature = "streaming")]
impl<L> FilteringProducer<L> {
    fn new(inner: L) -> Self {
        Self { inner }
    }
}

#[cfg(feature = "streaming")]
impl<L> TokenProducer<CalcToken> for FilteringProducer<L>
where
    L: TokenProducer<CalcToken>,
{
    fn poll_token(&mut self) -> Option<CalcToken> {
        while let Some(token) = self.inner.poll_token() {
            if let Some(filtered) = filter_streaming_token(token) {
                return Some(filtered);
            }
        }
        None
    }
}

#[cfg(feature = "streaming")]
impl<L, Ast> Outbound<CalcToken, Ast> for FilteringProducer<L>
where
    L: Outbound<CalcToken, Ast>,
{
    fn next_signal(&mut self) -> Option<StreamingSignal<CalcToken, Ast>> {
        while let Some(signal) = self.inner.next_signal() {
            match signal {
                StreamingSignal::SupplyToken(token) => {
                    if let Some(filtered) = filter_streaming_token(token) {
                        return Some(StreamingSignal::SupplyToken(filtered));
                    }
                    continue;
                }
                other => return Some(other),
            }
        }
        None
    }
}

#[cfg(feature = "streaming")]
impl<L, Ast> Inbound<CalcToken, Ast> for FilteringProducer<L>
where
    L: Inbound<CalcToken, Ast>,
{
    fn handle_signal(&mut self, signal: StreamingSignal<CalcToken, Ast>) {
        self.inner.handle_signal(signal);
    }
}

// 辅助函数：打印 AST 树形结构
fn print_ast_tree(expr: &Expr, indent: usize) {
    let prefix = "  ".repeat(indent);
    match expr {
        Expr::Number { value, .. } => {
            println!("{}Number({})", prefix, value);
        }
        Expr::Binary {
            op, left, right, ..
        } => {
            let op_str = match op {
                BinaryOp::Add => "+",
                BinaryOp::Subtract => "-",
                BinaryOp::Multiply => "*",
                BinaryOp::Divide => "/",
                BinaryOp::Power => "^",
            };
            println!("{}Binary({})", prefix, op_str);
            print_ast_tree(left, indent + 1);
            print_ast_tree(right, indent + 1);
        }
        Expr::Unary { op, operand, .. } => {
            let op_str = match op {
                UnaryOp::Negate => "-",
            };
            println!("{}Unary({})", prefix, op_str);
            print_ast_tree(operand, indent + 1);
        }
        Expr::Group { expr, .. } => {
            println!("{}Group", prefix);
            print_ast_tree(expr, indent + 1);
        }
    }
}
