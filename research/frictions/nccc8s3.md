# Friction report: nccc8s3

## No `last_index_of` on a domain table

- **Where:** `var last: [V]Int = 0` / `for (i, x) in a.items: last[x] = i`.
- **Ideal code (`dream.cp`):** `let last = a.last_index_of()`.
- **What happened:** `cpl run` on the dream line is E0203 "a `[4]K` has no method
  `last_index_of`"; the port fills the table with a loop.
- **General fix:** a std `last_index_of` (and `first_index_of`) returning `[V]?Key` for a sequence
  of domain values.
- **Goal it hurts:** express. **Severity:** low.

## Measurements

Contestant-written tokens: port 166, original 289 (`corpus/audit.py` count). Emitted C++: 10849 bytes (`cpl build`, judge `generic`).
