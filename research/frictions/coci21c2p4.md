# Friction report: coci21c2p4

spec.cp declared `domain State = (comps: 0..=n, used: 0..l, ends: 0..=2)`, the
named-factor form spec/core/04-indices.md shows, and every run was E0501. It
now walks a product of two named domains, `domain State = (Blocks, Cells)`,
and rolls `[State]Z` per magnet (smallest radius first), which also drops the
78M-state dense memo the old top-down form needed at n = 50, l = 1e4.

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
