# dmopc20c7p5

Score: 4/5

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

## Cold review (HEAD)

Port 3/5 → 4/5, rescored cold on current main. It checks with zero errors, and `std.flow.max_closure` owns the cut. The only workarounds are the two reported gaps: `Node` is flattened through `item_node`/`pair_node` because sum domains are missing (lines 18-21), and the synthetic `Need` domain with its `k.ord / 2` decode is there because `max_closure` takes an `[E]` edge table, not a `List` (lines 27-31).

Dream 4/5 holds (unchanged). Withdrawn: the earlier §53 note on line 30's `sum(pairs[p].w for p in Pair)`, because a generator fold is canonical (§53.1). Owed now: the dream lacks `use std.flow.{max_closure}` (line 19, E0201), while the port has it.
