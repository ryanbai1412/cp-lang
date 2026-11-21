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
for (j, c) in unmarked.items: dp[deg.count(0), j] = c
for z in 0..m:
    let others = n + z
    // `i` leaves and `j` unmarked; every other vertex is marked internal
    dp = [Count, Count] (i, j) =>
        // below a leaf: it becomes internal, marked now (j) or later (j - 1)
        i * (dp[i, j] + dp.get(i, j - 1)) +
        // below an unmarked vertex, marking the new leaf now
        (j + 1) * dp.get(i - 1, j + 1) +
        // below an unmarked vertex without marking, or below a marked one
        (others - (i - 1)) * dp.get(i - 1, j)

print(sum(dp[i, 0] / i for i in 1..=n + m) / prod(Z(n + z) for z in 0..m))
