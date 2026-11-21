# noi17p2

Score: pending cold rescore (port wraps `struct Block(hash: HashVal, len: Int)` + `concat(a, b)` so lengths travel with every hash and `joint` never passes lengths by hand, bounded inputs, 0 lints, `29,652 B`, verified on 50 ASan+UBSan join/cut/query traces vs brute force; length-carrying `Hash` + `+` + `Hash.empty` filed in `research/frictions/noi17p2.md`)
Dream: pending cold rescore (written: `proto/ports/scout_batch_10/dream/noi17p2.cp` using length-carrying `Hash` with `Hash.empty` and `+`)

Baseline review, cold, on main 15cc353a, after 14cdbd55 moved it onto `std.strings.RollingHash`. There is no dream, so the score is capped at 3. The header (lines 1-11) gives the block-count idea well.

## Checklist failures

- Bookkeeping the language should carry: the old `Hash` knew its length; `RollingHash.combine` (lines 38, 50) needs the right length passed by hand (`1`, `out.len`, `j + 1`), and line 49 rebuilds the length bound as `i + j + 2 > longest`. One wrong count is a silent wrong hash.
- Line 17 builds a `RollingHash` over 50 zeros only to borrow its bases, and line 33 spells the empty hash `HashVal(0, 0)`.

## Rewrites owed today

- Write the dream.
- Keep a length beside each block hash (a `(hash, len)` value, or loop `left` and `right` with `.items` so the lengths come from the indices once).

## Gaps that need the language

- A length-carrying hash value with a `+` that concatenates, and an empty identity: no friction report exists for noi17p2; one is owed for the stdlib lane.
