use crate::token::MarkdownToken;
use lexer_framework::{DefaultContext, LexContext, LexingRule};

/// 匹配 # 符号（标题）
pub struct HashRule;

impl LexingRule<DefaultContext, MarkdownToken> for HashRule {
    fn quick_check(&self, first_char: Option<char>) -> Option<bool> {
        Some(first_char == Some('#'))
    }

    fn try_match(&mut self, ctx: &mut DefaultContext) -> Option<MarkdownToken> {
        let position = ctx.position();
        let mut count = 0;

        // 计数连续的 #
        while ctx.peek() == Some('#') && count < 6 {
            count += 1;
            ctx.advance();
        }

        if count > 0 {
            Some(MarkdownToken::Hash { count, position })
        } else {
            None
        }
    }

    fn priority(&self) -> i32 {
        50
    }
}

/// 匹配换行符
pub struct NewlineRule;

impl LexingRule<DefaultContext, MarkdownToken> for NewlineRule {
    fn quick_check(&self, first_char: Option<char>) -> Option<bool> {
        Some(first_char == Some('\n'))
    }

    fn try_match(&mut self, ctx: &mut DefaultContext) -> Option<MarkdownToken> {
        if ctx.peek() == Some('\n') {
            let position = ctx.position();
            ctx.advance();
            Some(MarkdownToken::Newline { position })
        } else {
            None
        }
    }

    fn priority(&self) -> i32 {
        40
    }
}

/// 匹配反引号（代码）
pub struct BacktickRule;

impl LexingRule<DefaultContext, MarkdownToken> for BacktickRule {
    fn quick_check(&self, first_char: Option<char>) -> Option<bool> {
        Some(first_char == Some('`'))
    }

    fn try_match(&mut self, ctx: &mut DefaultContext) -> Option<MarkdownToken> {
        if ctx.peek() == Some('`') {
            let position = ctx.position();
            let mut count = 0;

            // 计数连续的反引号（最多3个用于代码块）
            while ctx.peek() == Some('`') && count < 3 {
                count += 1;
                ctx.advance();
            }

            Some(MarkdownToken::Backtick { count, position })
        } else {
            None
        }
    }

    fn priority(&self) -> i32 {
        45
    }
}

/// 匹配星号（列表或强调）
pub struct StarRule;

impl LexingRule<DefaultContext, MarkdownToken> for StarRule {
    fn quick_check(&self, first_char: Option<char>) -> Option<bool> {
        Some(first_char == Some('*'))
    }

    fn try_match(&mut self, ctx: &mut DefaultContext) -> Option<MarkdownToken> {
        if ctx.peek() == Some('*') {
            let position = ctx.position();
            let mut count = 0;

            // 计数连续的 *（用于强调）
            while ctx.peek() == Some('*') && count < 2 {
                count += 1;
                ctx.advance();
            }

            Some(MarkdownToken::Star { count, position })
        } else {
            None
        }
    }

    fn priority(&self) -> i32 {
        30
    }
}

/// 匹配下划线（强调）
pub struct UnderscoreRule;

impl LexingRule<DefaultContext, MarkdownToken> for UnderscoreRule {
    fn quick_check(&self, first_char: Option<char>) -> Option<bool> {
        Some(first_char == Some('_'))
    }

    fn try_match(&mut self, ctx: &mut DefaultContext) -> Option<MarkdownToken> {
        if ctx.peek() == Some('_') {
            let position = ctx.position();
            let mut count = 0;

            // 计数连续的下划线（用于强调）
            while ctx.peek() == Some('_') && count < 2 {
                count += 1;
                ctx.advance();
            }

            Some(MarkdownToken::Underscore { count, position })
        } else {
            None
        }
    }

    fn priority(&self) -> i32 {
        30
    }
}

/// 匹配破折号（列表）
pub struct DashRule;

impl LexingRule<DefaultContext, MarkdownToken> for DashRule {
    fn quick_check(&self, first_char: Option<char>) -> Option<bool> {
        Some(first_char == Some('-'))
    }

    fn try_match(&mut self, ctx: &mut DefaultContext) -> Option<MarkdownToken> {
        if ctx.peek() == Some('-') {
            let position = ctx.position();
            ctx.advance();
            Some(MarkdownToken::Dash { position })
        } else {
            None
        }
    }

    fn priority(&self) -> i32 {
        35
    }
}

/// 匹配方括号和圆括号（链接）
pub struct BracketRules;

impl LexingRule<DefaultContext, MarkdownToken> for BracketRules {
    fn quick_check(&self, first_char: Option<char>) -> Option<bool> {
        match first_char? {
            '[' | ']' | '(' | ')' => Some(true),
            _ => Some(false),
        }
    }

    fn try_match(&mut self, ctx: &mut DefaultContext) -> Option<MarkdownToken> {
        let position = ctx.position();
        let token = match ctx.peek()? {
            '[' => {
                ctx.advance();
                MarkdownToken::LeftBracket { position }
            }
            ']' => {
                ctx.advance();
                MarkdownToken::RightBracket { position }
            }
            '(' => {
                ctx.advance();
                MarkdownToken::LeftParen { position }
            }
            ')' => {
                ctx.advance();
                MarkdownToken::RightParen { position }
            }
            _ => return None,
        };
        Some(token)
    }

    fn priority(&self) -> i32 {
        35
    }
}

/// 匹配普通文本
pub struct TextRule;

impl LexingRule<DefaultContext, MarkdownToken> for TextRule {
    fn try_match(&mut self, ctx: &mut DefaultContext) -> Option<MarkdownToken> {
        // 如果当前字符是特殊字符，不匹配
        match ctx.peek()? {
            '#' | '\n' | '`' | '*' | '_' | '-' | '[' | ']' | '(' | ')' => return None,
            _ => {}
        }

        let position = ctx.position();
        let text = ctx.consume_while(|ch| {
            !matches!(
                ch,
                '#' | '\n' | '`' | '*' | '_' | '-' | '[' | ']' | '(' | ')'
            )
        });

        if !text.as_ref().is_empty() {
            Some(MarkdownToken::Text {
                content: text.as_ref().to_string(),
                position,
            })
        } else {
            None
        }
    }

    fn priority(&self) -> i32 {
        1 // 低优先级，在其他规则都不匹配时使用
    }
}

/// 匹配 EOF
pub struct EofRule;

impl LexingRule<DefaultContext, MarkdownToken> for EofRule {
    fn try_match(&mut self, ctx: &mut DefaultContext) -> Option<MarkdownToken> {
        if ctx.is_eof() {
            Some(MarkdownToken::Eof {
                position: ctx.position(),
            })
        } else {
            None
        }
    }

    fn priority(&self) -> i32 {
        -10 // 最低优先级
    }
}

/// 构建所有词法规则
pub fn build_lexer_rules() -> Vec<Box<dyn LexingRule<DefaultContext, MarkdownToken>>> {
    vec![
        Box::new(HashRule),
        Box::new(BacktickRule),
        Box::new(NewlineRule),
        Box::new(StarRule),
        Box::new(UnderscoreRule),
        Box::new(DashRule),
        Box::new(BracketRules),
        Box::new(TextRule),
        Box::new(EofRule),
    ]
}
