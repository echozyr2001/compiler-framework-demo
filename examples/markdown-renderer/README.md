# Markdown Renderer Example

这是一个使用 CGP 框架构建所见即所得 Markdown 编辑器的示例项目。它展示了如何将编译原理的思想（词法分析 → 语法分析 → AST）应用于实时文本渲染场景。

## 设计理念

### 把 Markdown 实时渲染看作"连续的编译任务"

与传统的一次性编译不同，所见即所得编辑器需要在用户每次输入时都重新解析文档。这个示例展示了如何用框架的通用能力来处理这种场景：

1. **词法分析**：将文本转换为 Markdown 的 Token 流
2. **语法分析**：将 Token 流解析为 AST 节点
3. **状态管理**：利用框架的 `StatefulNode` trait 为 AST 节点添加状态信息（Incomplete/Complete）
4. **渲染决策**：根据节点状态决定如何显示（原始文本 vs 渲染后的格式）

### 框架保持通用性

- 框架不包含任何 Markdown 特定的逻辑
- `StatefulNode` trait 允许为 AST 节点添加任意状态类型
- 同样的框架可以用于构建编译器、DSL 解析器等

## 项目结构

```
markdown-renderer/
├── src/
│   ├── lib.rs              # 库入口
│   ├── token.rs            # Markdown Token 定义
│   ├── ast.rs              # Markdown AST 节点（实现 StatefulNode）
│   ├── state.rs            # 内容状态定义（Incomplete/Complete）
│   ├── lexer_rules.rs      # 词法规则（Hash、Backtick、Newline 等）
│   ├── parser_rules.rs     # 语法规则（标题、段落、列表、代码块等）
│   └── renderer.rs         # 渲染引擎（编排词法分析和语法分析）
└── examples/
    ├── interactive_editor.rs # 预设场景：多步输入、不同语法
    └── live_terminal.rs      # 终端实时输入，立即查看渲染结果
```

## 核心组件

### 1. Token 定义

`MarkdownToken` 枚举定义了所有 Markdown 语法元素的 Token：
- 结构化：`Hash` (标题), `Newline`, `Text`
- 列表：`Dash`, `Asterisk`
- 代码：`Backtick` (支持 1-3 个反引号)
- 强调：`Star`, `Underscore`
- 链接：`LeftBracket`, `RightBracket`, `LeftParen`, `RightParen`

### 2. AST 节点

`MarkdownNode` 实现了 `StatefulNode` trait，可以携带 `ContentState` 状态：
- `Heading`：标题（level 1-6）
- `Paragraph`：段落
- `List`：列表（支持多项目）
- `CodeBlock`：代码块（可指定语言）
- `RawText`：原始文本（用于未确定的内容）

### 3. 状态管理

`ContentState` 枚举定义了两种状态：
- `Incomplete`：内容正在输入，语法未完整（显示原始文本）
- `Complete`：内容完整，可以渲染（显示格式化后的内容）

### 4. 解析规则

解析规则实现了 `ParsingRule` trait，包括：
- `HeadingRule`：匹配 `# ` 开头的标题
- `CodeBlockRule`：匹配 ``` 包裹的代码块
- `ListRule`：匹配 `- ` 或 `* ` 开头的列表
- `ParagraphRule`：匹配普通段落
- `RawTextRule`：兜底规则（当其他规则都不匹配时）

### 5. 渲染引擎

`MarkdownRenderer` 编排整个解析流程：
1. 创建 Lexer 进行词法分析
2. 创建 Parser 进行语法分析
3. 根据 AST 节点的状态生成渲染结果

## 使用示例

```rust
use markdown_renderer::MarkdownRenderer;

let mut renderer = MarkdownRenderer::new();

// 解析输入
let nodes = renderer.parse("# Hello\n");

// 获取渲染结果
let result = renderer.get_render_result(&nodes);

// 根据 RenderItem 类型进行渲染
for item in &result.items {
    match item {
        RenderItem::Heading { level, text } => {
            // 渲染为 H1-H6 标题
        }
        RenderItem::Paragraph(text) => {
            // 渲染为段落
        }
        RenderItem::RawText(text) => {
            // 显示原始文本（用于 Incomplete 状态）
        }
        // ...
    }
}
```

## 运行示例

```bash
# 预设场景：演示标题/段落/列表/代码块等语法
cargo run --example interactive_editor --manifest-path examples/markdown-renderer/Cargo.toml

# 终端实时输入：逐行输入 Markdown，立即查看解析/渲染结果
cargo run --example live_terminal --manifest-path examples/markdown-renderer/Cargo.toml
```

`interactive_editor` 会展示几个典型场景：
1. 用户逐步输入标题（展示状态从 Incomplete 到 Complete 的转换）
2. 多行文档解析
3. 代码块解析（展示需要配对符号的语法）
4. 列表解析

`live_terminal` 则提供了 REPL 式体验：
- 逐行输入 Markdown 文本即可即时解析并打印渲染结果
- 支持 `:show`（仅渲染）、`:clear`（清空文档）、`:quit`（退出）等指令
- 方便验证“实时解析 + 状态渲染”的实际效果

## 关键设计点

### 1. 状态管理在 AST 节点上

通过实现 `StatefulNode` trait，每个 AST 节点都可以携带状态信息。这使得解析器可以在解析时就判断内容的完整性（如标题是否有换行、代码块是否闭合等）。

### 2. 每次编辑重新运行 Pipeline

对于小到中等大小的文档，全量重新解析非常快速（毫秒级）。这种设计：
- 简单、可靠
- 避免复杂的增量更新逻辑
- 如果性能成为问题，可以在应用层添加缓存

### 3. 渲染决策在应用层

框架只负责解析和生成 AST，渲染决策完全由应用层控制。`RenderResult` 清晰地告诉 UI 层应该如何显示每种内容。

## 扩展性

这个示例可以轻松扩展：

1. **添加更多 Markdown 语法**：
   - 表格
   - 引用块
   - 任务列表
   - 等等

2. **添加更多状态**：
   - 除了 `Incomplete` 和 `Complete`，可以添加 `Confirmed`（用户明确确认）
   - 可以添加错误状态

3. **性能优化**：
   - 添加 AST 缓存
   - 实现增量更新（只重新解析修改的部分）
   - 行级缓存

## 与编译器的关系

虽然这个示例是关于编辑器的，但它展示了如何用编译器技术（词法分析、语法分析）解决非传统编译场景的问题。框架本身是为构建编译器而设计的，这个示例展示了它的通用性。

## 学习价值

- **编译原理应用**：展示如何将经典的编译器前端技术应用于新场景
- **状态管理**：展示如何用 trait 实现灵活的状态管理机制
- **框架设计**：展示如何设计通用的框架，既支持传统场景（编译器）又支持新场景（编辑器）

