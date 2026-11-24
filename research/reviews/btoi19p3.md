# btoi19p3 review

Score: 2/5
Dream: 2/5

Cold review of `port.cp` against the friction's `## Dream`, without the C++. The header derives
the cut-subtree test and the `far[R] + min(shop_under[t] - 2 far[t])` formula, and why a
depth-indexed table in preorder holds the path to the exit. The offline answering is clear.
The subtree minimum, though, is a hand sweep that std already provides.

## Checklist failures
- idiomatic-code/R3: `shop_under` is filled by a reverse-preorder `min=` sweep (lines 38-42).
  std `Rooted.up(op, lift:, finish:)` (std/tree.cp) is that fold, and `min.opt()` is the
  `?Int` monoid. The friction said "std's `Rooted` has no fold over subtrees", which is wrong.
- idiomatic-code/R3: `let cut = if tree.parent(u) == w: u else: w` (line 61) re-derives std
  `tree.lower(u, w)`. Unreported.

## Owed rewrites
- Fold `shop_under` with `tree.up` over `min.opt()`, joining a shop's own `far` in `finish`,
  and drop the sweep (the friction now marks it not a gap, with a run).
- `let cut = tree.lower(u, w)`.

## Language gaps
- An offline path-query helper over preorder (`offline(..., path: Depth)`): in the friction,
  without a run. It is a std proposal, not a missing language feature.

## Dream notes
The dream is two lines and the second ends in `...`: the query algorithm, the cut test and the
`Fate` result are not there. Write the whole program (send back).
