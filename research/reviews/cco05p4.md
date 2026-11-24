# cco05p4

Score: 3/5
Dream: 3/5

The search is the statement: for each length the leftmost window is `(0..=n - len).find(...)` over `prefix(a)` cuts, and the reads carry the statement bounds. The odd-only Bitset sieve is explained in two comment lines and stands in for a slow std `is_prime`, both reported; 0.95 s against 1.29 s for the C++.

## Checklist failures

- idiomatic-code/R8: `@multitest` (line 22) is the removed decorator spelling; `@!multitest` is the one divider (decision §20).

## Owed rewrites

- `@!multitest` for `@multitest`.
- `[a[i] for i in l..l + len]` (line 29) is the slice `a[l..l + len]`, which prints the same under 0.2 (as the dream has it).

## Language gaps

- The hand-rolled odd-only sieve: research/frictions/cco05p4.md ("std `is_prime` rebuilds its constant bases per call", "No odd-only or bit-packed sieve in std").

## Dream notes

- The dream uses `@multitest` too (idiomatic-code/R8, caps at 3); otherwise it is the port with std `is_prime` and the slice, and would be a 4 with `@!multitest`.
- A dream under 4 is sent back and caps the port at 3 (research/reviews/README.md); the port itself reads at 4.
