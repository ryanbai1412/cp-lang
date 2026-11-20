// cp: 0.1 1f59ac0+dirty
// AtCoder DP R Walk: count walks of length k in a directed graph: the sum of
// the entries of the adjacency matrix raised to the k-th power.
const P = 1000000007
type Z = Mod<P>
let n: 1..=50 = read()
let k: 1..=1e18 = read()
domain V = 0..n
type Mat = [V, V]Z
let edge: [V, V]0..=1 = read()

fn mul(a: Mat, b: Mat) -> Mat:
    [V, V] (i, j) => sum(a[i, m] * b[m, j] for m in V)

var step: Mat = [V, V] (i, j) => Z(edge[i, j])
var walks: Mat = [V, V] (i, j) => Z(if i == j: 1 else: 0)
var left = k
while left > 0:
    if left % 2 == 1: walks = mul(walks, step)
    step = mul(step, step)
    left /= 2
print(sum(walks[i, j] for i in V for j in V))
