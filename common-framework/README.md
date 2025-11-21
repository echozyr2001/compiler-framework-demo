# Common Framework

`common-framework` is the lowest layer in the workspace. It defines **positions, checkpoints, immutable text slices,** and the **streaming protocol** that higher-level crates share. Any consumer—whether it is our lexer, parser, or an external project—can rely on these types to exchange data without adopting the rest of the stack.

## Modules

| Module        | Purpose                                                                                       |
| ------------- | --------------------------------------------------------------------------------------------- |
| `position`    | Provides `Position`, a uniform `(line, column, offset)` marker for tokens and AST nodes.      |
| `checkpoint`  | Captures progress plus position so lexers/parsers can rollback or implement lazy evaluation.  |
| `text_slice`  | Owns an `Arc<str>` and exposes immutable slices with value semantics—great for DSL lexers.    |
| `streaming`   | Declares `StreamingSignal`, `Inbound`, and `Outbound`, enabling real-time / incremental flows.|

## Position

```rust
use common_framework::Position;

let origin = Position::new();              // line=1, column=1, offset=0
let custom = Position::at(5, 10, 100);
assert_eq!(custom.column, 10);
```

Lexers attach positions to tokens, parsers keep them on AST nodes for diagnostics. `Position` implements `Default`, `Copy`, `Eq`, and `Hash`, so it fits nicely inside larger structs or maps.

## Checkpoint

```rust
use common_framework::{Checkpoint, Position};

let cp = Checkpoint::new(42, Position::at(3, 5, 42));
assert_eq!(cp.index(), 42);          // treated as byte offset in lexers
assert_eq!(cp.token_index(), 42);    // treated as token index in parsers
```

- `LexContext::checkpoint` stores the current byte offset before trying a rule.
- `ParseContext::checkpoint` stores the current token index before trying a grammar rule.
- After a successful parse, contexts may call `commit()` to discard obsolete checkpoints.

## TextSlice

`TextSlice` keeps an `Arc<str>` alive and exposes a view into it. That eliminates lifetime juggling and lets rules return “matched text” by value:

```rust
use common_framework::TextSlice;
use std::sync::Arc;

let buffer: Arc<str> = Arc::from("hello world");
let slice = TextSlice::new(buffer.clone(), 0, 5);
assert_eq!(&*slice, "hello");
```

It implements `Deref<Target=str>`, `Display`, and equality with both `&str` and other `TextSlice`s.

## Streaming protocol

The `streaming` module offers a lightweight message protocol for real-time or incremental pipelines:

```rust
use common_framework::{StreamingSignal, Inbound, Outbound};

pub struct Controller;

impl<Tok, Ast> Inbound<Tok, Ast> for Controller {
    fn handle_signal(&mut self, signal: StreamingSignal<Tok, Ast>) {
        match signal {
            StreamingSignal::Produced(nodes) =>
                println!("parser produced {}", nodes.len()),
            StreamingSignal::NeedToken(n) =>
                println!("parser needs {n} more tokens"),
            _ => {}
        }
    }
}
```

- `StreamingSignal` covers “request token”, “supply token”, “EOF”, “abort”, etc.
- `Inbound` / `Outbound` let any component consume or emit those signals.
- `lexer-framework::streaming` and `parser-framework::streaming` already implement the traits, so you can wire them together to form a streaming pipeline.

## Integration

Add it to `Cargo.toml`:

```toml
[dependencies]
common-framework = { path = "../common-framework" }
```

Then import the types you need:

```rust
use common_framework::{Position, Checkpoint, TextSlice, StreamingSignal};
```

With that, your crate speaks the same “language” as the lexer and parser frameworks—positions, checkpoints, streaming signals are all shared and interoperable.
