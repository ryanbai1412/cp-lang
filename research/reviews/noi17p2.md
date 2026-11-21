# noi17p2

Score: 3/5
Dream: 4/5

Baseline review, cold, on main 15cc353a, after 14cdbd55 moved it onto `std.strings.RollingHash`. There is no dream, so the score is capped at 3. The header (lines 1-11) gives the block-count idea well.

## Checklist failures

- Bookkeeping the language should carry: the old `Hash` knew its length; `RollingHash.combine` (lines 38, 50) needs the right length passed by hand (`1`, `out.len`, `j + 1`), and line 49 rebuilds the length bound as `i + j + 2 > longest`. One wrong count is a silent wrong hash.
- Line 17 builds a `RollingHash` over 50 zeros only to borrow its bases, and line 33 spells the empty hash `HashVal(0, 0)`.

## Rewrites owed today

- Write the dream.
- Keep a length beside each block hash (a `(hash, len)` value, or loop `left` and `right` with `.items` so the lengths come from the indices once).

## Gaps that need the language

- A length-carrying hash value with a `+` that concatenates, and an empty identity: no friction report exists for noi17p2; one is owed for the stdlib lane.

## Cold review (HEAD)

Dream (new) 4/5, cold-reviewed on current main. Its only errors come from the length-carrying `Hash` with `Hash.empty` and `+`, which is reported in research/frictions/noi17p2.md (lines 16, 23, 30, 32). The header derives the block bookkeeping.
- Minor: `reach` picks the concatenation order with a `back: Bool` flag (lines 30, 35). `while … : if let w = at: … else: break` (lines 33-38) is a `while let w = at and …` header (§111).
- Minor: the header says "few cuts" but never bounds the join cost, about `n · 50²/2` hash insertions.

Port 3/5 → 3/5: blocked by the reported `Hash` gap. `Block(hash, len)`/`concat` keep each length with its hash, but line 18 builds a `RollingHash` over 50 zeros just to borrow its bases, line 21 spells the empty hash `HashVal(0, 0)`, and line 22 rebuilds single digits through `with_bases(...).all`. Once `Hash` lands, the owed change is to adopt the dream verbatim.
