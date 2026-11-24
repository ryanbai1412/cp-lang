# coci21c4p4

Score: 3/5
Dream: 4/5

The port binary-searches the covering radius and greedily places parks before an uncovered town becomes unreachable from above; `for v in tree.order.rev:` (L31) with propagation through `up.w` (L42–43) is the header's argument. It runs the same algorithm as `dream.cp`, but with the dream's one-line guards spelled as nested `if let`s, and its friction describes a gap the dream does not have.

## Checklist failures
- idiomatic-code/R2, unreported: L32–34 (`if let f = far[v]:` / `if let d = near[v]:` / `if f + d <= reach:`) and L37–38 nest what the dream writes as one guard, `if let f = far[v], let d = near[v], f + d <= r:` (dream L15); multi-binding guards with a condition check today (as in cf2152g's port).
- idiomatic-code/R2, unreported: L35–36 take both `tree.up_arc(v)` and `tree.parent(v)!` where the dream uses `t.parent(v)` and `t.w(v)`; L51–55 pad the answer with a `park` mask and a push loop where the dream writes `V.mask(chosen)` and `V.filter(..).take(k - chosen.len)`.
- Friction accuracy: the friction's gap is a missing arc-aware bottom-up fold, but `dream.cp` keeps the same `order.rev` sweep, so the report does not describe the distance between port and dream.

## Owed rewrites
- Rewrite `parks` with the dream's comma guards and parent weight (dream L14–24), and the padding with `V.mask` plus `filter(..).take(..)` (dream L29–31).
- Correct the friction: either drop the fold gap or write the fold into the dream and add the R11 run for it.

## Language gaps
- An arc-aware bottom-up fold is claimed in the friction with no `cpl run`; `dream.cp` does not use one, so nothing in the dream is blocked today.

## Dream notes
The dream states the greedy invariant in its comment, keeps optional distances instead of sentinels and pads with `V.mask` and `take`. It is short of 5 only because the read folds the roads into the `Rooted` constructor with an inline bound that the header does not restate.
