# cco24p1

Score: 4/5
Dream: 5/5

Cold review against `dream.cp`. The port is the dream in today's syntax: read the coins and routes into domains (lines 13-14), one multi-source `dijkstra` starting each island at `-coins[y]` (line 15), print the negated distances (line 16). The header derives the negation and the virtual source (lines 6-8). 0.46-0.54 s against the C++'s 0.69-0.76 s on three max shapes.

## Checklist failures

- R1, minor: line 8 gives `O((N + M) log M)`, but all N islands enter the heap at once (line 15) and `m` may be 0; the bound is `O((N + M) log(N + M))`.

## Owed rewrites

- Fix the cost on line 8.

## Language gaps

- None; the friction reports none.

## Dream notes

- Exemplar: two header lines state the reduction, the graph and `dijkstra` are named, the index space is a domain, and the final `!` is safe because every island is a source.
