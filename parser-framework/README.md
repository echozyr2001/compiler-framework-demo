# Parser Framework

基于 CGP (Context-Generic Programming) 设计模式的解析器框架。

## 设计理念

Parser Framework 遵循 CGP 设计模式，核心思想是：

1. **Context（上下文）**: `ParseContext` trait 定义了解析操作的抽象接口
2. **Rule（规则）**: `ParsingRule` trait 定义了如何从上下文中解析 AST 节点
3. **Parser（解析器）**: `Parser` 结构体协调规则的应用，按优先级尝试匹配

这种设计使得：
- 规则与具体的解析器实现解耦
- 可以轻松替换不同的上下文实现
- 支持优先级和快速检查优化

## 核心组件

### ParseContext

解析上下文 trait，提供对 token 流的访问：

```rust
pub trait ParseContext<'input, Tok> {
    fn peek(&self) -> Option<&Tok>;
    fn peek_at(&self, offset: usize) -> Option<&Tok>;
    fn advance(&mut self) -> Option<Tok>;
    fn position(&self) -> Position;
    fn is_eof(&self) -> bool;
    fn token_index(&self) -> usize;
    fn checkpoint(&self) -> Checkpoint;
    fn restore(&mut self, checkpoint: Checkpoint);
}
```

### ParsingRule

解析规则 trait，定义如何从上下文中解析 AST 节点：

```rust
pub trait ParsingRule<'input, Ctx, Tok, Ast>
where
    Ctx: ParseContext<'input, Tok>,
    Tok: Clone + std::fmt::Debug,
    Ast: AstNode,
{
    fn try_parse(&mut self, ctx: &mut Ctx) -> Option<Ast>;
    fn priority(&self) -> i32 { 0 }
    fn quick_check(&self, current_token: Option<&Tok>) -> Option<bool> { None }
}
```

### AstNode

AST 节点 trait，定义节点必须提供的位置信息：

```rust
pub trait AstNode: Clone + std::fmt::Debug {
    fn position(&self) -> Option<Position>;
    fn span(&self) -> Option<(Position, Position)> { ... }
}
```

### Parser

解析器结构体，协调规则的应用：

```rust
pub struct Parser<'input, Ctx, Tok, Ast> {
    context: Ctx,
    rules: Vec<Box<dyn ParsingRule<'input, Ctx, Tok, Ast> + 'input>>,
}
```

## 使用示例

### 1. 定义 Token 类型

```rust
#[derive(Debug, Clone)]
enum Token {
    Number(i64),
    Plus,
    Minus,
    Eof,
}
```

### 2. 定义 AST 节点类型

```rust
use parser_framework::{AstNode, Position};

#[derive(Debug, Clone)]
enum Expr {
    Number { value: i64, position: Position },
    Binary { op: Op, left: Box<Expr>, right: Box<Expr>, position: Position },
}

impl AstNode for Expr {
    fn position(&self) -> Option<Position> {
        Some(match self {
            Expr::Number { position, .. } => *position,
            Expr::Binary { position, .. } => *position,
        })
    }
}
```

### 3. 实现解析规则

```rust
use parser_framework::{ParseContext, ParsingRule, Position};

struct NumberRule;

impl<'input, Ctx, Tok> ParsingRule<'input, Ctx, Tok, Expr> for NumberRule
where
    Ctx: ParseContext<'input, Tok>,
    Tok: Clone + std::fmt::Debug + PartialEq,
{
    fn try_parse(&mut self, ctx: &mut Ctx) -> Option<Expr> {
        // 检查当前 token 是否为数字
        if let Some(Token::Number(n)) = ctx.peek() {
            let position = ctx.position();
            ctx.advance();
            Some(Expr::Number { value: *n, position })
        } else {
            None
        }
    }

    fn priority(&self) -> i32 {
        10
    }

    fn quick_check(&self, current_token: Option<&Tok>) -> Option<bool> {
        current_token.map(|t| matches!(t, Token::Number(_)))
    }
}
```

### 4. 使用 Parser

```rust
use parser_framework::{DefaultContext, Parser};

let tokens = vec![Token::Number(42), Token::Plus, Token::Number(10)];
let rules: Vec<Box<dyn ParsingRule<'_, DefaultContext<Token>, Token, Expr> + '_>> =
    vec![Box::new(NumberRule)];

let mut parser = Parser::from_tokens(tokens, rules);
let nodes = parser.parse();
```

## 特性

- ✅ **CGP 设计模式**: 上下文与规则解耦
- ✅ **优先级系统**: 规则按优先级排序，高优先级先尝试
- ✅ **快速检查优化**: `quick_check` 方法可以快速跳过不匹配的规则
- ✅ **检查点机制**: 支持回溯，规则失败时自动恢复状态
- ✅ **位置跟踪**: 自动跟踪 token 和 AST 节点的位置信息
- ✅ **错误处理**: 检测死循环和解析错误

## 与 Lexer Framework 集成

Parser Framework 设计为与 Lexer Framework 配合使用：

1. 使用 Lexer Framework 将输入文本转换为 token 流
2. 使用 Parser Framework 将 token 流解析为 AST

两个框架都遵循 CGP 设计模式，可以无缝集成。

## 架构设计

```
输入文本
    ↓
Lexer Framework (词法分析)
    ↓
Token 流
    ↓
Parser Framework (语法分析)
    ↓
AST 节点
```

两个框架都使用相同的设计模式：
- **Context**: 抽象的操作接口
- **Rule**: 具体的匹配/解析逻辑
- **Orchestrator**: 协调规则的应用

这种一致性使得两个框架可以很好地协同工作。

