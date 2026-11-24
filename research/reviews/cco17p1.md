# cco17p1

Score: 4/5
Dream: 4/5

The greedy is the header's: take the largest cycle with `c(c - 1) / 2 <= left` by `last_true`, then chain the cycles by bridges. Output is the vertex count, the trail count and the trails.

## Checklist failures

- idiomatic-code/R1: "at most 4587 vertices for any K" (line 8) is stated, not derived.

## Owed rewrites

- Derive the vertex bound in the header (the greedy's worst K), or cite the check that found it.
- `sizes` is built in one loop and laid out in a second, threading `placed` by hand (lines 15-30); one loop can push each cycle's trails as it picks `c`.

## Language gaps

- Vertex numbers are plain `Int`, with no growable domain to hand them out: research/frictions/cco17p1.md.

## Dream notes

- `GraphBuilder` with `g.cycle(c)` and a bridge to `prev` reads like the editorial. No header; `print(g)` leaves the output format implicit.
