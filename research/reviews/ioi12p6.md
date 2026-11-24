# ioi12p6 review

Score: 3/5
Dream: 3/5

Scored cold against the checklist, the port and the dream (the `## Dream` section of
research/frictions/ioi12p6.md).

Correct, with `Pos`/`Gap`/`Round`/`Node` domains and a std `Fenwick`, and the header gives the
round tree and the "beats the inner maximum" rule. But the k-th survivor is a Fenwick descent
(`alive.first_ge(s + 1)!.left!`), `-1` is the identity of `max` in `top`/`inner`, and `Node`
is a hand-packed sum of leaves and rounds. The first two are reported.

## Checklist failures

- `top`/`inner: [Node]-1..n = -1` use `-1` as "no knight" (R5); reported.
- `leaf(p) = Node(p.ord)`, `node(round) = Node(n + round.ord)` pack two index spaces into one
  by offset (R6); the friction report does not list this. It is the same "sum of two domains"
  gap as packed states elsewhere; an `enum Node = Leaf(Pos) | Fight(Round)`-keyed array would
  type it.
- `for i in 0..kids[v].len` with `if i + 1 < kids[v].len` (lines 51-54) singles out the last
  child by index; `kids[v].with_next` or splitting off `.last` would say it.

## Language gaps

- Order-statistics set, optional-aware `max=`: in research/frictions/ioi12p6.md.

## Dream notes

- Names the order-statistics set and `?Rank`, then stops at `...` before the tree and the win
  counting, which is most of the program. It should show the merge into a round node and the
  root-down `wins` pass.
