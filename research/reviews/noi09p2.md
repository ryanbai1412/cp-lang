# noi09p2

Score: 3/5
Dream: none

Port 3/5, dream none. The friction report has no `## Dream` section and there is no `dream.cp`, so the port is capped at 3. The friction's entry says "Ideal code: as written", outside a `## Dream`. Lines as edges between cuts, `monge_paths` for the convex line cost (line 22), the total recomputed exactly from the path (line 31): the header's argument.

## Checklist failures

- The path is rebuilt from `best.prev` with a hand `while` and `reverse` (lines 27-29), and `List(Sentence.cuts)` is built twice (lines 22, 30) to turn cut positions back into cuts.

## Owed rewrites

- Add a `## Dream` to the friction; walk the path with a std predecessor-path helper if one exists, else report it.

## Language gaps

None.

## Dream notes

None.
