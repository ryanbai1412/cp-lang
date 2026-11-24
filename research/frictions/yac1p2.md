# yac1p2 — A Boring Problem

## `Few + Few` is rejected, so pairwise count sums go through `Int(...)`
- **Where:** `port.cp`, the pairing loop: `if Int(b) + Int(b2) < 3 and Int(w) + Int(w2) < 3:`
- **Dream:** `for (b, w) in Few * Few ... if b + b2 < 3 and w + w2 < 3` (the Dream below).
- **What felt wrong:** `b + b2` on two `Few` positions is E0309 (`+` can't be applied to `Few` and `Few`); a counting domain whose labels are counts wants their sum as an `Int`. Four nested loops stand in for one loop over `Few * Few` pairs.
- **Goal it hurts:** express.
- **General fix:** a product of domains as a domain (`Few * Few`) with tuple positions, and `Int`-valued `+` between positions of one integer-labelled domain.
- **Severity:** low.

## Two `if let` levels to step a pair of counts
- **Where:** `if let Some(b1) = Few.at(b + mb): if let Some(w1) = Few.at(w + mw): ...`
- **Dream:** `if let (b1, w1) = (Few.at(b + me.0), Few.at(w + me.1))` (both present).
- **Goal it hurts:** express.
- **General fix:** a tuple pattern over optionals that matches when every part is `Some`.
- **Severity:** low.

## Old port no longer compiled: an `Int` element read from a list is "lent"
- **Where:** the v0.2 port's BFS, `let v = adj[u][j]; ... order.push(v)`: E0603 "would keep the lent element `v`". `Int` is a copy value; the checker treats it as a lend. Reported to Core with a 7-line repro. The rewrite uses `Rooted`, so it no longer needs the hand BFS.
- **Goal it hurts:** hard-to-get-wrong (a false error).
- **Severity:** medium.

## Dream

The ungated ideal (it does not compile today, so it is kept here rather than as `dream.cp`):

```cp
// Source: Yet Another Contest 1 P2 (A Boring Problem), https://dmoj.ca/problem/yac1p2
// dream: count the complement, paths with at most two nodes of each colour, by a tree DP
let n = read<3..=200_000>()
domain V = 1..=n
domain Few = 0..3            // a count of one colour that is still below three
let black = read<[V]Char>().map(c => c == 'B')
let t = Rooted(Graph.undirected(read<Tree<V>>()), root: V(1))

// down[u][b, w]: paths from u down into its subtree with b black and w white nodes
var down: [V][Few, Few]Int = 0
var calm = 0
for u in t.order.rev:
    let me = if black[u]: (Few(1), Few(0)) else: (Few(0), Few(1))
    down[u][me] = 1
    calm += 1
    for c in t.children(u):
        calm += sum(down[u][b, w] * down[c][b2, w2] for (b, w) in Few * Few
                    for (b2, w2) in Few * Few if b + b2 < 3 and w + w2 < 3)
        for (b, w) in Few * Few:
            if let (b1, w1) = (Few.at(b + me.0), Few.at(w + me.1)): down[u][b1, w1] += down[c][b, w]
print(n + choose2(n) - calm)
```
