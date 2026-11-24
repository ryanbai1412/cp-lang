# iati21a13

Score: 3/5
Dream: 3/5

Re-ported in v0.3 style.

## Cold review (HEAD)

Port 3/5 (capped: the dream is under 4), dream 3/5, from the friction's `## Dream`. The header (lines 6-11) names both phases: an offline pass finding each worker's first covering update, then a `DynamicOrderTree` over the preorder answering counts.

## Checklist failures

- Lines 33-48 walk the preorder with a hand-kept `open` stack of ancestors and an `undo` log to roll back the min SegTree when a subtree ends. That is a recursive DFS spelled by hand; the exemplars' rule is "undo mirrors recursion instead of a hand-kept stack" (idiomatic-code/R2). A recursive `fn visit(v)` that sets, recurses into `t.children(v)` and restores needs neither stack.
- `min(depth + k, n - 1)` clamps the depth limit twice (lines 45 and 61) (idiomatic-code/R4).
- Lines 52-53 nest `if let f = first[v]:` and `if knows[v] == 0:`. `if let f = first[v] and knows[v] == 0:` is today's spelling (decision 287); the friction's entry tries the condition first (`knows[v] == 0 and let f`), and only that order is E0103.

## Language gaps

- Depth `n` as the "doesn't know yet" sentinel in `told` (line 55): in the friction (`DynamicOrderTree<?T>`).

## Dream notes

- The dream fragment writes `for v in V where ...`; the canonical filter is an `if` (research/reviews/README.md), so it mixes spellings and caps at 3. It covers two lines, not the offline pass.
