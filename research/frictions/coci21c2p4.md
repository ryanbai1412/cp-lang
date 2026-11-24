# Friction report: coci21c2p4

Rethought. The v0.2 port left n and l untyped (the bounds were a comment) and kept the table as nested lists. The v0.3 port types n, l and every radius (`1..=l`) from the statement and keeps the table as a `[0..=n, 0..=l]Z` grid. It skips zero states with a filtered loop and sums the answer with `sum`. l stays `1..=1e4` rather than `n..=1e4` because one package case has l < n (answer 0).

dream.cp is a memo over a `State` product domain whose out-of-range moves fall to 0 through `State.at(..)`. The port keeps the forward table and bounds-checks each move by hand (`k + r <= l`), because that product domain does not build yet:

## Named product factors over inline ranges are E0501

- **Severity:** medium
- **Status:** open (routed to compiler)
- **Goal it hurts:** express and revise
- **Area:** lowering (`crates/cpl-ir/src/lower/stmt.rs`, domain definitions)
- **What happened:** `domain State = (comps: 0..=3, used: 0..4); let s =
  State.at((1, 2))!; print(s.comps)` is E0501 "a domain expression that does
  not produce a range, count, or index space", then E0302 on `State.at`.
  `domain State = (blocks: B, cells: C)` over named `B`, `C` works.
- **General fix:** lower a product whose factors are inline ranges by
  interning an anonymous domain per factor, as named factors already do.

## Measurements

gen.py --max (n = 50, l = 1e4): port 0.06 s, accepted C++ 0.93 s, same output.
