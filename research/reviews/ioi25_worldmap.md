# ioi25_worldmap

Score: 4/5
Dream: 4/5

Scored cold on main against the statement (IOI 2025 Day 1, World Map), `dream.cp` and
`port.cp`; the friction report was read only after scoring.

Dream 4/5. The header (lines 1-6) states the construction (one anti-diagonal per Euler-tour
visit, `v | ancestors | v` triples, the deep path last) and the body names `dfs_tree`,
`euler_tour(by:)`, `path` and a `Bitset`. Port 4/5: it reads as its header's argument, the
generic `create_map<V, E>` returns a sized `[2n][2n]V`, and the header derives why the 4n - 1
diagonals fill the grid exactly, so no padding is needed.

## Checklist failures

- The port's construction differs from the dream's: the port gives each back edge to the end
  whose best visit has the most room (`port.cp:23-29`) and expands every vertex once at that
  visit, where the dream tours the deepest path last and expands the first visit
  (`dream.cp:14, 21-25`). The port's version is the better one (no `while` padding, no
  `expanded` set), but the gap between them is not in the friction report (porting step 4).
  Route: the author, either to move the dream to the port's rule or to report why not.
- `port.cp:24`, `var best: [V]Int = 0` starts every vertex at visit 0, a visit of the root
  only; it is correct because the tour also ends at the root, so every other vertex has a
  visit with more room. That is a sentinel (R5) whose proof the reader must supply. Rewrite:
  `let best = [V] v => tour.items.filter(e => e.1 == v).max(by: e => room(e.0))!.0`, or a
  `domain Visit = 0..2 * n - 1` with `?Visit`. Route: the author.
- Visit numbers are raw `Int` (`port.cp:23-26, 32-34`), beside the `V` and `E` domains (R6).

## Fixed in this review

- `port.cp:35-36`: the cell formula `x - max(0, x + y - (2 * n - 1))` and the `?? d.last!`
  fallback were not derived (R1). Two comment lines now say where diagonal x + y enters the
  grid and that a short diagonal repeats its last colour. `scripts/corpus.py verify
  ioi25_worldmap` stays clean.

## Language gaps

Reported: `max_by` spelling, `Bitset.insert`/`add`, orienting back edges, `pad_to`, sized grids
across `@grader`, the root `grader.cpp`. Dream note: friction section 2 says `expanded.add(v)`
already returns whether `v` was new, so `dream.cp:23-24` can be
`if up[v].len > 0 and expanded.add(v):` today.
