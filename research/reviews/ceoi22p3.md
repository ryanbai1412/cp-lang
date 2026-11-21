# ceoi22p3

Score: 3/5

Baseline review, cold, on main 96359635. Reads like the editorial. The header (lines 7-14) states the auxiliary-tree insight and why the ties are connected. But the trees are read by hand with a `-1` root sentinel, and depths come from a hand stack traversal with a `known` flag array. No dream exists, so the score is capped at 3.

## Checklist failures

- `read_tree` (lines 24-31) scans `boss` for `-1` by hand; `Rooted.from_parents(par: [V]?V)` (`stdlib/tree.cp:50`) exists, and the WISH TR3 comment is out of date for it.
- Hand stack traversal with `known` flags (lines 58-70); `bfs` (`stdlib/graph.cp:659`) over the tie graph, or a weighted walk, says it.
- `col: Int` 1-or-2 selector (lines 48, 53, 74-75) where a projection `replies[i] => (a, b)` passed in is the general form.
- `picked[i.ord + 1]` (lines 42, 51) where `picked.windows(2)`/`pairs` exists.
- `q: Int` (line 18) is unbounded.
- WISH TR15-TR17 comments (lines 38, 43, 62) have no friction report.

## Rewrites owed with today's language

- `Rooted.from_parents` for both trees; a graph traversal from `std.graph` for the depths; `windows` for the adjacent picks.

## Gaps that need the language

- Interactive flush (TR16) and `while let x = stack.pop()` (TR17) are genuine; no research/frictions/ceoi22p3.md exists, so flag it.
