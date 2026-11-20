// AtCoder DP contest R Walk: count the walks of exactly k steps in a
// directed graph, mod 1e9 + 7.
const P = 1000000007
type Z = Mod<P>
let n: 1..=50 = read()
let k: 1..=1e18 = read()
domain V = 1..=n
let step: [V, V]Z = read()                                          // WISH G14: read straight into Mod

fn times(a: [V, V]Z, b: [V, V]Z) -> [V, V]Z:
    [V, V] (i, j) => sum(a[i, m] * b[m, j] for m in V)
let one: [V, V]Z = [V, V] (i, j) => if i == j: 1 else: 0

let walks = power(step, k, times, one)                              // WISH G13: binary power of anything
print(sum(walks))
