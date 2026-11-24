// Source: Universal Cup 5 Stage 1 (Nizhny Novgorod) A
// Problem: a rooted tree on n <= 500 vertices grows by m <= 500 vertices, each attached
//   to a uniformly random existing vertex. Print the expected product of the internal
//   vertices' degrees divided by the number of leaves, mod p.
// Solution: let every internal vertex mark one child: the markings count the product of
//   degrees. Count growths together with a partial marking by state (leaves, internal
//   vertices still unmarked), starting from prod(deg(v) + x); at the end every vertex
//   is marked, so the answer is sum dp[i, 0] / i over the prod (n + z) growth
//   sequences. O((n + m)^2 m).
// limits: 2000 ms, 256 MB
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
type Z = Mod<998_244_353>

let (n, m): (1..=500, 0..=500) = read()
domain V = 1..=n
let parent: [0..n - 1]V = read()
var deg: [V]Int = 0
for p in parent: deg[p] += 1

var unmarked = [Z(1)]
for d in deg if d > 0:
    unmarked = [
        (
            if j < unmarked.len: unmarked[j] * d else: Z(0)
        ) + (if j > 0: unmarked[j - 1] else: Z(0)) for j in 0..=unmarked.len
    ]
// leaves and unmarked vertices each number at most n + m
var dp: [0..=n + m, 0..=n + m]Z = 0
for (j, c) in unmarked.items: dp[deg.count(d => d == 0), j] = c
for z in 0..m:
    var next: [0..=n + m, 0..=n + m]Z = 0
    for i in 0..=n + z:
        for j in 0..=n + z - i:
            let v = dp[i, j]
            // below a leaf: it becomes internal, marked now or later
            next[i, j] += v * i
            next[i, j + 1] += v * i
            // below an unmarked vertex: mark the new leaf now, or not
            if j > 0:
                next[i + 1, j - 1] += v * j
                next[i + 1, j] += v * j
            // below a marked internal vertex
            next[i + 1, j] += v * (n + z - i - j)
    dp = next

print(sum(dp[i, 0] / i for i in 1..=n + m) / (0..m).map(z => Z(n + z)).prod())
