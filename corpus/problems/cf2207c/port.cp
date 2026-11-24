// Source: Codeforces 2207C (Where's My Water?), https://codeforces.com/contest/2207/problem/C
// Problem: n columns (n <= 2000, sum n <= 2000, multitest) under height h <= 1e9; column i has
//   1 <= a_i <= h dirt tiles at the bottom and water above. A drain takes all water that reaches
//   it moving down, left or right without crossing dirt. Print the most water taken by at most 2.
// Solution: the max-Cartesian tree of the heights. The subtree of u is the run of columns where
//   a[u] is the tallest; under its parent's height (h at the root) that run holds a layer
//   w = (top - a[u]) * size(u) that any drain in the subtree takes, and below a[u] the child
//   subtrees are independent. So one drain gets the best child's one-drain total plus w, and two
//   drains get w plus one drain in each of two children or both in one child. O(n).
// Bounds: n: 1..=2000; h: 1..=10 ** 9; a: 1..=h

@!multitest
use std.tree.{cartesian_tree}
let (n, h): (1..=2000, 1..=10 ** 9) = read()
domain C = 0..n
let a: [C]1..=h = read()
let t = Rooted.from_parents(cartesian_tree(a, by: (x, y) => x > y))

fn drain(u: C, top: Int) -> (one: Int, two: Int):
    var (one, both, two) = (0, 0, 0)
    for c in t.children(u):
        let d = drain(c, a[u])
        one max= d.one
        both += d.one
        two max= d.two
    let w = (top - a[u]) * t.size(u)
    (one: one + w, two: max(both, two) + w)

print(drain(t.root, h).two)
