# cpl-diag

Spans, stable error codes and rendering shared by every tool, so an error reads the same on the command line and in the editor.

## Invariants

- Codes are stable and catalogued in `spec/diagnostics.md`.
- Every tool reports through these types; none renders its own.

## Tests

Inline `#[cfg(test)]` modules; catalogue checks in `cpl-cli/tests/catalogue.rs`.

Where it sits among the crates: `handbook/design/architecture.md`.
