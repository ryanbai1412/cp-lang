# noip18p4

Score: 3/5
Dream: 4/5

Cold review: sorted-adjacency DFS from town 1, and with one cycle a walk per deleted edge (lines 22-40), as the header says; the best walk is a hand min loop, while the friction says it is `min`.

## Checklist failures

- R3: `var best: ?List<V> = None` and `if w.len == n and (best == None or w < best!): best = w` (lines 36-39) hand-roll a filtered minimum; `[w for e in E if ...].min()!` runs today.

## Owed rewrites

- `print([w for w in [walk(e) for e in E] if w.len == n].min()!)` in place of lines 36-40.

## Language gaps

- None; research/frictions/noip18p4.md has no gaps and describes the `min` form as the dream.

## Dream notes

- None.
