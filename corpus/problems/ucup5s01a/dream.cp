// Universal Cup 5 Stage 1 (Nizhny Novgorod) A: a rooted tree on n vertices
// grows by m vertices, each attached to a uniformly random existing vertex.
//
// Let every internal vertex mark one of its children: the number of markings
// is the product of the internal degrees, and the answer is the expected
// value of that product over the number of leaves. Count, over all ways to
// grow, the tree together with a partial marking: state (leaves, internal
// vertices still unmarked). An unmarked vertex may still be marked by any
// child added later. The initial row is prod (deg(v) + x) over internal
// vertices (mark one of the deg(v) children now, or stay unmarked). At the end
// every vertex must be marked, so the answer is sum dp[i, 0] / i, divided by
// the prod (n + z) equally likely growth sequences.
use std.poly.{Poly}

type Z = Mod<998_244_353>

let (n, m): (1..=500, 0..=500) = read()
domain V = 1..=n
let parent: [0..n - 1]V = read()
let deg = [V] v => parent.count(v)
let unmarked = prod(Poly<Z>([d, 1]) for d in deg if d > 0)

domain Count = 0..=n + m
var dp: [Count, Count]Z = 0
let init_leaves = Count(deg.count(0))
for (j, c) in unmarked.c.items: dp[init_leaves, Count(j)] = c
// A read past either end of the table is 0.
fn at(dp: [Count, Count]Z, i: Int, j: Int) -> Z =
    if let a = Count.at(i), let b = Count.at(j): dp[a, b] else: 0
for z in 0..m:
    let others = n + z
    // `i` leaves and `j` unmarked; every other vertex is marked internal
    dp = [Count, Count] (i, j) => (
        // below a leaf: it becomes internal, marked now (j) or later (j - 1)
        Z(i.ord) * (
            dp[i, j] + at(dp, i.ord, j.ord - 1)
        )
        // below an unmarked vertex, marking the new leaf now
        + (j + 1) * at(dp, i.ord - 1, j.ord + 1)
        // below an unmarked vertex without marking, or below a marked one
        + (others - (i - 1)) * at(dp, i.ord - 1, j.ord)
    )

let numer = sum(dp[i, Count.first!] / Z(i.ord) for i in Count if i > Count.first!)
print(numer / prod(Z(n + z) for z in 0..m))
