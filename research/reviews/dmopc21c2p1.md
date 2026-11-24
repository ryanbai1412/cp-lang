# dmopc21c2p1

Score: 4/5
Dream: 3/5

Scored cold against the checklist, the dream and the port.

Sort the bosses, then sweep from the tallest: cutting every boss above height a costs `h a` plus `p` times the excess, and the best over all a (or no cut, `p` times the total) is the answer.

## Checklist failures

- `above` and `above_sum` are two running counters beside the loop; a suffix count and sum over the sorted list would state the cost formula directly.
- `var boss` is read then sorted in place; `read().sorted()` would keep it a `let`.

## Gaps that need the language

- The dream's ternary search is slower than the sweep, so the dream scores lower than the port: research/frictions/dmopc21c2p1.md.
- The port runs 1.6x the C++ at `--max` (0.18 s against 0.11 s), over R10's 1.1x: perf friction in research/frictions/dmopc21c2p1.md.
