// Codeforces 2138 E2: Determinant Construction (Hard Version).
// The tridiagonal matrix with diagonal d, 1 above and -1 below has determinant
// K(d), the continuant: K(d + [c]) = c * K(d) + K(d without its last entry).
// With entries in {0, 1} that recurrence is the subtractive Euclid walk run
// backwards, so x is reached from (z, x - z) iff gcd(x, z) = 1. Trying z
// outward from x / phi finds a walk of length <= 50 for every x <= 1e7
// (exhaustively verified).
const MAX_N = 50

fn continuant_diag(x: Int, z: Int, lim: Int) -> ?List<0..=1>:
    var (a, b) = (z, x - z)
    var diag: List<0..=1> = [1]
    while diag.len < lim:
        if a < b:
            diag.push(0)
            (a, b) = (b, a)
        else:
            diag.push(1)
            if a == b: return if a == 1: diag else: none
            (a, b) = (b, a - b)
    none

fn find_diag(x: Int, lim: Int) -> List<0..=1>:
    let z0 = max(1, min(x - 1, x * 618_033_989 / 1_000_000_000))
    for d in 0..x:
        if z0 + d < x:
            if let diag = continuant_diag(x, z0 + d, lim): return diag
        if d > 0 and z0 - d >= 1:
            if let diag = continuant_diag(x, z0 - d, lim): return diag
    []

@!multitest
let x: 0..=10_000_000 = read()
let diag: List<0..=1> = if x <= 1: [x] else: find_diag(x, MAX_N)
domain N = 0..diag.len
let m: [N, N]Int = [N, N] (i, j) => match j - i:
    0 => diag[i.ord]
    1 => 1
    -1 => -1
    _ => 0

print(N.count)
for i in N: print([m[i, j] for j in N])
