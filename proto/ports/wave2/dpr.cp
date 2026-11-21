// AtCoder DP R Walk: count the walks of length k in a directed graph: the sum
// of the entries of the k-th power of its adjacency matrix.
use std.matrix.{Mat}

type Z = Mod<1_000_000_007>
let (n, k): (1..=50, 1..=1e18) = read()
let edge: [0..n * n]Z = read()
let walks = Mat.from_flat(n, n, edge).pow(k)
print(sum(walks.data))
