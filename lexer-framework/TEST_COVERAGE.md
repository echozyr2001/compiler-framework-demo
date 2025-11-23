# Test Coverage Overview

This document summarizes the coverage provided by the lexer-framework test suite.

## Test Suites

### 1. `cursor_test.rs` (24 tests)
- Cursor construction, `peek`, `advance`.
- Position tracking (line, column, offset).
- Unicode handling: Chinese characters, emoji.
- Boundary conditions: empty input, EOF, out-of-range peeks.
- Checkpoint/restore flow.
- Helpers such as `peek_str`, `consume_while`, `advance_by`.

### 2. `position_test.rs` (4 tests)
- Default construction.
- Equality comparisons.
- Copy semantics.

### 3. `context_test.rs` (8 tests)
- `DefaultContext` and `LexContext` basics (peek, advance, consume).
- Position updates.
- Checkpoint + restore.
- EOF detection.
- Empty-input handling.

### 4. `lexer_test.rs` (19 tests)
- Lexer initialization.
- Single/multi-token matches.
- Rule prioritization.
- `quick_check` optimizations.
- `Iterator` implementation.
- Empty input flows.
- Checkpoint safety.

### 5. `quick_check_test.rs` (10 tests)
- Ensures `quick_check` skips impossible rules.
- Confirms allowed matches still get tried.
- Rules that omit `quick_check`.
- Mixed input cases.
- EOF behavior.

### 6. `integration_test.rs` (8 tests)
- End-to-end pipelines: statements, keywords vs. identifiers.
- Multi-line sources.
- Complex expressions (multi-char operators).
- Unicode identifiers.
- Empty/whitespace-only input.
- Priority ordering checks.

### 7. `edge_cases_test.rs` (14 tests)
- `peek_str` boundaries (0 length, beyond EOF).
- Repeated `advance` at EOF.
- `consume_while` edge cases.
- Unicode surrogate handling.
- Iterator adapters (`take`, `filter`).
- Borrowing contexts immutably/mutably.
- Checkpoint stability + reset logic.
- Offset accuracy (ASCII and Unicode).
- Empty rule lists, single-character inputs, cursor cloning.

### 8. `error_handling_test.rs` (5 tests)
- Missing rule sets.
- Inputs where no rule matches.
- Buggy rules that fail to advance.
- Every rule returning `quick_check = false`.
- Extremely long inputs, Unicode edges, nested checkpoints, EOF checks, `size_hint` updates.

### 9. `traits_test.rs` (9 tests)
- Full `LexToken` coverage.
- `LexingRule` defaults.
- Custom priorities and `quick_check` overrides.

### 10. `size_hint_test.rs` (6 tests)
- `size_hint` for ASCII, Chinese text, emoji, and mixed Unicode strings.

## Metrics

- **Test files:** 10  
- **Test cases:** 107+  
- **Covered areas:** cursor/context APIs, lexer core logic, priorities, `quick_check`,
  iterator guarantees, Unicode support, error paths, and integration scenarios.

## Running the Tests

```bash
# Everything
cargo test

# A specific test target
cargo test --test cursor_test

# Only the library tests
cargo test --lib

# Show stdout/stderr
cargo test -- --nocapture
```

## Quality Assurance

- Builds cleanly without warnings.
- Exercises edge cases and error handling.
- Validates Unicode processing and byte/char offsets.

## Continuous Improvement

- Every new feature must land with dedicated tests.
- Edge cases expand alongside new functionality.
- Performance optimizations are guarded by regressions tests.
- Backwards compatibility remains verified via integration coverage.
