# ioi25_worldmap

Score: 4/5
Dream: 4/5

Re-reviewed cold on fbed8d10d against the statement (IOI 2025 Day 1, World Map), `dream.cp` and
`port.cp`; the friction report was read only after scoring.

Dream 4/5. The header states the construction (one anti-diagonal per Euler-tour visit,
`v | ancestors | v` triples, the deep path last) and the body names `dfs_tree`,
`euler_tour(by:)`, `path` and a `Bitset`. Port 4/5: it reads as its header's argument, the
generic `create_map<V, E>` returns a sized `[2n][2n]V`, and the header derives the cell
formula and why the 4n - 1 diagonals fill the grid with no padding.

## Checklist failures

None. The first review's two are fixed: `port.cp:24` is now
`let best = [V] v => tour.keys.filter(i => tour[i] == v).max(by: room)!`, with no sentinel, and
the different constructions are in the friction report ("The port and the dream build the map
differently").

## Owed rewrites

- `port.cp:23-32`: visit numbers are still raw `Int` (`room`, `best`, `i == best[v]`) beside
  the `V` and `E` domains (R6). A `domain Visit = 0..2 * n - 1` over `tour.keys` would type
  them. Route: the author (minor).

## Dream notes

- `dream.cp:23-24`: `Bitset.add` returns whether the bit was new since 59bed056c (marked fixed in
  the friction report), so this is `if up[v].len > 0 and expanded.add(v):` today.
- The friction report says to move the dream to the port's rule (no `expanded` set, no
  `while` padding at `dream.cp:26`) when it is next rewritten; that rewrite would make it a 5.
