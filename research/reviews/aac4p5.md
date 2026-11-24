# aac4p5

Score: 3/5
Dream: 4/5

Cold review of `port.cp` against the ideal under `## Dream` in `research/frictions/aac4p5.md`
(there is no `dream.cp`), without the C++. The port reads like the editorial. `Dsu<V>` unites
the pairs, the groups are sorted by size, the x - 1 smallest are kept and the rest merged. A
lazy `edges(comps)` generator lists cycles, then in-group pairs, then forward cross-group
pairs, and `.take(m)` stops after m.

## Checklist failures
- idiomatic-code/R1: the `room` formula (`sum(c.len * (c.len - 1)) + (n * n - sum(c.len * c.len)) / 2`)
  is not derived in the header; "m fits in the room" does not say what the room counts.
- idiomatic-code/R13: `fn components()` is a function with a `var` and a `push`, called once
  for one value.

## Owed rewrites
- `let comps = groups.take(x - 1).collect() + [groups.drop(x - 1).flatten().collect()]`.
- One comment on `room`, as in the friction's dream.

## Language gaps
None.

## Dream notes
The friction's dream names every object the editorial does. It uses an `exit()` after the -1
case, which ports don't use; an `if/else` is the house form.
