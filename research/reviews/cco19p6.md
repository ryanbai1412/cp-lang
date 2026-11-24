# cco19p6

Score: 2/5
Dream: 4/5

Tails are a product domain `(Word, Cut)` and `lay` is the one transition; the answer is the nearest empty tail. But the arcs are materialised in a push loop because the friction says `dijkstra` cannot take a function, and it can.

## Checklist failures

- idiomatic-code/R13: `var arcs` and its push loop (lines 29-32) and `Graph.of(arcs)` build the graph by hand: under 0.2, `dijkstra(t => W.filter_map(x => lay(t, x)), from: [((W(1), C(0)), 0)])` over `domain T = (W, C)` runs and prints the distance. The friction's remaining gap does not hold.

## Owed rewrites

- `let dist = dijkstra(t => if ended(t): [] else: Word.filter_map(x => lay(t, x)), from: starts)`, dropping `arcs`; remove the friction's "Remaining friction".

## Language gaps

None open.

## Dream notes

- The fragment is the program with the arcs given by `lay`; it is runnable today.
