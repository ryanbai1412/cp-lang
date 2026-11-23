# cpl-json

The one JSON implementation: value type, RFC 8259 parser, deterministic writer.

## Invariants

- Zero dependencies.
- Objects are `BTreeMap`s: output and equality do not depend on insertion order; duplicate keys resolve last-wins.

## Tests

Inline `#[cfg(test)]` module in `src/lib.rs`.

Where it sits among the crates: `handbook/design/architecture.md`.
