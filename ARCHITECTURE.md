# CGP (Context-Generic Programming) 词法分析器架构

本词法分析器基于 **CGP (Context-Generic Programming)** 设计模式构建，该模式的核心思想是**将上下文与规则分离**，使词法规则可以在不同的上下文中复用。

## 核心组件

### 1. `Cursor<'input>`
- **作用**: 遍历输入文本的游标
- **功能**: 
  - 跟踪当前位置（行、列、字节偏移）
  - 提供 peek/advance 操作
  - 支持检查点和恢复
- **位置**: `src/lexer/cursor.rs`

### 2. `LexContext<'input>` (Trait)
- **作用**: 词法分析的上下文抽象
- **功能**:
  - 封装 Cursor 的操作
  - 提供统一的上下文接口
  - 允许自定义上下文实现
- **位置**: `src/lexer/context.rs`

### 3. `LexingRule<'input, Ctx, Tok>` (Trait)
- **作用**: 词法规则的接口
- **功能**:
  - `try_match`: 尝试匹配并消费一个 token
  - `priority`: 返回规则优先级（高优先级优先尝试）
- **位置**: `src/lexer/traits.rs`

### 4. `Lexer<'input, Ctx, Tok>`
- **作用**: 词法分析器主结构
- **功能**:
  - 管理规则列表（按优先级排序）
  - 按顺序尝试规则匹配
  - 支持 tokenize 批量处理
- **位置**: `src/lexer/lexer.rs`

### 5. `LexToken` (Trait)
- **作用**: Token 的通用接口
- **功能**: 提供 token 的查询方法（位置、类型判断等）
- **位置**: `src/lexer/traits.rs`

## CGP 设计模式的优势

1. **上下文与规则分离**: 规则不直接依赖具体的 lexer 实现，而是通过 `LexContext` trait 交互
2. **可扩展性**: 可以轻松添加新的规则，只需实现 `LexingRule` trait
3. **可复用性**: 规则可以在不同的上下文中使用（例如：标准上下文、带缓存的上下文等）
4. **优先级机制**: 通过 `priority()` 方法控制规则匹配顺序
5. **错误恢复**: 通过检查点机制，规则失败时可以恢复到匹配前的状态

## 使用示例

```rust
use cp_test::lexer::{default_rules, Lexer};

let input = "let x = 42";
let rules = default_rules();
let mut lexer = Lexer::from_str(input, rules);

for token in lexer.tokenize() {
    println!("{:?}", token);
}
```

## 实现自定义规则

要实现自定义规则，只需实现 `LexingRule` trait:

```rust
pub struct MyCustomRule;

impl<'input, Ctx> LexingRule<'input, Ctx, SimpleToken> for MyCustomRule
where
    Ctx: LexContext<'input>,
{
    fn try_match(&mut self, ctx: &mut Ctx) -> Option<SimpleToken> {
        // 检查是否匹配
        if ctx.peek() == Some('#') {
            let position = ctx.position();
            ctx.advance();
            // ... 处理逻辑
            Some(/* token */)
        } else {
            None
        }
    }

    fn priority(&self) -> i32 {
        15 // 设置优先级
    }
}
```

## 文件结构

```
src/
├── lib.rs              # 库入口
├── position.rs         # 位置信息
└── lexer/
    ├── mod.rs          # 模块导出
    ├── cursor.rs       # 游标实现
    ├── context.rs      # 上下文 trait
    ├── traits.rs       # 核心 traits (LexToken, LexingRule)
    ├── lexer.rs        # 词法分析器主结构
    └── rules.rs        # 内置规则实现
```

## 应用示例

框架设计为通用解析框架，可以用于构建多种类型的工具。我们提供了示例来展示框架的应用：

### Markdown 渲染引擎示例

位置：`examples/markdown-renderer/`

这个示例展示了如何用框架构建所见即所得的 Markdown 编辑器后端。它展示了编译原理思想在非传统编译场景中的应用：

1. **词法分析**：将 Markdown 文本转换为 Token 流
2. **语法分析**：将 Token 流解析为 AST 节点
3. **状态管理**：利用 `StatefulNode` trait 为 AST 节点添加状态信息（Incomplete/Complete）
4. **渲染决策**：根据节点状态决定如何显示内容

#### 关键特性

- **通用性**：框架不包含任何 Markdown 特定的逻辑，`StatefulNode` trait 允许为 AST 节点添加任意状态类型
- **编译原理应用**：展示了如何将词法分析→语法分析→AST 的经典流程应用于实时文本渲染场景
- **状态管理**：展示了如何用 trait 实现灵活的状态管理机制

详细的实现说明和使用方法请参考 `examples/markdown-renderer/README.md`。

### 运行示例

```bash
# 运行 Markdown 渲染引擎示例
cargo run --example interactive_editor --manifest-path examples/markdown-renderer/Cargo.toml
```

## 框架通用能力

### StatefulNode Trait

框架提供了 `StatefulNode` trait（在 `parser-framework` 中），允许 AST 节点携带任意用户定义的状态信息。这对于多种场景都很有用：

- **编辑器场景**：标记内容是否完整（Incomplete/Complete）
- **编译器场景**：错误恢复状态、部分解析状态
- **IDE/LSP**：语法高亮状态、诊断状态等

```rust
pub trait StatefulNode: AstNode {
    type State: Clone + std::fmt::Debug;
    
    fn state(&self) -> &Self::State;
    fn set_state(&mut self, state: Self::State);
    fn transition(&mut self, trigger: &dyn Any) -> bool { false }
}
```

这个设计保持了框架的通用性：框架不关心具体的状态类型，完全由用户定义。

