use markdown_renderer::{MarkdownNode, MarkdownRenderer, RenderItem, RenderResult};

fn main() {
    let mut renderer = MarkdownRenderer::new();

    println!("=== Markdown编辑器模拟 ===\n");

    run_progressive_case(
        &mut renderer,
        "场景1：用户输入标题",
        &["#", "# Hello", "# Hello\n"],
    );

    run_single_case(
        &mut renderer,
        "场景2：多行文档",
        "# Title\n\nThis is a paragraph.\n\n## Subtitle\n",
    );

    run_progressive_case(
        &mut renderer,
        "场景3：代码块",
        &["```rust\nfn main() {", "```rust\nfn main() {}\n```"],
    );

    run_single_case(&mut renderer, "场景4：列表", "- Item 1\n- Item 2\n");
}

fn run_progressive_case(renderer: &mut MarkdownRenderer, title: &str, inputs: &[&str]) {
    println!("\n{title}");
    for (idx, input) in inputs.iter().enumerate() {
        let step_label = format!("步骤{}", idx + 1);
        run_case(renderer, &step_label, input);
    }
}

fn run_single_case(renderer: &mut MarkdownRenderer, title: &str, input: &str) {
    println!("\n{title}");
    run_case(renderer, "示例", input);
}

fn run_case(renderer: &mut MarkdownRenderer, label: &str, input: &str) {
    println!("\n[{label}] 输入:\n{input}");
    let nodes = renderer.parse(input);
    print_ast(&nodes);
    let result = renderer.get_render_result(&nodes);
    display_result(&result);
}

fn print_ast(nodes: &[MarkdownNode]) {
    println!("解析的AST节点:");
    if nodes.is_empty() {
        println!("  (无节点，可能输入为空)");
        return;
    }

    for (i, node) in nodes.iter().enumerate() {
        println!("  节点{}: {:?}", i, node);
    }
}

fn display_result(result: &RenderResult) {
    println!("渲染输出:");
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
                for item in items {
                    println!("    - {}", item);
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
