# Friction report: ioi19p2

## Dream

```cp
@grader fn find_split(n: Int, sizes: [3]Int, links: [Link](u: Spot, v: Spot)) -> [Spot]0..=3
let (small, mid, big) = [(size: a, part: 1), (size: b, part: 2), (size: c, part: 3)].sorted().tuple
let hub = Rooted(land, Spot.first!).centroid()
var group = Dsu<Spot>.of(tree.edges.filter(e => hub not in e))
group.unite_while(links.filter(e => hub not in e), (u, v) => max(group.size(u), group.size(v)) < small.size)
let seed = Spot.find(v => v != hub and group.size(v) >= small.size) else: return [Spot] _ => 0
let near = land.bfs(from: seed, through: v => group.same(v, seed)).take(small.size)
let far = land.bfs(from: hub, through: v => not group.same(v, seed)).take(mid.size)
[Spot] v => if v in near: small.part elif v in far: mid.part else: big.part
```

## The grader lives in the solution

`find_split` is a grader function; the port reads `n m a b` and the edges and prints the
labels. The accepted C++ (`corpus/submissions/5661848.cpp`) has no `main`, so it cannot be
run on stdin here; `checker.py` judges any valid split.

## No bounded, filtered BFS

`claim` is a hand-written BFS over a `Deque` with a `taken` counter and a side test. A std
`land.bfs(from:, through:)` that yields vertices lazily would make each patch
`bfs(...).take(count)`, with no queue or counter in the port.

## No sorted tuple destructuring

`var by_size = [...]; by_size.sort()` and three indexed reads stand in for sorting a fixed
triple straight into `(small, mid, big)`.

## Verification

30 + 10 random cases agree with the v0.2 port under `checker.py` (bushy trees, and stars that
are often impossible to split). The max case (n = 1e5, m = 2e5) runs in 0.03 s and passes the
checker.
