# Pipeline Core

A lightweight orchestrator for coordinating lexer and parser in both batch and streaming modes.

## Features

- **Default (batch mode)**: Processes input in two stages: tokenize entire input, then parse all tokens. This is the default mode.
- **`streaming`** (optional): Enables streaming pipeline functionality. When enabled, provides the `StreamingPipeline` struct for coordinating lexer and parser in a streaming fashion.

## Usage

### Batch Mode (Default)

By default, `pipeline-core` provides `BatchPipeline` for processing input in two stages:

```toml
[dependencies]
pipeline-core = { path = "../pipeline-core" }
lexer-framework = { path = "../lexer-framework" }
parser-framework = { path = "../parser-framework" }
```

```rust
use pipeline_core::BatchPipeline;
use lexer_framework::{Lexer, LexingRule};
use parser_framework::{Parser, ParsingRule};

// Define your lexer and parser rules
let lexer_rules: Vec<Box<dyn LexingRule<_, Token>>> = /* ... */;
let parser_rules: Vec<Box<dyn ParsingRule<_, Token, Ast>>> = /* ... */;

// Run the batch pipeline
let pipeline = BatchPipeline::new();
let asts = pipeline.run(input, lexer_rules, parser_rules);
```

### Streaming Mode

To use the streaming pipeline functionality, enable the `streaming` feature:

```toml
[dependencies]
pipeline-core = { path = "../pipeline-core", features = ["streaming"] }
lexer-framework = { path = "../lexer-framework", features = ["streaming"] }
parser-framework = { path = "../parser-framework", features = ["streaming"] }
```

When the `streaming` feature is enabled, you can use the `StreamingPipeline` struct (or the `Pipeline` type alias for backward compatibility):

```rust
use pipeline_core::StreamingPipeline; // or Pipeline for backward compatibility
use lexer_framework::streaming::FilteringTokenProducer;
use parser_framework::streaming::StreamingParseContext;

let lexer = FilteringTokenProducer::new(/* ... */);
let parser = Parser::new(StreamingParseContext::new(), /* ... */);
let pipeline = StreamingPipeline::new(lexer, parser);
let asts = pipeline.run();
```

## Design Philosophy

- **High Cohesion**: Pipeline logic is self-contained and focused on orchestration
- **Low Coupling**: Only depends on trait interfaces, not concrete implementations
- **Dual Mode Support**: Supports both batch (default) and streaming (optional) processing modes
- **Feature Gating**: Streaming functionality is completely gated behind the `streaming` feature flag

