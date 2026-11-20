// Universal Cup 5 Stage 1 (Nizhny Novgorod) A: a rooted tree on n vertices
// grows by m vertices, each attached to a uniformly random existing vertex.
// State (leaves, unmarked internal vertices) -> weighted count; the initial
// row is the polynomial prod (deg(v) + x) over internal vertices.
const MOD = 998244353
type Z = Mod<MOD>
let (n, m): (1..=500, 0..=500) = read()
let parent: [0..n - 1]1..=n = read()
var deg: [0..n]Int = 0
for p in parent: deg[p - 1] += 1

let leaves = count(d == 0 for d in deg)
var unmarked = [Z(1)]
for d in deg if d > 0:
    // multiply the polynomial by (d + x)
    unmarked.push(Z(0))
    for z in (1..unmarked.len).rev: unmarked[z] = Z(d) * unmarked[z] + unmarked[z - 1]
    unmarked[0] *= Z(d)

let size = n + m + 2
var dp = [[Z(0) for _ in 0..size] for _ in 0..size]
for j in 0..unmarked.len: dp[leaves][j] = unmarked[j]
for z in 0..m:
    var next = [[Z(0) for _ in 0..size] for _ in 0..size]
    for i in 0..=n + z:
        for j in 0..=n + z - i:
            let v = dp[i][j]
            // attach below a leaf: it stays counted either way
            next[i][j] += v * Z(i)
            next[i][j + 1] += v * Z(i)
            // attach below an unmarked vertex
            if j > 0:
                next[i + 1][j - 1] += v * Z(j)
                next[i + 1][j] += v * Z(j)
            // attach anywhere else
            next[i + 1][j] += v * Z(n + z - i - j)
    dp = next

var ans = Z(0)
for i in 1..=n + m: ans += dp[i][0] / Z(i)
for z in 0..m: ans /= Z(n + z)
print(ans)
