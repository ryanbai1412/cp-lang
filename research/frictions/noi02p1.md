# Friction Report: noi02p1

## 1. An enum with domain-element payload fields doesn't read by tag (`compiler`, already filed)

- **Where:** `domain Ship = 1..=30000` / `enum Cmd = M(i: Ship, j: Ship) | C(i: Ship, j: Ship)` /
  `for cmd in read<[t]Cmd>(): ...` on `M 2 3` is R0501 "expected a number, found `M`" in `cpl run`,
  and the C++ build stops with "something else where a number was expected". The reader treats the
  finite enum as a domain and reads an ordinal (`M 2` alone reads `2` as `M(3)`).
- **Repro:** `domain Ship = 1..=5` / `enum Cmd = M(i: Ship) | C(i: Ship)` /
  `match read<Cmd>(): M(i) => print(i)` / `C(i) => print(i)` on input `M 2`.
- **Goal it hurts:** express (the bound is written five times, and every use converts with `Ship(i)`).
- **General fix:** the tagged-enum reader reads a domain-typed payload field like `read<Ship>()`
  (R10.2.3). Routed to compiler from this problem (`plan/domains/compiler/worklog.md`, 2026-11-22).
- **Workaround:** payload fields are `1..=30000`, converted with `Ship(i)`.
- **Severity:** medium. **Status:** fixed: the port reads `M(i: Ship, j: Ship)` and needs no
  conversions.

## 2. `PotentialDsu` roots by size, so the column's front is kept beside it

- **Where:** `var head: [Ship]?Ship` and `fn front(v)`, patched after every `unite`.
- **Ideal code (`dream.cp`):** `line.unite(line.find(j).root, line.find(i).root, diff: line.size(j))`,
  with the front ship being the root.
- **What happened:** the root is whichever set was larger, not the column's front, so the port
  tracks the front per root by hand: two facts that must agree.
- **General fix:** let `PotentialDsu.unite` keep a chosen root (or keep the potential origin as a
  separate per-set value), so potentials are measured from a stable element.
- **Severity:** low.

## Measurements

Contestant-written tokens: port 177, original 218 (`corpus/audit.py` count). Emitted C++: 16327 bytes (`cpl build`, judge `generic`).
