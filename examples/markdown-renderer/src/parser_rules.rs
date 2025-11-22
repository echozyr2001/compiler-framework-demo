use crate::ast::{Inline, MarkdownNode};
use crate::state::ContentState;
use crate::token::MarkdownToken;
use parser_framework::{DefaultContext, ParseContext, ParsingRule};

/// 标题解析规则
pub struct HeadingRule;

impl ParsingRule<DefaultContext<MarkdownToken>, MarkdownToken, MarkdownNode> for HeadingRule {
    fn quick_check(&self, current_token: Option<&MarkdownToken>) -> Option<bool> {
        Some(matches!(current_token, Some(MarkdownToken::Hash { .. })))
    }

    fn try_parse(&mut self, ctx: &mut DefaultContext<MarkdownToken>) -> Option<MarkdownNode> {
        let checkpoint = ctx.checkpoint();
        let position = ctx.position();

        // 检查是否以#开头
        let level = match ctx.peek()? {
            MarkdownToken::Hash { count, .. } => *count,
            _ => {
                ctx.restore(checkpoint);
                return None;
            }
        };
        ctx.advance();

        // 可选：跳过空格
        if matches!(ctx.peek(), Some(MarkdownToken::Text { content, .. }) if content.trim().is_empty())
        {
            ctx.advance();
        }

        // 收集内容直到换行或EOF
        let mut content = Vec::new();
        let mut has_newline = false;

        while let Some(token) = ctx.peek() {
            match token {
                MarkdownToken::Newline { .. } => {
                    has_newline = true;
                    ctx.advance();
                    break;
                }
                MarkdownToken::Text { content: text, .. } => {
                    content.push(Inline::Text(text.clone()));
                    ctx.advance();
                }
                MarkdownToken::Eof { .. } => break,
                _ => {
                    // 其他token也作为文本处理（简化版）
                    ctx.advance();
                }
            }
        }

        // 判断状态：有换行就是Complete，否则Incomplete
        let state = if has_newline {
            ContentState::Complete
        } else {
            ContentState::Incomplete
        };

        Some(MarkdownNode::Heading {
            level,
            content,
            position,
            state,
        })
    }

    fn priority(&self) -> i32 {
        100
    }
}

/// 代码块解析规则
pub struct CodeBlockRule;

impl ParsingRule<DefaultContext<MarkdownToken>, MarkdownToken, MarkdownNode> for CodeBlockRule {
    fn quick_check(&self, current_token: Option<&MarkdownToken>) -> Option<bool> {
        Some(matches!(
            current_token,
            Some(MarkdownToken::Backtick { count: 3, .. })
        ))
    }

    fn try_parse(&mut self, ctx: &mut DefaultContext<MarkdownToken>) -> Option<MarkdownNode> {
        let checkpoint = ctx.checkpoint();
        let position = ctx.position();

        // 检查是否以```开头
        match ctx.peek()? {
            MarkdownToken::Backtick { count: 3, .. } => {
                ctx.advance();
            }
            _ => {
                ctx.restore(checkpoint);
                return None;
            }
        }

        // 读取语言标识（可选）
        let language = if let Some(MarkdownToken::Text { content, .. }) = ctx.peek() {
            let lang = content.clone();
            ctx.advance();
            if lang.trim().is_empty() {
                None
            } else {
                Some(lang.trim().to_string())
            }
        } else {
            None
        };

        // 必须有一个换行
        if !matches!(ctx.peek(), Some(MarkdownToken::Newline { .. })) {
            ctx.restore(checkpoint);
            return None;
        }
        ctx.advance();

        // 收集代码内容直到找到结束的```
        let mut code = String::new();
        let mut found_end = false;

        while let Some(token) = ctx.peek() {
            match token {
                MarkdownToken::Backtick { count: 3, .. } => {
                    ctx.advance();
                    found_end = true;
                    break;
                }
                MarkdownToken::Text { content, .. } => {
                    code.push_str(content);
                    ctx.advance();
                }
                MarkdownToken::Newline { .. } => {
                    code.push('\n');
                    ctx.advance();
                }
                MarkdownToken::Eof { .. } => break,
                _ => {
                    // 其他token也作为代码内容
                    ctx.advance();
                }
            }
        }

        // 判断状态：找到结束标记就是Complete，否则Incomplete
        let state = if found_end {
            ContentState::Complete
        } else {
            ContentState::Incomplete
        };

        Some(MarkdownNode::CodeBlock {
            language,
            code,
            position,
            state,
        })
    }

    fn priority(&self) -> i32 {
        90
    }
}

/// 列表解析规则
pub struct ListRule;

impl ParsingRule<DefaultContext<MarkdownToken>, MarkdownToken, MarkdownNode> for ListRule {
    fn quick_check(&self, current_token: Option<&MarkdownToken>) -> Option<bool> {
        Some(matches!(
            current_token,
            Some(MarkdownToken::Dash { .. } | MarkdownToken::Star { count: 1, .. })
        ))
    }

    fn try_parse(&mut self, ctx: &mut DefaultContext<MarkdownToken>) -> Option<MarkdownNode> {
        let checkpoint = ctx.checkpoint();
        let position = ctx.position();

        // 检查是否以-或*开头（列表标记）
        let is_list_marker = matches!(
            ctx.peek(),
            Some(MarkdownToken::Dash { .. } | MarkdownToken::Star { count: 1, .. })
        );

        if !is_list_marker {
            ctx.restore(checkpoint);
            return None;
        }
        ctx.advance();

        // 跳过空格
        if matches!(ctx.peek(), Some(MarkdownToken::Text { content, .. }) if content.trim().is_empty())
        {
            ctx.advance();
        }

        // 收集列表项
        let mut items = Vec::new();
        let mut current_item = Vec::new();
        let mut has_newline = false;

        while let Some(token) = ctx.peek() {
            match token {
                MarkdownToken::Newline { .. } => {
                    has_newline = true;
                    ctx.advance();
                    if !current_item.is_empty() {
                        items.push(std::mem::take(&mut current_item));
                    }
                    // 检查下一个是否是列表项
                    if !matches!(
                        ctx.peek(),
                        Some(MarkdownToken::Dash { .. } | MarkdownToken::Star { count: 1, .. })
                    ) {
                        break;
                    }
                    // 是列表项，继续
                    ctx.advance();
                    // 跳过空格
                    if matches!(ctx.peek(), Some(MarkdownToken::Text { content, .. }) if content.trim().is_empty())
                    {
                        ctx.advance();
                    }
                }
                MarkdownToken::Text { content, .. } => {
                    current_item.push(Inline::Text(content.clone()));
                    ctx.advance();
                }
                MarkdownToken::Eof { .. } => {
                    if !current_item.is_empty() {
                        items.push(std::mem::take(&mut current_item));
                    }
                    break;
                }
                _ => {
                    ctx.advance();
                }
            }
        }

        // 如果循环结束时还有未完成的项，添加它
        if !current_item.is_empty() {
            items.push(current_item);
        }

        if items.is_empty() {
            ctx.restore(checkpoint);
            return None;
        }

        // 判断状态：有换行就是Complete，否则Incomplete
        let state = if has_newline {
            ContentState::Complete
        } else {
            ContentState::Incomplete
        };

        Some(MarkdownNode::List {
            items,
            position,
            state,
        })
    }

    fn priority(&self) -> i32 {
        80
    }
}

/// 段落解析规则
pub struct ParagraphRule;

impl ParsingRule<DefaultContext<MarkdownToken>, MarkdownToken, MarkdownNode> for ParagraphRule {
    fn try_parse(&mut self, ctx: &mut DefaultContext<MarkdownToken>) -> Option<MarkdownNode> {
        let checkpoint = ctx.checkpoint();
        let position = ctx.position();

        // 跳过空白
        while let Some(token) = ctx.peek() {
            let is_whitespace = match token {
                MarkdownToken::Text { content, .. } => content.trim().is_empty(),
                MarkdownToken::Newline { .. } => {
                    // 空行表示段落结束
                    break;
                }
                _ => false,
            };
            if !is_whitespace {
                break;
            }
            ctx.advance();
        }

        // 如果已经是EOF或特殊token，不匹配
        if matches!(
            ctx.peek(),
            None | Some(MarkdownToken::Eof { .. })
                | Some(MarkdownToken::Hash { .. })
                | Some(MarkdownToken::Backtick { count: 3, .. })
        ) {
            ctx.restore(checkpoint);
            return None;
        }

        // 收集段落内容直到换行或EOF
        let mut content = Vec::new();
        let mut has_newline = false;

        while let Some(token) = ctx.peek() {
            match token {
                MarkdownToken::Newline { .. } => {
                    ctx.advance();
                    // 如果下一 token 是空行 / EOF 或者是块级语法起始（标题、代码块、列表），终止段落
                    match ctx.peek() {
                        Some(MarkdownToken::Newline { .. }) | None => {
                            has_newline = true;
                            break;
                        }
                        Some(MarkdownToken::Hash { .. })
                        | Some(MarkdownToken::Backtick { count: 3, .. })
                        | Some(MarkdownToken::Dash { .. })
                        | Some(MarkdownToken::Star { count: 1, .. }) => {
                            has_newline = true;
                            break;
                        }
                        _ => {
                            // 单个换行，继续作为段落内容
                            content.push(Inline::Text("\n".to_string()));
                        }
                    }
                }
                MarkdownToken::Text { content: text, .. } => {
                    content.push(Inline::Text(text.clone()));
                    ctx.advance();
                }
                MarkdownToken::Eof { .. } => break,
                _ => {
                    // 其他token忽略（简化版）
                    ctx.advance();
                }
            }
        }

        if content.is_empty() {
            ctx.restore(checkpoint);
            return None;
        }

        // 段落有内容就认为是Complete（简化版）
        let state = if has_newline {
            ContentState::Complete
        } else {
            ContentState::Incomplete
        };

        Some(MarkdownNode::Paragraph {
            content,
            position,
            state,
        })
    }

    fn priority(&self) -> i32 {
        10
    }
}

/// 原始文本规则 - 兜底规则，当其他规则都不匹配时
pub struct RawTextRule;

impl ParsingRule<DefaultContext<MarkdownToken>, MarkdownToken, MarkdownNode> for RawTextRule {
    fn try_parse(&mut self, ctx: &mut DefaultContext<MarkdownToken>) -> Option<MarkdownNode> {
        let position = ctx.position();
        let mut text = String::new();

        // 收集到换行或EOF
        while let Some(token) = ctx.peek() {
            match token {
                MarkdownToken::Text { content, .. } => {
                    text.push_str(content);
                    ctx.advance();
                }
                MarkdownToken::Newline { .. } => {
                    text.push('\n');
                    ctx.advance();
                }
                MarkdownToken::Eof { .. } => break,
                _ => {
                    // 其他token也作为原始文本处理
                    ctx.advance();
                }
            }
        }

        if text.trim().is_empty() {
            None
        } else {
            Some(MarkdownNode::RawText { text, position })
        }
    }

    fn priority(&self) -> i32 {
        -100 // 最低优先级
    }
}

/// 构建所有解析规则
pub fn build_parser_rules(
) -> Vec<Box<dyn ParsingRule<DefaultContext<MarkdownToken>, MarkdownToken, MarkdownNode>>> {
    vec![
        Box::new(HeadingRule),
        Box::new(CodeBlockRule),
        Box::new(ListRule),
        Box::new(ParagraphRule),
        Box::new(RawTextRule),
    ]
}
