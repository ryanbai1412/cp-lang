# coci21c6p3

Score: 3/5
Dream: none

The port reads like the header's reverse-elimination argument: `Graph.directed(wires)` and `out.transpose()` (lines 19–20) supply the two directions, and `while let b = todo.pop():` (line 38) drives the degree peel. The closure `fn offer(b: Ball):` (line 31) owns eligibility bookkeeping, while `for step in order.rev: print(step.ball, step.charge)` (line 49) directly expresses reconstruction (idiomatic-code/R2, R7). The missing dream caps an otherwise readable port at 3.

## Checklist failures
- Dream-first process requirement: there is no `dream.cp` and no `## Dream` in the friction report, so there is no independent ideal against which to assess the port.

## Owed rewrites
- Seed the remaining-wire counters from the graphs instead of counting them by hand (L26–27, `out_left[w.u] += 1` / `in_left[w.v] += 1`): `var out_left = [Ball] b => out.deg(b)` and the same over the transpose; `Graph.deg` is in std/graph.cp.
- Supply the missing dream.

## Language gaps
- Performance (idiomatic-code/R10, in the friction): the reported maximum-input times are 0.12 s versus 0.10 s with the C++ stack limit removed, or 1.2x. This exceeds the 1.1x target; the report records the measurement but does not isolate a cause in the peel loop (lines 38–45, beginning `while let b = todo.pop():`).

## Dream notes
None: there is no `dream.cp` and no `## Dream` in the friction report, so the port is capped at 3.
