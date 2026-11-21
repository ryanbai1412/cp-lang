// Universal Cup 5 Stage 1 (Nizhny Novgorod) A: a rooted tree on n vertices
// grows by m vertices, each attached to a uniformly random existing vertex.
// State (leaves, unmarked internal vertices) -> weighted count; the initial
// row is the polynomial prod (deg(v) + x) over internal vertices.
type Z = Mod<998_244_353>

let (n, m): (1..=500, 0..=500) = read()
domain V = 1..=n
let parent: [0..n - 1]V = read()
var deg: [V]Int = 0
for p in parent: deg[p] += 1

let leaves = count(d == 0 for d in deg)
var unmarked: [0..=n]Z = 0
unmarked[0] = 1
var ulen = 1
for d in deg if d > 0:
    // multiply the polynomial by (d + x)
    for z in (1..=ulen).rev: unmarked[z] = unmarked[z] * d + unmarked[z - 1]
    unmarked[0] *= d
    ulen += 1

let size = n + m + 2
var dp: [0..size, 0..size]Z = 0
for j in 0..ulen: dp[leaves, j] = unmarked[j]
for z in 0..m:
    var next: [0..size, 0..size]Z = 0
    for i in 0..=n + z:
        for j in 0..=n + z - i:
            let v = dp[i, j]
            // attach below a leaf: it stays counted either way
            next[i, j] += v * i
            next[i, j + 1] += v * i
            // attach below an unmarked vertex
            if j > 0:
                next[i + 1, j - 1] += v * j
                next[i + 1, j] += v * j
            // attach anywhere else
            next[i + 1, j] += v * (n + z - i - j)
    dp = next

var ans = sum(dp[i, 0] / i for i in 1..=n + m)
for z in 0..m: ans /= n + z
print(ans)
