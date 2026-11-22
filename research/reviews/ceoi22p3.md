# ceoi22p3

Score: 4/5
Dream: 4/5

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

## Cold review (HEAD)

Dream 3/5 (new); sent back. The header (lines 1-14) derives the auxiliary-tree ties and the single BFS, and lines 20-21 and 26 are the right spellings. I probed `read<[V]?V>(none: -1)` and `.pairs` on a `List`; both work.
- Line 23 fails (E0603): `[v for v in one.at][..k]` keeps a lent slice in a `var`. Use the port's comprehension, `[one.at[one.Pre.at(i)] for i in 0..k]`, or `.to_list()` the slice. That makes it a 4.
- Line 24's `print(picked)` prints the k vertices on one line, while the port prints one per line (line 27). State which the judge expects.
- Minor: `q` (line 16) is read and never used.

Port 3/5; sent back. It checks with zero errors, and the header and BFS are the dream's. Owed today, both probed:
- Lines 21-24 decode the `-1` parent by hand, twice; `read<[V]?V>(none: -1)` says it once.
- Line 29 builds adjacent pairs from `picked[i.ord + 1]`; `picked.pairs` does it.

## Cold review (HEAD, after rewrite)

Port 3 → 4/5, dream 3 → 4/5 (port = dream), rescored cold on current main. Both check with zero errors. The E0603 slice (dream line 23), the hand `-1` parent decoding, the index pairs, the unused `q`, and the output format (line 25) are all fixed.
- Minor: `[(a, b) for (a, b) in picked.pairs]` (line 28) is an identity comprehension, and `ask_pairs[i.ord]` (line 40) indexes a `List` by an `Ask` position's `.ord`.
