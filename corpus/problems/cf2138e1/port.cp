// Codeforces 2138 E1: Determinant Construction (Easy Version).
// The tridiagonal matrix with diagonal d, 1 above and -1 below has determinant
// K(d), the continuant: K(d + [c]) = c * K(d) + K(d without its last entry).
// With entries in {0, 1} that recurrence is the subtractive Euclid walk run
// backwards, so x is reached from the pair (z, x - z) exactly when gcd(x, z) = 1,
// and z near x / phi keeps the walk short. The side length is at most MAX_N = 80.

const MAX_N = 80

// The diagonal built by the walk from (z, x - z) down to (1, 1), if it fits in `room`.
fn walk(x: Int, z: Int, room: Int) -> ?List<0..=1>:
    var (a, b) = (z, x - z)
    var diag: List<0..=1> = [1]
    while diag.len < room:
        if a < b:
            diag.push(0)
            (a, b) = (b, a)
        else:
            diag.push(1)
            if a == b: return if a == 1: diag else: None
            (a, b) = (b, a - b)
    None

// Candidates z in 1..x, nearest to x / phi first.
fn nearest_walk(x: Int, room: Int) -> List<0..=1>:
    let z0 = max(1, min(x - 1, x * 618_033_989 / 1_000_000_000))
    for d in 0..x:
        for z in [z0 + d, z0 - d] if 1 <= z and z < x and (d > 0 or z == z0):
            if let diag = walk(x, z, room): return diag
    []

@!multitest
let x: 0..=10_000_000 = read()
let diag = if x <= 1: [x] else: nearest_walk(x, room: MAX_N)
domain N = 0..diag.len
print(N.count)
print([N, N] (i, j) => match j - i:
    0 => diag[i.ord]
    1 => 1
    -1 => -1
    _ => 0
)
