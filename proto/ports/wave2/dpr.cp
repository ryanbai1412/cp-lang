// AtCoder DP R Walk: count the walks of length k in a directed graph: the sum
// of the entries of the k-th power of its adjacency matrix.
type Z = Mod<1_000_000_007>
let (n, k): (1..=50, 1..=1e18) = read()
domain V = 0..n
type Mat = [V, V]Z
let edge: Mat = read()

fn mul(a: Mat, b: Mat) -> Mat:
    var c: Mat = 0
    for i in V:
        for m in V:
            for j in V: c[i, j] += a[i, m] * b[m, j]
    c

var walks: Mat = [V, V] (i, j) => Int(i == j)
var (step, left) = (edge, k)
while left > 0:
    if left % 2 == 1: walks = mul(walks, step)
    step = mul(step, step)
    left /= 2
var total: Z = 0
for i in V:
    for j in V: total += walks[i, j]
print(total)
