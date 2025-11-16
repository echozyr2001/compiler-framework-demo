# Common Framework

提供 lexer-framework 和 parser-framework 之间的共同组件。

## 组件

### Position

表示源代码中 token 或 AST 节点的位置信息。

```rust
use common_framework::Position;

let pos = Position::new();  // 创建初始位置 (line: 1, column: 1, offset: 0)
let pos = Position::at(5, 10, 100);  // 创建指定位置
```

#### 字段

- `line`: 行号（从 1 开始）
- `column`: 列号（从 1 开始）
- `offset`: 从输入开始的字节偏移量

## 使用

这个 crate 被 lexer-framework 和 parser-framework 共同使用，确保两个框架使用相同的位置表示。

