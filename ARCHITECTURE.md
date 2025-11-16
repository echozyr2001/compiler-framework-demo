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

