//! 计算器词法分析器示例
//! 展示如何使用 lexer-framework 定义计算器风格的 Token 和规则

use lexer_framework::{DefaultContext, LexContext, LexToken, Lexer, LexingRule, Position};

/// 计算器 Token 类型（更简单的设计）
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

/// 匹配数字（浮点数）
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

        // 处理整数部分
        if first.is_ascii_digit() {
            value_str.push(first);
            has_digit = true;
            ctx.advance();
            let int_part = ctx.consume_while(|c| c.is_ascii_digit());
            value_str.push_str(int_part);
        }

        // 处理小数部分
        if ctx.peek() == Some('.') {
            value_str.push('.');
            ctx.advance();
            let decimal = ctx.consume_while(|c| c.is_ascii_digit());
            if !decimal.is_empty() {
                has_digit = true;
            }
            value_str.push_str(decimal);
        }

        // 必须有至少一个数字
        if !has_digit {
            return None;
        }

        // 尝试解析为浮点数
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

/// 匹配操作符
///
/// 这里展示了两种实现方式：
/// 1. 使用 match 语句：适合固定、少量的操作符（性能更好，编译期优化）
/// 2. 使用 Vec 映射：适合运行时可配置的操作符（更灵活，但需要查找）
///
/// 当前实现使用 match，因为计算器的操作符是固定的。
/// 如果需要动态配置操作符，可以参考 json_lexer.rs 中的 PunctuationRule 实现。
pub struct OperatorRule;

impl<Ctx> LexingRule<Ctx, CalcToken> for OperatorRule
where
    Ctx: LexContext,
{
    /// 快速检查：只有操作符字符才可能匹配
    ///
    /// 这是一个性能优化：当输入是字母、数字、空格等时，
    /// 直接跳过这个规则，避免不必要的 checkpoint 创建和 try_match 调用。
    fn quick_check(&self, first_char: Option<char>) -> Option<bool> {
        match first_char? {
            '+' | '-' | '*' | '/' | '^' | '(' | ')' => Some(true), // 可能是操作符
            _ => Some(false), // 字母、数字、空格等肯定不是操作符
        }
    }

    fn try_match(&mut self, ctx: &mut Ctx) -> Option<CalcToken> {
        use CalcToken::*;
        let ch = ctx.peek()?;
        let position = ctx.position();

        // 方式1：使用 match（当前实现）
        // 优点：编译期优化，性能好，代码简洁
        // 缺点：操作符列表固定，编译期确定
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

        // 方式2：使用 Vec 映射（可选的实现方式）
        // 如果要使用这种方式，可以将 OperatorRule 改为：
        // pub struct OperatorRule {
        //     mappings: Vec<(char, fn(Position) -> CalcToken)>,
        // }
        // 然后在 try_match 中遍历 mappings 查找匹配的字符
        // 优点：可以在运行时配置操作符，更灵活
        // 缺点：需要动态查找，性能略差
    }

    fn priority(&self) -> i32 {
        10
    }
}

/// 匹配空白字符
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

/// 匹配 EOF
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
