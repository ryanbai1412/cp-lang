# ucup4s25c

Score: 4/5
Dream: 4/5

Baseline review, cold, on main 53a08f6f (port = dream). Both files check with zero errors. SCC plus per-component BFS mod 3 is the right structure, and the header (lines 3-9) states the test. Lines 15-18 hand-write an interner where `std.marks.Interner<(Int, Int)>` exists (I probed `id`/`len`).

## Checklist failures

- `verts` plus `get_or_insert_with` plus `verts.len` (lines 15-18) is `Interner`.
- `(a, b, c)` are unbounded `Int` (line 21), and the header doesn't say whether they're distinct. It calls the pairs "distinct ordered pairs" (line 5) but never says why `(x, x)` can't occur.
- Lines 29-31 build one representative per SCC through an optional table. Say that `rep` keeps the last vertex seen, or take the first vertex of each component directly.

## Rewrites owed with today's language

- Use `Interner<(Int, Int)>`, bound the triple, and state the distinctness assumption.

## Gaps that need the language

- The `Map<(Int, Int), Int>` slowdown is measured in `research/frictions/ucup4s25c.md`. Re-measure with `Interner`.

## Cold rescore (3f044ab9)

Port 4, dream 4 (port = dream). Rescored cold on main 3f044ab9; both check clean. `std.marks.Interner` replaces the hand map (lines 17, 23), the triples are bounded (line 21), and the header states distinctness (lines 3-5).
- Minor: every SCC has a vertex, so `rep: [C]?V` (line 29) is never `none` after line 30. The `if let` in line 31 filters nothing.
- Minor: `raw_edges` is re-wrapped into `V` (line 26) only because `domain V` exists after interning; that is inherent to the order, but one sentence would say why.
