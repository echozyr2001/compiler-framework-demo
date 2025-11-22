use common_framework::Position;
use lexer_framework::LexToken;

#[derive(Debug, Clone, PartialEq)]
pub enum MarkdownToken {
    // 结构化tokens
    Hash { count: usize, position: Position }, // #, ##, ...
    Newline { position: Position },
    Text { content: String, position: Position },

    // 列表
    Dash { position: Position },     // -
    Asterisk { position: Position }, // *

    // 代码
    Backtick { count: usize, position: Position }, // `, ```, etc.

    // 强调
    Star { count: usize, position: Position }, // *, **
    Underscore { count: usize, position: Position }, // _, __

    // 链接
    LeftBracket { position: Position },  // [
    RightBracket { position: Position }, // ]
    LeftParen { position: Position },    // (
    RightParen { position: Position },   // )

    Eof { position: Position },
}

impl LexToken for MarkdownToken {
    fn position(&self) -> Option<Position> {
        Some(match self {
            MarkdownToken::Hash { position, .. }
            | MarkdownToken::Newline { position }
            | MarkdownToken::Text { position, .. }
            | MarkdownToken::Dash { position }
            | MarkdownToken::Asterisk { position }
            | MarkdownToken::Backtick { position, .. }
            | MarkdownToken::Star { position, .. }
            | MarkdownToken::Underscore { position, .. }
            | MarkdownToken::LeftBracket { position }
            | MarkdownToken::RightBracket { position }
            | MarkdownToken::LeftParen { position }
            | MarkdownToken::RightParen { position }
            | MarkdownToken::Eof { position } => *position,
        })
    }

    fn is_eof(&self) -> bool {
        matches!(self, MarkdownToken::Eof { .. })
    }

    fn is_newline(&self) -> bool {
        matches!(self, MarkdownToken::Newline { .. })
    }

    fn is_whitespace(&self) -> bool {
        false
    }

    fn is_indent(&self) -> bool {
        false
    }
}
