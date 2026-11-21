# dmopc20c7p5

Score: pending cold rescore (port uses `std.flow.max_closure` owning src/snk, bounded `n`/`m`, `domain Item = 1..=n` eliminating `u - 1`, bulk `[Pair](u: Item, v: Item, w: Int)` read, `16,628 B`, verified 100 ASan+UBSan cases vs brute force; sum-domain gap filed in `research/frictions/dmopc20c7p5.md`)

Reviewed cold on main ac43c2db. The header (lines 1-9) names project selection, but the port hand-builds the min-cut network on a flattened `Node = 0..n + m + 2` (lines 17-32). That is raw `n + e.to_idx` offsets, a `u - 1` base shift and unbounded `(Int, Int)` reads (line 12), while `max_closure` (`stdlib/flow.cp:267`) already solves this model.

## Checklist failures

- A hand-rolled standard algorithm (lines 18-34): the project-selection cut is `max_closure(profit, needs)`.
- Raw index arithmetic (lines 19, 23, 28, 31-32): `Node(n + m)`, `Node(i.to_idx)`, `Node(u - 1)`.
- Unbounded input (line 12): `(n, m): (Int, Int)` gives no ranges.
- Inline reads inside the build loop (line 27).

## Rewrites owed with today's language

- `max_closure` over `[Item]` profits plus pair nodes. Until sum domains land, keep one `Node` domain, but let the library own the source and sink.
- Bounds on `n`, `m` and the pair fields; read pairs as `[Pair](u: Item, v: Item, w: Int)` after `with io(base: 1)` lands, or shift once at the read.

## Gaps that need the language

- Sum domains `Item + Pair` (spec §21.11), logged in decisions.md as a known gap. No research/frictions/dmopc20c7p5.md exists; flag it.

Dream: 4/5

Dream scored cold on main ac43c2db (first score). It reads like an exemplar: profit per node, needs edges and one `max_closure` call (lines 14-19). Syntax not yet accepted: `with io(base: 1)` (line 7) is not in decisions.md, and is the top dream parse gap; sum domains (line 14) are planned, not landed. The trailing alternative (line 30) uses a free `sum(...)` (§53). Passes the dream bar.
