# Parser Framework

A parser implementation that follows the CGP (Context-Generic Programming) design pattern.

## Design Philosophy

The framework follows three pillars:

1. **Context** – `ParseContext` defines the abstract interface for reading tokens.
2. **Rule** – `ParsingRule` describes how to turn the current context into an AST node.
3. **Parser** – `Parser` orchestrates rules, honoring priorities and `quick_check`.

This separation:
- decouples parsing rules from concrete parser implementations,
- allows swapping contexts (e.g., batch vs. streaming),
- enables rule prioritization and inexpensive `quick_check` pruning.

## Core Components

### ParseContext
Access to the token stream lives behind a trait:

```rust
pub trait ParseContext<Tok> {
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
Rules convert context into AST nodes:

```rust
pub trait ParsingRule<Ctx, Tok, Ast>
where
    Ctx: ParseContext<Tok>,
    Tok: Clone + std::fmt::Debug,
    Ast: AstNode,
{
    fn try_parse(&mut self, ctx: &mut Ctx) -> Option<Ast>;
    fn priority(&self) -> i32 { 0 }
    fn quick_check(&self, current_token: Option<&Tok>) -> Option<bool> { None }
}
```

### AstNode
AST nodes expose positional information:

```rust
pub trait AstNode: Clone + std::fmt::Debug {
    fn position(&self) -> Option<Position>;
    fn span(&self) -> Option<(Position, Position)> { ... }
}
```

### Parser
The orchestrator that runs rules in order:

```rust
pub struct Parser<Ctx, Tok, Ast> {
    context: Ctx,
    rules: Vec<Box<dyn ParsingRule<Ctx, Tok, Ast>>>,
}
```

## Usage Example

### 1. Define tokens

```rust
#[derive(Debug, Clone)]
enum Token {
    Number(i64),
    Plus,
    Minus,
    Eof,
}
```

### 2. Define AST nodes

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

### 3. Implement rules

```rust
use parser_framework::{ParseContext, ParsingRule};

struct NumberRule;

impl<Ctx, Tok> ParsingRule<Ctx, Tok, Expr> for NumberRule
where
    Ctx: ParseContext<Tok>,
    Tok: Clone + std::fmt::Debug,
{
    fn try_parse(&mut self, ctx: &mut Ctx) -> Option<Expr> {
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

### 4. Run the parser

```rust
use parser_framework::{DefaultContext, Parser};

let tokens = vec![Token::Number(42), Token::Plus, Token::Number(10)];
let rules: Vec<Box<dyn ParsingRule<DefaultContext<Token>, Token, Expr>>> =
    vec![Box::new(NumberRule)];

let mut parser = Parser::from_tokens(tokens, rules);
let ast_nodes = parser.parse();
```

## Streaming Support (`streaming` feature)

Enable the feature to use `StreamingParseContext` and `TokenConsumer` for incremental parsing:

```rust
use parser_framework::{Parser, StreamingParseContext, TokenConsumer};

let context = StreamingParseContext::new();
let mut parser = Parser::new(context, rules);

for token in token_stream {
    let asts = parser.push_token(token);
    // consume finished AST nodes
}

let remaining = parser.finish();
```

This lets lexer → parser operate in the same pipeline for true streaming workflows.

## Features

- ✅ **CGP design** – contexts and rules stay decoupled.
- ✅ **Priority scheduler** – higher priority rules run first.
- ✅ **`quick_check` optimization** – skip rules that cannot match.
- ✅ **Checkpoint system** – safe backtracking when a rule fails.
- ✅ **Position tracking** – consistent line/column/offset metadata.
- ✅ **Error detection** – warn when rules make no progress.

## Integration with the Lexer Framework

1. Use lexer-framework to produce a token stream.
2. Feed the tokens into parser-framework to build AST nodes.

Both frameworks embrace CGP so they compose naturally.

## Architecture Diagram

```
Input text
    ↓
Lexer Framework (lexing)
    ↓
Token stream
    ↓
Parser Framework (parsing)
    ↓
AST nodes
```

Shared concepts:
- **Context** – abstraction over the data source.
- **Rule** – concrete lex/parse logic.
- **Orchestrator** – coordinates rule execution.

Consistency across these layers keeps the toolchain cohesive.

