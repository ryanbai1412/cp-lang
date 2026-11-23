# cpl-syntax

Lexer, parser and lossless syntax tree: `lex` makes layout tokens explicit, `split_items` cuts top-level items, `parse_item` parses one item with item-relative spans.

## Invariants

- Parsing never fails: broken source gives a full tree with `Missing` and `Error` nodes and one diagnostic per root cause.
- Spans are item-relative, so an edit re-parses only the items it touches.

## Tests

`src/tests.rs`, `src/foreign_tests.rs` (`cargo nextest run -p cpl-syntax`); grammar cases in `spec/conformance/core-01`.

Where it sits among the crates: `plan/ARCHITECTURE.md`.
