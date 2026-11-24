# apio09p3

Score: 3/5
Dream: 4/5

Cold review of `port.cp` against `dream.cp`, without the C++. Std `scc` and `condense` give the
DAG of components, each robbed whole, and `best` over `C.rev` is the longest path to a pub
component. That is the editorial. The reads, though, are untyped and the fold is by hand.

## Checklist failures
- idiomatic-code/R5: `let (n, m): (Int, Int)` and `cash: [Spot]Int` (lines 10, 14) drop the
  statement's N, M <= 5e5 and cash <= 4000, and `p` is `Int` too. The `// Bounds:` line omits
  them as well. Unreported.
- idiomatic-code/R3: the successor fold (lines 32-35) threads `var after: ?Int` with
  `max(after ?? b, b)` by hand. The dream's `dag[c].filter_map(d => best[d.to]).max()` is the
  fold. The friction reports the let-bound `.max()` typing as `T` (E0309), so this half is
  reported.
- `has_pub: [C]Bool` (line 25) is a flag table where the dream's `Set` of pub components reads
  directly.

## Owed rewrites
- Type the reads: `(1..=5e5, 1..=5e5)`, `cash: [Spot]0..=4000`, `p: 1..=n`, and list them in
  `// Bounds:`.
- `scc(g)`/`condense(g, comp)` if std takes the graph, as the dream writes them.

## Language gaps
- `.max()` bound by `let` typed `T` rather than `?T`: in the friction.
- `@memo(array)` recursion through a closure overflowing the interpreter stack: in the friction.

## Dream notes
Reads like the editorial, but it keeps the untyped `(Int, Int)` and `[V]Int` reads, which it
should type from the statement.
