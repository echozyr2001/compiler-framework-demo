use crate::ast::{Inline, MarkdownNode};
use crate::lexer_rules::build_lexer_rules;
use crate::parser_rules::build_parser_rules;
use crate::state::ContentState;
use crate::token::MarkdownToken;
use lexer_framework::Lexer;
use parser_framework::{DefaultContext as ParseDefaultContext, Parser};

/// Markdown渲染引擎 - 编排词法分析和语法分析
pub struct MarkdownRenderer {
    /// 可选：缓存之前的AST结果
    cached_nodes: Option<Vec<MarkdownNode>>,
}

impl MarkdownRenderer {
    pub fn new() -> Self {
        Self { cached_nodes: None }
    }

    /// 解析输入文本，返回AST节点
    pub fn parse(&mut self, input: &str) -> Vec<MarkdownNode> {
        // 1. 词法分析
        let mut lexer = Lexer::from_str(input, build_lexer_rules());
        let tokens: Vec<MarkdownToken> = lexer.tokenize();

        // 2. 语法分析
        let mut parser =
            Parser::<ParseDefaultContext<MarkdownToken>, MarkdownToken, MarkdownNode>::from_tokens(
                tokens,
                build_parser_rules(),
            );
        let nodes = parser.parse();

        // 3. 缓存结果（可选）
        self.cached_nodes = Some(nodes.clone());

        nodes
    }

    /// 获取渲染结果 - 上层根据节点状态决定如何渲染
    pub fn get_render_result(&self, nodes: &[MarkdownNode]) -> RenderResult {
        let mut items = Vec::new();

        for node in nodes {
            match node {
                MarkdownNode::Heading {
                    level,
                    content,
                    state,
                    ..
                } => {
                    match state {
                        ContentState::Complete => {
                            // 应该渲染为标题
                            items.push(RenderItem::Heading {
                                level: *level,
                                text: inline_to_text(content),
                            });
                        }
                        ContentState::Incomplete => {
                            // 显示原始文本
                            items.push(RenderItem::RawText(format!(
                                "{} {}",
                                "#".repeat(*level),
                                inline_to_text(content)
                            )));
                        }
                    }
                }
                MarkdownNode::Paragraph { content, state, .. } => match state {
                    ContentState::Complete => {
                        items.push(RenderItem::Paragraph(inline_to_text(content)));
                    }
                    ContentState::Incomplete => {
                        items.push(RenderItem::RawText(inline_to_text(content)));
                    }
                },
                MarkdownNode::List {
                    items: list_items,
                    state,
                    ..
                } => {
                    match state {
                        ContentState::Complete => {
                            let texts: Vec<String> =
                                list_items.iter().map(|item| inline_to_text(item)).collect();
                            items.push(RenderItem::List(texts));
                        }
                        ContentState::Incomplete => {
                            // 显示原始文本
                            for item in list_items {
                                items.push(RenderItem::RawText(format!(
                                    "- {}",
                                    inline_to_text(item)
                                )));
                            }
                        }
                    }
                }
                MarkdownNode::CodeBlock {
                    language,
                    code,
                    state,
                    ..
                } => {
                    match state {
                        ContentState::Complete => {
                            items.push(RenderItem::CodeBlock {
                                language: language.clone(),
                                code: code.clone(),
                            });
                        }
                        ContentState::Incomplete => {
                            // 显示原始文本
                            let prefix = if let Some(lang) = language {
                                format!("```{}\n", lang)
                            } else {
                                "```\n".to_string()
                            };
                            items.push(RenderItem::RawText(format!("{}{}", prefix, code)));
                        }
                    }
                }
                MarkdownNode::RawText { text, .. } => {
                    items.push(RenderItem::RawText(text.clone()));
                }
            }
        }

        RenderResult { items }
    }
}

impl Default for MarkdownRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// 渲染结果 - 告诉UI层应该如何显示
#[derive(Debug, Clone)]
pub struct RenderResult {
    pub items: Vec<RenderItem>,
}

#[derive(Debug, Clone)]
pub enum RenderItem {
    Heading {
        level: usize,
        text: String,
    },
    Paragraph(String),
    List(Vec<String>),
    CodeBlock {
        language: Option<String>,
        code: String,
    },
    RawText(String), // 未确定的内容，显示原始文本
}

fn inline_to_text(inlines: &[Inline]) -> String {
    inlines
        .iter()
        .map(|i| match i {
            Inline::Text(s) => s.clone(),
            Inline::Bold(s) => format!("**{}**", s),
            Inline::Italic(s) => format!("*{}*", s),
            Inline::Code(s) => format!("`{}`", s),
            Inline::Link { text, url } => format!("[{}]({})", text, url),
        })
        .collect()
}
