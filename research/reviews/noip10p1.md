# noip10p1

Score: 5/5
Dream: 5/5

Scored cold against the checklist, the dream and the port.

The port is the dream: memory is a `Deque<Int>` of at most m words, a lookup pushes the word and evicts from the front once over capacity, and the answer is the lookup count. It passes stress against the v0.2 port and its package cases.

## Checklist failures

- None. `memory.contains(w)` is O(m) per word, but m <= 100 and n <= 1000 make that the plain reading of the statement, not a cost.
