# ioi11p1 review

Score: 3/5
Dream: 3/5

Scored cold against the checklist, the port and the dream (the `## Dream` section of
research/frictions/ioi11p1.md).

Correct and typed (`F`, `Trail`, `State`, `Query`, `Len` domains, `?Trail` for the two best
trails, std `bfs`), but the functional-graph machinery is built by hand: the reversed
successor lists `back`, the cycle length read off `dist[next(goal)]`, and `State` packed as
`2 * f + bit`. All three are reported.

## Checklist failures

- `state(f, by_best) = State(2 * f.ord + Int(by_best))` and `F(s.ord / 2)` (lines 35, 38):
  index arithmetic for a product state (R6); reported.
- The per-query accumulation is written twice, once per branch of `if let d = dist[next(goal)]`
  (lines 59-65); a `lap: ?Int` and one loop folding `k` down when it is present would say it
  once (R4).
- `back: [State]List<(to: State)>` wraps each predecessor in a one-field record only to fit
  `bfs`'s adjacency shape.
- `2 * n - (2 * n - k) % lap` (line 62) relies on Euclidean `%` (decision 1.9) for `k > 2n`;
  correct, but the header's "reduce larger k by whole laps" does not derive the formula (R1).

## Language gaps

- Functional-graph distances/cycle length, the packed product state, and the grader task: all
  in research/frictions/ioi11p1.md.

## Dream notes

- The dream stops at `...` right where the folding by laps happens, so it does not show the
  part of the program that is hard to read. It should finish the loop, e.g.
  `ans += starts[k if k <= 2n else k.reduce_mod(lap, into: 0..=2n)]` with `lap: ?Int`, and the
  port be judged against that.
