use std::io::{self, Write};

use markdown_renderer::{MarkdownNode, MarkdownRenderer, RenderItem, RenderResult};

fn main() {
    println!("=== Markdown 实时渲染终端 ===");
    println!("逐行输入 Markdown 文本，我会在每次输入后重新解析并展示渲染结果。");
    println!("特殊指令：");
    println!("  :show   - 不修改内容，重新渲染当前文档");
    println!("  :clear  - 清空当前文档");
    println!("  :quit   - 退出程序\n");

    let mut renderer = MarkdownRenderer::new();
    let mut document = String::new();
    let stdin = io::stdin();

    loop {
        print!("md> ");
        io::stdout().flush().expect("flush stdout");

        let mut line = String::new();
        if stdin.read_line(&mut line).is_err() {
            println!("\n读取输入失败，退出。");
            break;
        }

        let trimmed = line.trim_end();
        match trimmed {
            ":quit" | ":exit" => {
                println!("再见！");
                break;
            }
            ":clear" => {
                document.clear();
                println!("文档已清空。");
                render_document(&mut renderer, &document);
                continue;
            }
            ":show" => {
                render_document(&mut renderer, &document);
                continue;
            }
            _ => {
                document.push_str(&line);
            }
        }

        render_document(&mut renderer, &document);
    }
}

fn render_document(renderer: &mut MarkdownRenderer, document: &str) {
    println!("\n--- 当前文档（{} 字符）---", document.len());
    if document.is_empty() {
        println!("(空)");
    } else {
        println!("{document}");
    }

    let nodes = renderer.parse(document);
    print_ast_summary(&nodes);
    let result = renderer.get_render_result(&nodes);
    display_render_items(&result);
    println!("--------------------------------\n");
}

fn print_ast_summary(nodes: &[MarkdownNode]) {
    println!("AST 节点 ({} 个):", nodes.len());
    if nodes.is_empty() {
        println!("  (空)");
        return;
    }

    for (idx, node) in nodes.iter().enumerate() {
        println!("  [{}] {:?}", idx, node);
    }
}

fn display_render_items(result: &RenderResult) {
    println!("渲染输出:");
    if result.items.is_empty() {
        println!("  (无渲染项)");
        return;
    }

    for item in &result.items {
        match item {
            RenderItem::Heading { level, text } => {
                println!("  [H{}] {}", level, text);
            }
            RenderItem::Paragraph(text) => {
                println!("  [P] {}", text);
            }
            RenderItem::List(items) => {
                println!("  [LIST]");
                for entry in items {
                    println!("    - {}", entry);
                }
            }
            RenderItem::CodeBlock { language, code } => {
                if let Some(lang) = language {
                    println!("  [CODE:{}]", lang);
                } else {
                    println!("  [CODE]");
                }
                for line in code.lines() {
                    println!("    {}", line);
                }
            }
            RenderItem::RawText(text) => {
                println!("  [RAW] {}", text);
            }
        }
    }
}
