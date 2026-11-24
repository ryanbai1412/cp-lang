# Friction report: yac6p3

`dream.cp` is verified. The port and dream use the same five queries, but are written differently:
- The dream answers with `near(x: X, y: Y) -> 0..=2e9 = ask(...)` and inline sum/difference arithmetic, with an `is_mine` helper made of nested `if let`.
- The port names `Cell = (X, Y)` and a `cell(sum, diff) -> ?Cell` helper, and picks the pairing with one `if let ..., near(first) == 0` chain.

The port is the clearer of the two. The dream should take its `cell` helper when it is next verified.

## `ask` needs a `let` to take its range
- **Where:** port lines 22-24: `fn near(c: Cell) -> Int: let d: 0..=2e9 = ask("?", c.0, c.1); d`.
- **What felt wrong:** a function's return type should carry the reply's range, as in the dream's `-> 0..=2e9 = ask(...)`. The port writes the binding only to give `ask` its type.
- **Goal it hurts:** express.
- **Ideal code:** `fn near(c: Cell) -> 0..=2e9 = ask("?", c.0, c.1)`.
- **General fix:** take `ask`'s reply type from any expected type, not only a `let` annotation (the E0501 repro sent to Core). Owner: core.
- **Severity:** low.

## Measurements

- **Tokens:** port 301, accepted C++ 418 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.00 s / 3 MB, accepted C++ 0.00 s / 3 MB.
- **Emitted C++:** 17052 bytes.
