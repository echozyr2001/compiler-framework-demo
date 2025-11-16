# Parser Example

本 crate 展示了如何使用 `parser-framework` 和 `lexer-framework` 配合工作，实现一个完整的计算器表达式解析器。

## 架构说明

### 完整流程

```
输入文本: "3 + 4 * 5"
    ↓
[Lexer Framework] 词法分析
    ↓
Token 流: [Number(3), Plus, Number(4), Multiply, Number(5)]
    ↓
[Parser Framework] 语法分析
    ↓
AST 节点: Binary(Add, Number(3), Binary(Multiply, Number(4), Number(5)))
```

### CGP 设计模式体现

1. **Lexer Framework (词法分析层)**
   - Context: `LexContext` - 字符流操作接口
   - Rule: `LexingRule` - 词法规则（匹配字符序列）
   - Orchestrator: `Lexer` - 协调规则应用

2. **Parser Framework (语法分析层)**
   - Context: `ParseContext` - Token 流操作接口
   - Rule: `ParsingRule` - 语法规则（解析 AST 节点）
   - Orchestrator: `Parser` - 协调规则应用

## 运行示例

```bash
# 运行简单解析器示例（推荐先看这个）
cargo run --bin simple-parser

# 运行计算器解析器示例
cargo run --bin calc-parser
```

## 示例说明

### simple-parser（推荐入门）

一个最简单的解析器示例，展示 parser-framework 的基本用法：

- **目的**: 快速理解 parser-framework 的核心概念
- **功能**: 解析简单的算术表达式（数字 + 运算符 + 数字）
- **特点**:
  - 代码简洁，易于理解
  - 展示了完整的解析流程
  - 包含详细的注释和说明

**运行示例**:
```bash
cargo run --bin simple-parser
```

**输出示例**:
```
【示例 1】解析单个数字:
输入: 42
AST: Number { value: 42, ... }

【示例 2】解析加法表达式:
输入: 3 + 4
AST: Binary { op: Add, left: Number(3), right: Number(4) }

【示例 3】解析减法表达式:
输入: 10 - 3
AST: Binary { op: Subtract, left: Number(10), right: Number(3) }
```

### calc-parser

一个完整的计算器表达式解析器，支持：

- **数字**: 整数和浮点数（如 `3`, `3.14`）
- **运算符**: `+`, `-`, `*`, `/`, `^` (幂)
- **括号**: `()` 用于改变优先级
- **一元运算符**: 负号 `-5`
- **运算符优先级**: 
  - 括号 > 幂 > 乘除 > 加减

### 使用步骤

1. **定义 Token 类型**: 实现 `LexToken` trait
2. **实现 Lexer 规则**: 实现 `LexingRule` trait 来匹配 token
3. **定义 AST 节点**: 实现 `AstNode` trait
4. **实现 Parser 规则**: 实现 `ParsingRule` trait 来解析 AST
5. **组合使用**: 使用 `Lexer` 生成 token 流，然后用 `Parser` 解析为 AST

## 代码结构

```
parser-example/
├── Cargo.toml              # 依赖配置
├── README.md               # 本文件
└── src/
    ├── lib.rs              # 库代码（如果有共享代码）
    └── bin/
        ├── simple_parser.rs  # 简单解析器示例（推荐入门）
        └── calc_parser.rs    # 计算器解析器示例
```

## 学习要点

1. **CGP 模式**: 观察如何通过 Context 和 Rule 实现解耦
2. **优先级处理**: 了解如何通过规则优先级处理运算符优先级
3. **递归解析**: 观察如何处理嵌套结构（如括号表达式）
4. **错误处理**: 了解解析失败时的检查点恢复机制
5. **位置跟踪**: 观察如何从 token 传递位置信息到 AST 节点

