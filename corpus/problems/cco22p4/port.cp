// Source: CCO 2022 Day 2 (Bi-ing Lottery Treekets), https://dmoj.ca/problem/cco22p4
// Problem: a rooted binary tree of N <= 4000 nodes; K <= 4000 numbered balls each have a drop
//   node. Balls are dropped in any order; a ball falls into an empty child, picks either side
//   when both are empty on its drop, keeps its direction afterwards, and stops when no child is
//   empty. Count the distinct final ball placements with every ball dropped, mod 1e9+7.
// Solution: spare(u) = subtree size - balls dropped in it; any negative spare means 0. dp(u, k)
//   counts placements in u's subtree when k balls arrive from above (they keep their direction).
//   The incoming balls fill the child on their side first; for each split i of the balls dropped
//   at u, multiply binomials for which balls go where and falling factorials for their order
//   among the arrivals; when both children fill, one ball stops at u. Memoized over (u, k).
// original: corpus/submissions/4801884.cpp
// Bounds: n: 1..=4000; m: 0..=4000; kids.l, kids.r: 0..=n
let (n, m): (1..=4000, 0..=4000) = read()
domain Node = 1..=n
type Z = Mod<1_000_000_007>
let comb = Comb<Z>(n + m)
var here: [Node]Int = 0
for u in read<[m]Node>(): here[u] += 1
let kids: [Node](l: 0..=n, r: 0..=n) = read()
let left = [Node] u => Node.at(kids[u].l)
let right = [Node] u => Node.at(kids[u].r)
var is_right: [Node]Bool = false
for u in Node:
    if let r = right[u]: is_right[r] = true

@memo(array)
fn spare(u: Node) -> Int = 1 - here[u] + (left[u].map(spare) ?? 0) + (right[u].map(spare) ?? 0)

domain Arrive = 0..=n
fn ways(u: Node, k: Int) -> Z = if k > spare(u): Z(0) else: fill(u, Arrive(k))

@memo(array)
fn fill(u: Node, k: Arrive) -> Z:
    let (h, a) = (here[u], k.ord)
    match (left[u], right[u]):
        (Some(l), Some(r)) =>
            let (near, far) = if is_right[u]: (r, l) else: (l, r)
            var total = Z(0)
            for i in 0..=h:
                let al = min(a, spare(near) - i)
                if al < 0: continue
                let ar = a - al
                let past = ar + h - i - Int(al + i == spare(near) and ar + h - i > spare(far))
                total += comb.choose(h, i) * comb.arrange(al + i, i) * comb.arrange(ar + h - i, h - i)
                    * ways(near, al + i) * ways(far, past)
            total
        (Some(c), None) | (None, Some(c)) => comb.arrange(a + h, h) * ways(c, a + h - Int(a == spare(u)))
        (None, None) => Z(1)

print(if Node.any(u => spare(u) < 0): Z(0) else: ways(Node.first!, 0))
