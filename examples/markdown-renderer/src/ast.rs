use crate::state::ContentState;
use common_framework::Position;
use parser_framework::{AstNode, StatefulNode};

/// Markdown AST节点
#[derive(Debug, Clone)]
pub enum MarkdownNode {
    Heading {
        level: usize,
        content: Vec<Inline>,
        position: Position,
        state: ContentState,
    },
    Paragraph {
        content: Vec<Inline>,
        position: Position,
        state: ContentState,
    },
    List {
        items: Vec<Vec<Inline>>,
        position: Position,
        state: ContentState,
    },
    CodeBlock {
        language: Option<String>,
        code: String,
        position: Position,
        state: ContentState,
    },
    // 当内容不完整时使用
    RawText {
        text: String,
        position: Position,
    },
}

/// 行内元素
#[derive(Debug, Clone)]
pub enum Inline {
    Text(String),
    Bold(String),
    Italic(String),
    Code(String),
    Link { text: String, url: String },
}

impl AstNode for MarkdownNode {
    fn position(&self) -> Option<Position> {
        Some(match self {
            MarkdownNode::Heading { position, .. }
            | MarkdownNode::Paragraph { position, .. }
            | MarkdownNode::List { position, .. }
            | MarkdownNode::CodeBlock { position, .. }
            | MarkdownNode::RawText { position, .. } => *position,
        })
    }
}

impl StatefulNode for MarkdownNode {
    type State = ContentState;

    fn state(&self) -> &Self::State {
        match self {
            MarkdownNode::Heading { state, .. }
            | MarkdownNode::Paragraph { state, .. }
            | MarkdownNode::List { state, .. }
            | MarkdownNode::CodeBlock { state, .. } => state,
            MarkdownNode::RawText { .. } => {
                // RawText 总是 Incomplete
                &ContentState::Incomplete
            }
        }
    }

    fn set_state(&mut self, state: Self::State) {
        match self {
            MarkdownNode::Heading { state: s, .. }
            | MarkdownNode::Paragraph { state: s, .. }
            | MarkdownNode::List { state: s, .. }
            | MarkdownNode::CodeBlock { state: s, .. } => {
                *s = state;
            }
            MarkdownNode::RawText { .. } => {
                // RawText 不支持状态设置
            }
        }
    }

    fn transition(&mut self, _trigger: &dyn std::any::Any) -> bool {
        // 默认不实现状态转换，应用层可以处理
        false
    }
}
