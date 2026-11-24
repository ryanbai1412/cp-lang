# dmopc21c8p1

Score: 3/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port.

A greedy line over the sorted heights: take the smallest unused, move every big one it can pair with onto a stack, and follow it with one of those.

## Checklist failures

- `lo`, `hi`, `partners`, `line` and `stuck` are five mutable pieces of state for one two-pointer greedy.
- `let x = if lo < hi: a[lo] else: partners.pop()!` then a second `if lo < hi: lo += 1` repeats the branch.
- The loop runs on `not stuck` and a flag set inside, a `break` with a result written as state.
