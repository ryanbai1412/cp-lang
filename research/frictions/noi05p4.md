# Friction report: noi05p4

## `mean` of a non-empty chain is still optional

- **Where:** `1.0 + chain([m], g.next(m)).mean(of: m2 => turns(c2, m2))!`.
- **Ideal code (`dream.cp`):** the same line without `!`: `chain([m], ...)` is never empty.
- **What happened:** `mean` returns `?F64` (empty input has no mean), and printing it without `!`
  is E0606 "has type `?F64`, which does not implement `Show`". The `!` is a check the reader must
  prove by hand.
- **General fix:** non-emptiness carried by the type (a `chain` that starts with a literal list is
  non-empty), so `mean`/`min`/`max` return plain values there.
- **Goal it hurts:** hard to get wrong. **Severity:** low.

## Measurements

Contestant-written tokens: port 234, original 350 (`corpus/audit.py` count). Emitted C++: 25891 bytes (`cpl build`, judge `generic`).
