# ucup4s25c

Score: pending cold rescore (adopted std.marks.Interner<(Int, Int)>, bounded (a,b,c) to 1..=10**9, stated distinctness of triples, used rep[comp[u]] ??= u for first SCC vertex)
Dream: pending cold rescore (std.marks.Interner, bounded triples, stated distinctness, first-vertex rep)

Baseline review, cold, on main 53a08f6f (port = dream). Both files check with zero errors. SCC plus per-component BFS mod 3 is the right structure, and the header (lines 3-9) states the test. Lines 15-18 hand-write an interner where `std.marks.Interner<(Int, Int)>` exists (I probed `id`/`len`).

## Checklist failures

- `verts` plus `get_or_insert_with` plus `verts.len` (lines 15-18) is `Interner`.
- `(a, b, c)` are unbounded `Int` (line 21), and the header doesn't say whether they're distinct. It calls the pairs "distinct ordered pairs" (line 5) but never says why `(x, x)` can't occur.
- Lines 29-31 build one representative per SCC through an optional table. Say that `rep` keeps the last vertex seen, or take the first vertex of each component directly.

## Rewrites owed with today's language

- Use `Interner<(Int, Int)>`, bound the triple, and state the distinctness assumption.

## Gaps that need the language

- The `Map<(Int, Int), Int>` slowdown is measured in `research/frictions/ucup4s25c.md`. Re-measure with `Interner`.
